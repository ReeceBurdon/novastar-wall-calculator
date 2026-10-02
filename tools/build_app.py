import re,json,pathlib,hashlib
src=pathlib.Path('src/novastar-wall-calculator.html').read_text()
app=pathlib.Path('docs')
head,body=src.split('</style>',1)
head+='</style>'
head=head.replace('<script src="https://cdn.jsdelivr.net/npm/mp4-muxer@5.2.2/build/mp4-muxer.js"></script>','<script src="mp4-muxer.js"></script>')
assert 'mp4-muxer.js"></script>' in head
links='''<link rel="manifest" href="manifest.webmanifest">
<meta name="theme-color" content="#0e1217">
<meta name="application-name" content="Wall Calc">
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-title" content="Wall Calc">
<link rel="apple-touch-icon" href="apple-touch-icon.png">
<link rel="icon" type="image/png" href="favicon.png">
'''
reg='''
<script>
if('serviceWorker' in navigator&&location.protocol!=='file:'){window.addEventListener('load',()=>{navigator.serviceWorker.register('sw.js').catch(()=>{})})}
</script>'''
html='<!doctype html>\n<html lang="en-GB">\n<head>\n'+head+'\n'+links+'</head>\n<body>'+body+reg+'\n</body>\n</html>\n'
(app/'index.html').write_text(html)
ver=hashlib.sha1(html.encode()).hexdigest()[:8]
(app/'sw.js').write_text('''/* NovaStar Wall Calculator service worker: offline app shell. Bump VERSION when files change (build script does this). */
const VERSION='nswc-%s';
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
'''%ver)
(app/'manifest.webmanifest').write_text(json.dumps({
 "name":"NovaStar Wall Calculator","short_name":"Wall Calc",
 "description":"LED wall data and power routing, wiring plans and test cards for NovaStar processors.",
 "start_url":"./","scope":"./","display":"standalone",
 "background_color":"#0e1217","theme_color":"#0e1217","orientation":"any","categories":["productivity","utilities"],
 "icons":[{"src":"icon-192.png","sizes":"192x192","type":"image/png"},
          {"src":"icon-512.png","sizes":"512x512","type":"image/png"},
          {"src":"icon-maskable-512.png","sizes":"512x512","type":"image/png","purpose":"maskable"}]},indent=2))
print('built',ver)
