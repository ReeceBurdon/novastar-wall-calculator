// NovaStar Wall Calculator: Tauri shell for Windows around the single-page app.
// Updates come two ways:
// - The page: same as the Mac app. When GitHub Pages (docs/app/) has a newer version, index.html and the MP4
//   muxer are downloaded into the app data folder and served from there from then on (fonts and anything
//   else missing fall back to the bundled copy). Most releases need nothing more.
// - The shell (this file, the installer): the Tauri updater reads a signed latest.json from the
//   windows-updater GitHub release and installs the new setup.exe. Only needed when this folder changes.
// Live output: desktop.js gives the page window.nswcDesktop (like the Mac app's preload.js), backed by the
// nswc_* commands below. Saved projects live in WebView2's localStorage for the app origin, which every
// version shares.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
  borrow::Cow,
  cmp::Ordering,
  fs,
  path::PathBuf,
  sync::{
    atomic::{AtomicBool, Ordering as AtOrd},
    Mutex, OnceLock,
  },
  time::Duration,
};
use tauri::{
  menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
  utils::assets::{AssetKey, AssetsIter, CspHash},
  App, AppHandle, Assets, Emitter, Manager, Monitor, PhysicalPosition, PhysicalSize, Runtime, WebviewUrl,
  WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;

const UPDATE_URL: &str = "https://reeceburdon.github.io/novastar-wall-calculator/app/";
const DESKTOP_JS: &str = include_str!("desktop.js");

static PAGE_VERSION: OnceLock<String> = OnceLock::new();
static WEB_DIR: OnceLock<PathBuf> = OnceLock::new();
static CHECKING: AtomicBool = AtomicBool::new(false);

fn read_ver(html: &str) -> String {
  const TAG: &str = "<meta name=\"app-version\" content=\"";
  html.find(TAG)
    .and_then(|i| html[i + TAG.len()..].split('"').next())
    .filter(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit() || c == '.'))
    .unwrap_or("0")
    .to_string()
}
fn cmp_ver(a: &str, b: &str) -> Ordering {
  let p = |s: &str| s.split('.').map(|x| x.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
  let (x, y) = (p(a), p(b));
  for i in 0..x.len().max(y.len()) {
    let o = x.get(i).unwrap_or(&0).cmp(y.get(i).unwrap_or(&0));
    if o != Ordering::Equal {
      return o;
    }
  }
  Ordering::Equal
}

/* ---- assets: the downloaded page when it's newer than the bundled one, else the bundled page ---- */
struct NoAssets;
impl<R: Runtime> Assets<R> for NoAssets {
  fn get(&self, _: &AssetKey) -> Option<Cow<'_, [u8]>> {
    None
  }
  fn iter(&self) -> Box<AssetsIter<'_>> {
    Box::new(std::iter::empty())
  }
  fn csp_hashes(&self, _: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    Box::new(std::iter::empty())
  }
}
struct PageAssets<R: Runtime>(Box<dyn Assets<R>>);
impl<R: Runtime> Assets<R> for PageAssets<R> {
  fn setup(&self, app: &App<R>) {
    let bundled = self.0.get(&"index.html".into()).map(|b| read_ver(&String::from_utf8_lossy(&b))).unwrap_or_default();
    let mut running = bundled.clone();
    if let Ok(dir) = app.path().app_data_dir().map(|d| d.join("web")) {
      let dl = fs::read_to_string(dir.join("index.html")).map(|h| read_ver(&h)).unwrap_or_default();
      if cmp_ver(&dl, &bundled) == Ordering::Greater {
        running = dl;
        let _ = WEB_DIR.set(dir);
      }
    }
    let _ = PAGE_VERSION.set(running);
  }
  fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    let k = key.as_ref();
    let data = WEB_DIR
      .get()
      .filter(|_| !k.contains(".."))
      .and_then(|d| fs::read(d.join(k.trim_start_matches('/'))).ok())
      .map(Cow::Owned)
      .or_else(|| self.0.get(key))?;
    if k != "/index.html" {
      return Some(data);
    }
    // The page carries the Mac app's CSP meta tag; here the CSP comes from tauri.conf.json (it adds the IPC origin)
    let html = String::from_utf8_lossy(&data);
    const META: &str = "<meta http-equiv=\"Content-Security-Policy\"";
    Some(match html.find(META).and_then(|i| html[i..].find('>').map(|j| (i, i + j + 1))) {
      Some((a, b)) => Cow::Owned(format!("{}{}", &html[..a], &html[b..]).into_bytes()),
      None => data,
    })
  }
  fn iter(&self) -> Box<AssetsIter<'_>> {
    self.0.iter()
  }
  fn csp_hashes(&self, key: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    self.0.csp_hashes(key)
  }
}

