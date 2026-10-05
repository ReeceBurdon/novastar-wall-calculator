// NovaStar Wall Calculator: Electron shell around the single-page app.
// Updates: the page itself is downloaded from GitHub Pages (docs/app/) when a newer version is published,
// stored in the app's data folder and used from then on. Everything runs from local files, so it works
// offline and saved projects (localStorage on file://) are shared by every version.
// Live output (1.3.0): preload.js exposes window.nswcDesktop to open a borderless output window on any display.
const {app,BrowserWindow,Menu,shell,net,dialog,ipcMain,screen}=require('electron');
const path=require('path'),fs=require('fs');

const UPDATE_URL='https://reeceburdon.github.io/novastar-wall-calculator/app/';
const bundledDir=__dirname;
const webDir=()=>path.join(app.getPath('userData'),'web');

const readVer=file=>{try{const m=/<meta name="app-version" content="([\d.]+)"/.exec(fs.readFileSync(file,'utf8'));return m?m[1]:'0'}catch(e){return '0'}};
const cmp=(a,b)=>{const x=a.split('.').map(Number),y=b.split('.').map(Number);for(let i=0;i<Math.max(x.length,y.length);i++){const d=(x[i]||0)-(y[i]||0);if(d)return d}return 0};
function entry(){const dl=path.join(webDir(),'index.html'),bv=readVer(path.join(bundledDir,'index.html'));
  return cmp(readVer(dl),bv)>0?dl:path.join(bundledDir,'index.html')}
const runningVersion=()=>readVer(entry());

if(!app.requestSingleInstanceLock()){app.quit()}
let win=null,checking=false;

/* ---- live output: a second, borderless window filling the chosen display; the page runs in #output mode ---- */
let outWin=null,outLabel='';
function displaysList(){const pri=screen.getPrimaryDisplay().id,mainD=win?screen.getDisplayMatching(win.getBounds()).id:pri;
  return screen.getAllDisplays().map((d,i)=>({id:d.id,label:d.label||(d.internal?'Built-in display':'Display '+(i+1)),
    w:Math.round(d.size.width*d.scaleFactor),h:Math.round(d.size.height*d.scaleFactor),scale:d.scaleFactor,primary:d.id===pri,main:d.id===mainD,internal:!!d.internal}))}
const outState=()=>outWin?{open:true,label:outLabel}:{open:false};
function sendState(){if(win&&!win.isDestroyed())win.webContents.send('nswc:output-state',outState())}
function closeOutput(){if(outWin){const w=outWin;outWin=null;w.removeAllListeners('closed');w.destroy();sendState()}}
function openOutput(id){
  const d=screen.getAllDisplays().find(x=>x.id===id)||screen.getAllDisplays().find(x=>!win||x.id!==screen.getDisplayMatching(win.getBounds()).id)||screen.getPrimaryDisplay();
  const b=d.bounds,info=displaysList().find(x=>x.id===d.id)||{label:'display',w:b.width,h:b.height};
  const onMain=win&&screen.getDisplayMatching(win.getBounds()).id===d.id;
  closeOutput();outLabel=`${info.label} (${info.w}×${info.h})`;
  outWin=new BrowserWindow({x:b.x,y:b.y,width:b.width,height:b.height,frame:false,show:false,backgroundColor:'#000000',title:'Live output',
    hasShadow:false,enableLargerThanScreen:true,fullscreenable:true,skipTaskbar:true,
    webPreferences:{contextIsolation:true,nodeIntegration:false,sandbox:true,spellcheck:false,backgroundThrottling:false,preload:path.join(bundledDir,'preload.js')}});
  outWin.loadFile(entry(),{hash:'output'});
  outWin.once('ready-to-show',()=>{if(!outWin)return;outWin.setBounds(b);outWin.showInactive();
    if(process.platform==='darwin')outWin.setSimpleFullScreen(true);else outWin.setFullScreen(true);
    if(!onMain)outWin.setAlwaysOnTop(true,'screen-saver');
    setTimeout(()=>{if(win&&!win.isDestroyed()&&!onMain)win.focus()},400)});
  outWin.webContents.on('before-input-event',(e,i)=>{if(i.type==='keyDown'&&i.key==='Escape'){e.preventDefault();closeOutput()}});
  outWin.webContents.setWindowOpenHandler(()=>({action:'deny'}));
  outWin.webContents.on('will-navigate',e=>e.preventDefault());
  outWin.on('closed',()=>{outWin=null;sendState()});
  sendState();return{ok:true,label:outLabel}}
ipcMain.handle('nswc:displays',()=>displaysList());
ipcMain.handle('nswc:output-open',(e,id)=>{try{return openOutput(id)}catch(err){return{ok:false,error:String(err&&err.message||err)}}});
ipcMain.handle('nswc:output-close',()=>{closeOutput();return true});
ipcMain.handle('nswc:output-state',()=>outState());

function create(){
  win=new BrowserWindow({width:1480,height:960,minWidth:900,minHeight:600,backgroundColor:'#0e1217',title:'NovaStar Wall Calculator',
    webPreferences:{contextIsolation:true,nodeIntegration:false,sandbox:true,spellcheck:false,backgroundThrottling:false,preload:path.join(bundledDir,'preload.js')}});
  win.loadFile(entry());
  win.webContents.setWindowOpenHandler(({url})=>{if(/^https?:/i.test(url))shell.openExternal(url);return{action:'deny'}});
  win.webContents.on('will-navigate',(e,url)=>{if(!url.startsWith('file://')){e.preventDefault();if(/^https?:/i.test(url))shell.openExternal(url)}});
  win.on('closed',()=>{win=null;closeOutput()});
}

