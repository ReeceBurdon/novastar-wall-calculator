/* NovaStar Wall Calculator service worker: offline app shell. Bump VERSION when files change (build script does this). */
const VERSION='nswc-cd9dddc3';
const SHELL=['./','./index.html','./mp4-muxer.js','./manifest.webmanifest','./icon-192.png','./icon-512.png','./icon-maskable-512.png','./apple-touch-icon.png','./favicon.png'];
self.addEventListener('install',e=>{e.waitUntil(caches.open(VERSION).then(c=>c.addAll(SHELL)).then(()=>self.skipWaiting()))});
self.addEventListener('activate',e=>{e.waitUntil(caches.keys().then(keys=>Promise.all(keys.filter(k=>k!==VERSION&&k!=='nswc-fonts').map(k=>caches.delete(k)))).then(()=>self.clients.claim()))});
self.addEventListener('fetch',e=>{
  const req=e.request;if(req.method!=='GET')return;
  const url=new URL(req.url);
  // Fonts: serve from cache, refresh in the background
  if(url.hostname==='fonts.googleapis.com'||url.hostname==='fonts.gstatic.com'){
    e.respondWith(caches.open('nswc-fonts').then(async c=>{const hit=await c.match(req);const net=fetch(req).then(r=>{c.put(req,r.clone());return r}).catch(()=>hit);return hit||net}));return}
  if(url.origin!==location.origin)return;
  // The page itself: network first so updates arrive, cache when offline
  if(req.mode==='navigate'){
    e.respondWith(fetch(req).then(r=>{const cp=r.clone();caches.open(VERSION).then(c=>c.put('./index.html',cp));return r}).catch(()=>caches.match('./index.html')));return}
  e.respondWith(caches.match(req).then(hit=>hit||fetch(req)));
});
