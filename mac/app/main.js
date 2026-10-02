// NovaStar Wall Calculator: Electron shell around the single-page app.
const {app,BrowserWindow,Menu,shell}=require('electron');
const path=require('path');

if(!app.requestSingleInstanceLock()){app.quit()}
let win=null;
function create(){
  win=new BrowserWindow({width:1480,height:960,minWidth:900,minHeight:600,backgroundColor:'#0e1217',title:'NovaStar Wall Calculator',
    webPreferences:{contextIsolation:true,nodeIntegration:false,sandbox:true,spellcheck:false}});
  win.loadFile(path.join(__dirname,'index.html'));
  // links to websites open in the normal browser, never inside the app
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