/* ---- live output: a borderless window filling the chosen display; the page runs in #output mode ---- */
#[derive(Default)]
struct Output(Mutex<Option<(String, PhysicalPosition<i32>)>>);

#[derive(serde::Serialize, Clone)]
struct OutState {
  open: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  label: Option<String>,
}
#[derive(serde::Serialize)]
struct Display {
  id: String,
  label: String,
  w: u32,
  h: u32,
  scale: f64,
  primary: bool,
  main: bool,
  internal: bool,
}
#[derive(serde::Serialize)]
struct Opened {
  ok: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  label: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<String>,
}

fn same(a: &Monitor, b: &Monitor) -> bool {
  a.name() == b.name() && a.position() == b.position()
}
fn mon_id(m: &Monitor, i: usize) -> String {
  m.name().cloned().unwrap_or_else(|| format!("display-{}", i + 1))
}
fn mon_label(m: &Monitor, i: usize) -> String {
  // Windows names monitors like \\.\DISPLAY2
  let n = m.name().map(|s| s.trim_start_matches(|c: char| !c.is_ascii_digit()).to_string()).unwrap_or_default();
  format!("Display {}", if n.is_empty() { (i + 1).to_string() } else { n })
}
fn main_monitor(app: &AppHandle) -> Option<Monitor> {
  app.get_webview_window("main").and_then(|w| w.current_monitor().ok().flatten())
}
fn out_state(app: &AppHandle) -> OutState {
  let o = app.state::<Output>().0.lock().unwrap().as_ref().map(|x| x.0.clone());
  OutState { open: o.is_some(), label: o }
}
fn send_state(app: &AppHandle) {
  let _ = app.emit_to("main", "nswc:output-state", out_state(app));
}
fn close_output(app: &AppHandle) {
  let had = app.state::<Output>().0.lock().unwrap().take().is_some();
  if let Some(w) = app.get_webview_window("output") {
    let _ = w.destroy();
  }
  if had {
    send_state(app);
  }
}

#[tauri::command]
fn nswc_displays(app: AppHandle) -> Vec<Display> {
  let (pri, main) = (app.primary_monitor().ok().flatten(), main_monitor(&app));
  app.available_monitors()
    .unwrap_or_default()
    .iter()
    .enumerate()
    .map(|(i, m)| Display {
      id: mon_id(m, i),
      label: mon_label(m, i),
      w: m.size().width,
      h: m.size().height,
      scale: m.scale_factor(),
      primary: pri.as_ref().is_some_and(|p| same(p, m)),
      main: main.as_ref().is_some_and(|p| same(p, m)),
      internal: false,
    })
    .collect()
}

#[tauri::command]
async fn nswc_output_open(app: AppHandle, id: Option<String>) -> Opened {
  match open_output(&app, id) {
    Ok(label) => Opened { ok: true, label: Some(label), error: None },
    Err(e) => Opened { ok: false, label: None, error: Some(e.to_string()) },
  }
}
fn open_output(app: &AppHandle, id: Option<String>) -> tauri::Result<String> {
  let mons = app.available_monitors()?;
  let main = main_monitor(app);
  let on = |m: &Monitor| main.as_ref().is_some_and(|x| same(x, m));
  let i = id
    .and_then(|id| mons.iter().enumerate().position(|(i, m)| mon_id(m, i) == id))
    .or_else(|| mons.iter().position(|m| !on(m)))
    .unwrap_or(0);
  let m = mons.get(i).cloned().or(app.primary_monitor()?).ok_or_else(|| tauri::Error::WindowNotFound)?;
  let on_main = on(&m);
  let label = format!("{} ({}×{})", mon_label(&m, i), m.size().width, m.size().height);
  close_output(app);
  let w = WebviewWindowBuilder::new(app, "output", WebviewUrl::App("index.html#output".into()))
    .title("Live output")
    .decorations(false)
    .resizable(false)
    .skip_taskbar(true)
    .visible(false)
    .focused(false)
    .shadow(false)
    .background_color(tauri::window::Color(0, 0, 0, 255))
    .initialization_script(DESKTOP_JS)
    .on_navigation(|u| u.fragment() == Some("output") || u.scheme() == "tauri")
    .build()?;
  w.set_position(*m.position())?;
  w.set_size(PhysicalSize::new(m.size().width, m.size().height))?;
  w.set_fullscreen(true)?;
  if !on_main {
    w.set_always_on_top(true)?;
  }
  w.show()?;
  let h = app.clone();
  w.on_window_event(move |e| {
    if let WindowEvent::Destroyed = e {
      if h.state::<Output>().0.lock().unwrap().take().is_some() {
        send_state(&h);
      }
    }
  });
  *app.state::<Output>().0.lock().unwrap() = Some((label.clone(), *m.position()));
  send_state(app);
  if !on_main {
    if let Some(mw) = app.get_webview_window("main") {
      let _ = mw.set_focus();
    }
  }
  Ok(label)
}