async function getText(url){const r=await net.fetch(url,{cache:'no-store'});if(!r.ok)throw new Error(r.status+' '+url);return r.text()}
async function getBuf(url){const r=await net.fetch(url,{cache:'no-store'});if(!r.ok)throw new Error(r.status+' '+url);return Buffer.from(await r.arrayBuffer())}

async function checkForUpdate(manual){
  if(checking)return;checking=true;
  try{
    const remote=(await getText(UPDATE_URL+'version.txt')).trim(),have=runningVersion();
    if(!/^\d+(\.\d+)*$/.test(remote))throw new Error('bad version file');
    if(cmp(remote,have)<=0){if(manual&&win)dialog.showMessageBox(win,{type:'info',message:'You have the latest version',detail:`Version ${have}`});return}
    const html=await getText(UPDATE_URL+'index.html');
    if(!html.includes('NovaStar Wall Calculator')||cmp(/<meta name="app-version" content="([\d.]+)"/.exec(html)?.[1]||'0',have)<=0)throw new Error('download did not match');
    const muxer=await getBuf(UPDATE_URL+'mp4-muxer.js');
    // write to a fresh folder, then swap it in
    const next=webDir()+'-next';fs.rmSync(next,{recursive:true,force:true});fs.mkdirSync(next,{recursive:true});
    fs.cpSync(path.join(bundledDir,'fonts'),path.join(next,'fonts'),{recursive:true});
    fs.writeFileSync(path.join(next,'mp4-muxer.js'),muxer);
    fs.writeFileSync(path.join(next,'index.html'),html);
    fs.rmSync(webDir(),{recursive:true,force:true});fs.renameSync(next,webDir());
    if(win){const {response}=await dialog.showMessageBox(win,{type:'info',buttons:['Restart now','Later'],defaultId:0,cancelId:1,
      message:`Version ${remote} is ready`,detail:'Restart the app to start using it. Your projects are kept.'});
      if(response===0){closeOutput();app.relaunch();app.exit(0)}}
  }catch(e){
    if(manual&&win)dialog.showMessageBox(win,{type:'warning',message:'Could not check for updates',detail:'Check your internet connection and try again. You can keep using this version offline.'});
  }finally{checking=false}
}

app.on('second-instance',()=>{if(win){if(win.isMinimized())win.restore();win.focus()}});
app.whenReady().then(()=>{
  const isMac=process.platform==='darwin';
  const page=js=>win&&win.webContents.executeJavaScript(js);
  Menu.setApplicationMenu(Menu.buildFromTemplate([
    ...(isMac?[{label:app.name,submenu:[{role:'about'},{label:'Check for Updates…',click:()=>checkForUpdate(true)},{type:'separator'},{role:'services'},{type:'separator'},{role:'hide'},{role:'hideOthers'},{role:'unhide'},{type:'separator'},{role:'quit'}]}]:[]),
    {label:'File',submenu:[{label:'Print all screens…',accelerator:'CmdOrCtrl+P',click:()=>page("document.getElementById('btnPrint').click()")},{type:'separator'},isMac?{role:'close'}:{role:'quit'}]},
    {label:'Edit',submenu:[
      // Undo/Redo go to the wiring-plan history when drawing; otherwise to normal text undo
      {label:'Undo',accelerator:'CmdOrCtrl+Z',click:async()=>{if(!win)return;const done=await page('window.nsUndo?window.nsUndo():false').catch(()=>false);if(!done)win.webContents.undo()}},
      {label:'Redo',accelerator:'Shift+CmdOrCtrl+Z',click:async()=>{if(!win)return;const done=await page('window.nsRedo?window.nsRedo():false').catch(()=>false);if(!done)win.webContents.redo()}},
      {type:'separator'},{role:'cut'},{role:'copy'},{role:'paste'},{role:'selectAll'}]},
    {label:'View',submenu:[{label:'Stop live output',accelerator:'CmdOrCtrl+Shift+O',click:()=>closeOutput()},{type:'separator'},{role:'reload'},{type:'separator'},{role:'resetZoom'},{role:'zoomIn'},{role:'zoomOut'},{type:'separator'},{role:'togglefullscreen'},{type:'separator'},{role:'toggleDevTools'}]},
    {role:'windowMenu'},
    ...(isMac?[]:[{label:'Help',submenu:[{label:'Check for Updates…',click:()=>checkForUpdate(true)}]}])
  ]));
  app.setAboutPanelOptions({applicationName:'NovaStar Wall Calculator',applicationVersion:runningVersion(),version:'',copyright:'Reece Burdon'});
  create();
  const changed=()=>{if(win&&!win.isDestroyed())win.webContents.send('nswc:displays-changed');
    if(outWin){const b=outWin.getBounds();if(!screen.getAllDisplays().some(d=>d.bounds.x===b.x&&d.bounds.y===b.y))closeOutput()}};
  screen.on('display-added',changed);screen.on('display-removed',changed);screen.on('display-metrics-changed',changed);
  setTimeout(()=>checkForUpdate(false),4000);
  app.on('activate',()=>{if(!BrowserWindow.getAllWindows().length)create()});
});
app.on('window-all-closed',()=>{if(process.platform!=='darwin')app.quit()});
