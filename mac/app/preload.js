// Bridge for the page: lists displays and opens/closes the live output window.
const {contextBridge,ipcRenderer}=require('electron');
contextBridge.exposeInMainWorld('nswcDesktop',{
  displays:()=>ipcRenderer.invoke('nswc:displays'),
  openOutput:id=>ipcRenderer.invoke('nswc:output-open',id),
  closeOutput:()=>ipcRenderer.invoke('nswc:output-close'),
  outputState:()=>ipcRenderer.invoke('nswc:output-state'),
  onOutputState:cb=>ipcRenderer.on('nswc:output-state',(e,v)=>cb(v)),
  onDisplaysChanged:cb=>ipcRenderer.on('nswc:displays-changed',()=>cb())
});