#[tauri::command]
fn nswc_output_close(app: AppHandle) -> bool {
  close_output(&app);
  true
}
#[tauri::command]
fn nswc_output_state(app: AppHandle) -> OutState {
  out_state(&app)
}
#[tauri::command]
fn nswc_open_external(app: AppHandle, url: String) {
  if url.starts_with("https://") || url.starts_with("http://") {
    let _ = app.opener().open_url(url, None::<&str>);
  }
}

// Tell the page when displays come and go, and drop the output if its display went away
fn watch_displays(app: AppHandle) {
  std::thread::spawn(move || {
    let sig = |a: &AppHandle| {
      a.available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| format!("{:?}{:?}{:?}{}", m.name(), m.position(), m.size(), m.scale_factor()))
        .collect::<Vec<_>>()
    };
    let mut last = sig(&app);
    loop {
      std::thread::sleep(Duration::from_secs(2));
      let now = sig(&app);
      if now == last {
        continue;
      }
      last = now;
      let _ = app.emit_to("main", "nswc:displays-changed", ());
      let pos = app.state::<Output>().0.lock().unwrap().as_ref().map(|x| x.1);
      if let Some(p) = pos {
        if !app.available_monitors().unwrap_or_default().iter().any(|m| *m.position() == p) {
          close_output(&app);
        }
      }
    }
  });
}

/* ---- updates ---- */
fn info(app: &AppHandle, kind: MessageDialogKind, msg: &str, detail: &str) {
  app.dialog().message(detail).title(msg).kind(kind).blocking_show();
}
fn ask(app: &AppHandle, msg: &str, detail: &str, yes: &str) -> bool {
  app.dialog()
    .message(detail)
    .title(msg)
    .buttons(MessageDialogButtons::OkCancelCustom(yes.into(), "Later".into()))
    .blocking_show()
}

fn check_for_updates(app: AppHandle, manual: bool) {
  if CHECKING.swap(true, AtOrd::SeqCst) {
    return;
  }
  std::thread::spawn(move || {
    let shell_failed = match check_shell(&app) {
      Ok(true) => {
        CHECKING.store(false, AtOrd::SeqCst);
        return;
      }
      Ok(false) => false,
      Err(_) => true,
    };
    let page = tauri::async_runtime::block_on(check_page(&app));
    CHECKING.store(false, AtOrd::SeqCst);
    match page {
      Ok(Some(v)) => {
        if ask(&app, &format!("Version {v} is ready"), "Restart the app to start using it. Your projects are kept.", "Restart now") {
          close_output(&app);
          app.restart();
        }
      }
      Ok(None) if manual => {
        let have = PAGE_VERSION.get().cloned().unwrap_or_default();
        info(&app, MessageDialogKind::Info, "You have the latest version", &format!("Version {have}"));
      }
      Err(_) if manual && shell_failed => info(
        &app,
        MessageDialogKind::Warning,
        "Could not check for updates",
        "Check your internet connection and try again. You can keep using this version offline.",
      ),
      _ => {}
    }
  });
}

// A new installer (only published when the Windows shell changes). Ok(true) means it is installing.
fn check_shell(app: &AppHandle) -> Result<bool, Box<dyn std::error::Error>> {
  let Some(update) = tauri::async_runtime::block_on(app.updater()?.check())? else { return Ok(false) };
  if !ask(
    app,
    &format!("Version {} of the Windows app is available", update.version),
    "It installs in a few seconds and the app reopens. Your projects are kept.",
    "Install now",
  ) {
    return Ok(false);
  }
  close_output(app);
  tauri::async_runtime::block_on(update.download_and_install(|_, _| {}, || {}))?;
  app.restart();
}

