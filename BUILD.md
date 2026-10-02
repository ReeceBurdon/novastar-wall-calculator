# NovaStar Wall Calculator: source and build notes

`src/novastar-wall-calculator.html` is the **single source of truth**. Every build is generated from it:

| Output | How it is made |
|---|---|
| Claude artifact (preview) | Publish the HTML as-is |
| Standalone file | The HTML as-is (prints and downloads work when not framed) |
| PWA folder | Wrap in a proper `<head>` with manifest + service worker; swap the CDN muxer for local `mp4-muxer.js` |
| Mac DMG (arm64) | Wrap like the PWA but with local fonts (Fontsource woff2) and a CSP; Electron shell below |

Current version: **1.1.0** (2 Oct 2026).

## Mac build pipeline (runs on Linux, no Mac needed)

1. Build `app/index.html`: strip Google Fonts links, add `@font-face` rules pointing at `fonts/*.woff2` (from npm `@fontsource/barlow`, `@fontsource/barlow-condensed`, `@fontsource/jetbrains-mono`, latin 400/500/600, 500/600/700, 400/600), replace the jsDelivr mp4-muxer script with `mp4-muxer.js` (npm `mp4-muxer@5.2.2`, `build/mp4-muxer.js`), add the CSP meta `default-src 'self' 'unsafe-inline' data: blob:`.
2. `app/` also holds `main.js` and `package.json` (below), `mp4-muxer.js`, `fonts/`.
3. Icon: 1024 px PNG drawn with Pillow (dark rounded tile, 4×3 coloured panels, white snake route) saved as `icon.icns`.
4. Package: `npx electron-packager app "NovaStar Wall Calculator" --platform=darwin --arch=arm64 --electron-version=44.5.1 --icon=icon.icns --app-bundle-id=uk.co.reeceburdon.wallcalc --app-version=<version> --app-category-type=public.app-category.productivity --overwrite --out=dist` (`@electron/packager@18`).
5. Ad-hoc sign (needed on Apple Silicon): `rcodesign sign "NovaStar Wall Calculator.app"` (apple-codesign 0.29.0, GitHub releases).
6. DMG: stage the `.app` plus a symlink `Applications -> /Applications`, then `xorrisofs -D -l -V "NovaStar Wall Calculator" -no-pad -r -dir-mode 0755 -o raw.iso stage` and `dmg raw.iso out.dmg` (libdmg-hfsplus from github.com/fanquake/libdmg-hfsplus, built with cmake).
7. The DMG is about 123 MB. Chat uploads are capped, so deliver it to the Mac via the device bridge in 19 MB parts (`split -b 19m`), joined with `cat part-* > file.dmg`.

### main.js

```js
const {app,BrowserWindow,Menu,shell}=require('electron');
const path=require('path');
if(!app.requestSingleInstanceLock()){app.quit()}
let win=null;
function create(){
  win=new BrowserWindow({width:1480,height:960,minWidth:900,minHeight:600,backgroundColor:'#0e1217',title:'NovaStar Wall Calculator',
    webPreferences:{contextIsolation:true,nodeIntegration:false,sandbox:true,spellcheck:false}});
  win.loadFile(path.join(__dirname,'index.html'));
  win.webContents.setWindowOpenHandler(({url})=>{if(/^https?:/i.test(url))shell.openExternal(url);return{action:'deny'}});
  win.webContents.on('will-navigate',(e,url)=>{if(!url.startsWith('file://')){e.preventDefault();if(/^https?:/i.test(url))shell.openExternal(url)}});
  win.on('closed',()=>{win=null});
}
app.on('second-instance',()=>{if(win){if(win.isMinimized())win.restore();win.focus()}});
app.whenReady().then(()=>{
  const isMac=process.platform==='darwin';
  Menu.setApplicationMenu(Menu.buildFromTemplate([
    ...(isMac?[{role:'appMenu'}]:[]),
    {label:'File',submenu:[{label:'Print all screens…',accelerator:'CmdOrCtrl+P',click:()=>win&&win.webContents.executeJavaScript("document.getElementById('btnPrint').click()")},{type:'separator'},isMac?{role:'close'}:{role:'quit'}]},
    {role:'editMenu'},
    {label:'View',submenu:[{role:'reload'},{type:'separator'},{role:'resetZoom'},{role:'zoomIn'},{role:'zoomOut'},{type:'separator'},{role:'togglefullscreen'},{type:'separator'},{role:'toggleDevTools'}]},
    {role:'windowMenu'}
  ]));
  create();
  app.on('activate',()=>{if(!BrowserWindow.getAllWindows().length)create()});
});
app.on('window-all-closed',()=>{if(process.platform!=='darwin')app.quit()});
```

### package.json

```json
{"name":"novastar-wall-calculator","productName":"NovaStar Wall Calculator","version":"1.0.0",
 "description":"LED wall data and power routing, wiring plans and test cards for NovaStar processors.",
 "main":"main.js","author":"Reece Burdon","license":"UNLICENSED"}
```

## Storage keys (localStorage)

`nsWallCalc.projects` (all projects), `nsWallCalc.current`, `nsWallCalc.library` (panel library), `nsWallCalc.logo`, `nsWallCalc.tab`. Older keys `nsWallCalc.v1/v2/v3` are migrated on load. Keep these keys stable so updates never lose saved work.

## Release checklist

1. Edit `novastar-wall-calculator.html`, bump the version here.
2. Syntax-check the script and run it headless (Playwright) for the changed feature.
3. Republish the artifact, rebuild the PWA folder and DMG.
4. Update this Project's copy of the HTML and this file.

## Mac app updates

From 1.1.0 the Mac app updates itself. `tools/build_mac.py` writes `docs/app/` (page, fonts, MP4 muxer, `version.txt`). On launch, and via *Check for Updates…* in the app menu, the app compares `version.txt` with its own `<meta name="app-version">`. If Pages has a newer one, it downloads it into its data folder and offers a restart. Bump the `app-version` meta (and the version shown in the top bar) in the source for every release. A new DMG is only needed when `mac/app/main.js` changes.

## Changelog

- **1.1.0**, 2 Oct 2026: undo/redo for drawn data and power paths (buttons plus ⌘Z / ⇧⌘Z). Added seam & alignment grid, gamma & greyscale ramps, and edge & overscan test patterns. Added heat output in BTU/h per screen and project. The Mac app now updates itself from GitHub Pages. The version is shown in the top bar.

- **1.0.0**, 2 Oct 2026: first release. Covers multi-screen projects, saved projects, panel library, main/backup processors with custom data paths, custom power paths, printable plans, test cards (static, moving and wiring-plan cards), output canvases, MP4 export and the Mac app.
