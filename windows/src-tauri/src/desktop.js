// Injected into every window before the page runs. Gives the page the same window.nswcDesktop bridge the
// Mac app's preload.js does (displays, live output), plus the keyboard shortcuts and link handling that
// the Electron menus covered there.
(function(){
  const T=()=>window.__TAURI__;
  const invoke=(cmd,args)=>T().core.invoke(cmd,args);
  const listen=(ev,cb)=>{const go=()=>T()?T().event.listen(ev,e=>cb(e.payload)):setTimeout(go,50);go()};
  window.nswcDesktop={
    displays:()=>invoke('nswc_displays'),
    openOutput:id=>invoke('nswc_output_open',{id:id==null?null:String(id)}),
    closeOutput:()=>invoke('nswc_output_close'),
    outputState:()=>invoke('nswc_output_state'),
    onOutputState:cb=>listen('nswc:output-state',cb),
    onDisplaysChanged:cb=>listen('nswc:displays-changed',()=>cb())
  };
  const output=location.hash==='#output';
  document.addEventListener('keydown',e=>{
    if(output&&e.key==='Escape'){e.preventDefault();invoke('nswc_output_close');return}
    // Ctrl+Shift+O stops the live output from anywhere (View › Stop live output)
    if(e.ctrlKey&&e.shiftKey&&e.key.toLowerCase()==='o'){e.preventDefault();invoke('nswc_output_close');return}
    // Ctrl+P prints every screen (the page's own print layout), not just what's on screen
    if(!output&&e.ctrlKey&&!e.shiftKey&&!e.altKey&&e.key.toLowerCase()==='p'){const b=document.getElementById('btnPrint');if(b){e.preventDefault();b.click()}}
  },true);
  // Web links open in the default browser; the app itself never navigates away
  const ext=u=>{try{const x=new URL(u,location.href);return /^https?:$/.test(x.protocol)&&x.origin!==location.origin?x.href:null}catch(e){return null}};
  document.addEventListener('click',e=>{const a=e.target.closest&&e.target.closest('a[href]');if(!a||a.hasAttribute('download'))return;
    const u=ext(a.getAttribute('href'));if(u){e.preventDefault();invoke('nswc_open_external',{url:u})}},true);
  const open=window.open;
  window.open=function(u,...rest){const x=u&&ext(String(u));if(x){invoke('nswc_open_external',{url:x});return null}return open.call(window,u,...rest)};
})();