// A newer page on GitHub Pages: download it into the app data folder. Returns the new version.
async fn check_page(app: &AppHandle) -> Result<Option<String>, Box<dyn std::error::Error>> {
  let http = reqwest::Client::builder().timeout(Duration::from_secs(30)).build()?;
  let get = |f: &str| http.get(format!("{UPDATE_URL}{f}")).header("Cache-Control", "no-cache").send();
  let remote = get("version.txt").await?.error_for_status()?.text().await?.trim().to_string();
  if remote.is_empty() || !remote.chars().all(|c| c.is_ascii_digit() || c == '.') {
    return Err("bad version file".into());
  }
  let have = PAGE_VERSION.get().cloned().unwrap_or_default();
  if cmp_ver(&remote, &have) != Ordering::Greater {
    return Ok(None);
  }
  let html = get("index.html").await?.error_for_status()?.text().await?;
  if !html.contains("NovaStar Wall Calculator") || cmp_ver(&read_ver(&html), &have) != Ordering::Greater {
    return Err("download did not match".into());
  }
  let muxer = get("mp4-muxer.js").await?.error_for_status()?.bytes().await?;
  // write to a fresh folder, then swap it in
  let web = app.path().app_data_dir()?.join("web");
  let next = web.with_file_name("web-next");
  let _ = fs::remove_dir_all(&next);
  fs::create_dir_all(&next)?;
  fs::write(next.join("mp4-muxer.js"), &muxer)?;
  fs::write(next.join("index.html"), &html)?;
  let _ = fs::remove_dir_all(&web);
  fs::rename(&next, &web)?;
  Ok(Some(read_ver(&html)))
}

/* ---- app ---- */
fn menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
  let item = |id: &str, text: &str| MenuItem::with_id(app, id, text, true, None::<&str>);
  Menu::with_items(
    app,
    &[
      &Submenu::with_items(
        app,
        "File",
        true,
        &[&item("print", "Print all screens…\tCtrl+P")?, &PredefinedMenuItem::separator(app)?, &PredefinedMenuItem::quit(app, Some("Exit"))?],
      )?,
      &Submenu::with_items(
        app,
        "View",
        true,
        &[
          &item("stop-output", "Stop live output\tCtrl+Shift+O")?,
          &PredefinedMenuItem::separator(app)?,
          &item("reload", "Reload")?,
          &item("fullscreen", "Toggle full screen")?,
        ],
      )?,
      &Submenu::with_items(app, "Help", true, &[&item("updates", "Check for Updates…")?, &item("about", "About NovaStar Wall Calculator")?])?,
    ],
  )
}

fn create_main(app: &AppHandle) -> tauri::Result<()> {
  let h = app.clone();
  let w = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
    .title("NovaStar Wall Calculator")
    .inner_size(1480.0, 960.0)
    .min_inner_size(900.0, 600.0)
    .background_color(tauri::window::Color(0x0e, 0x12, 0x17, 255))
    .menu(menu(app)?)
    .initialization_script(DESKTOP_JS)
    .on_navigation(move |u| {
      let local = u.scheme() == "tauri" || u.host_str() == Some("tauri.localhost");
      if !local && matches!(u.scheme(), "http" | "https") {
        let _ = h.opener().open_url(u.as_str(), None::<&str>);
      }
      local
    })
    .build()?;
  let h = app.clone();
  w.on_window_event(move |e| {
    if let WindowEvent::CloseRequested { .. } = e {
      close_output(&h);
    }
  });
  Ok(())
}

fn on_menu(app: &AppHandle, id: &str) {
  let main = app.get_webview_window("main");
  match id {
    "print" => {
      if let Some(w) = main {
        let _ = w.eval("document.getElementById('btnPrint').click()");
      }
    }
    "stop-output" => close_output(app),
    "reload" => {
      if let Some(w) = main {
        let _ = w.eval("location.reload()");
      }
    }
    "fullscreen" => {
      if let Some(w) = main {
        let _ = w.set_fullscreen(!w.is_fullscreen().unwrap_or(false));
      }
    }
    "updates" => check_for_updates(app.clone(), true),
    "about" => {
      let (a, p) = (app.clone(), PAGE_VERSION.get().cloned().unwrap_or_default());
      let shell = app.package_info().version.to_string();
      std::thread::spawn(move || {
        info(&a, MessageDialogKind::Info, "NovaStar Wall Calculator", &format!("Version {p}\nWindows app {shell}\n© Reece Burdon"))
      });
    }
    _ => {}
  }
}

fn main() {
  let mut ctx = tauri::generate_context!();
  let bundled = ctx.set_assets(Box::new(NoAssets));
  ctx.set_assets(Box::new(PageAssets(bundled)));
  tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(|app, _, _| {
      if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.set_focus();
      }
    }))
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .manage(Output::default())
    .invoke_handler(tauri::generate_handler![
      nswc_displays,
      nswc_output_open,
      nswc_output_close,
      nswc_output_state,
      nswc_open_external
    ])
    .on_menu_event(|app, e| on_menu(app, e.id().as_ref()))
    .setup(|app| {
      let h = app.handle().clone();
      create_main(&h)?;
      watch_displays(h.clone());
      tauri::async_runtime::spawn(async move {
        tokio_sleep(Duration::from_secs(4)).await;
        check_for_updates(h, false);
      });
      Ok(())
    })
    .run(ctx)
    .expect("error while running the app");
}

async fn tokio_sleep(d: Duration) {
  let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(d)).await;
}
