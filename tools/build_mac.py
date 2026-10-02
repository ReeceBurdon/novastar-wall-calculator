"""Build the Mac app page from the master source.
Writes mac/app/index.html (bundled in the DMG) and docs/app/ (the copy the installed Mac app downloads to update itself).
Run from the repo root: python3 tools/build_mac.py"""
import pathlib,re,shutil
src=pathlib.Path('src/novastar-wall-calculator.html').read_text()
head,body=src.split('</style>',1);head+='</style>'
head=re.sub(r'<link rel="preconnect"[^>]*>\n?','',head)
head=re.sub(r'<link rel="stylesheet" href="https://fonts.googleapis.com[^>]*>\n?','',head)
head=head.replace('<script src="https://cdn.jsdelivr.net/npm/mp4-muxer@5.2.2/build/mp4-muxer.js"></script>','<script src="mp4-muxer.js"></script>')
faces=[('Barlow',400,'barlow-latin-400'),('Barlow',500,'barlow-latin-500'),('Barlow',600,'barlow-latin-600'),
 ('Barlow Condensed',500,'barlow-condensed-latin-500'),('Barlow Condensed',600,'barlow-condensed-latin-600'),('Barlow Condensed',700,'barlow-condensed-latin-700'),
 ('JetBrains Mono',400,'jetbrains-mono-latin-400'),('JetBrains Mono',600,'jetbrains-mono-latin-600')]
ff='<style>\n'+''.join(f"@font-face{{font-family:'{n}';font-style:normal;font-weight:{w};font-display:swap;src:url(fonts/{f}-normal.woff2) format('woff2')}}\n" for n,w,f in faces)+'</style>\n'
assert 'googleapis' not in head and 'jsdelivr' not in head
html='<!doctype html>\n<html lang="en-GB">\n<head>\n<meta http-equiv="Content-Security-Policy" content="default-src \'self\' \'unsafe-inline\' data: blob:; img-src \'self\' data: blob:; media-src \'self\' blob:">\n'+ff+head+'\n</head>\n<body>'+body+'\n</body>\n</html>\n'
ver=re.search(r'<meta name="app-version" content="([\d.]+)"',html).group(1)
mac=pathlib.Path('mac/app');(mac/'index.html').write_text(html)
web=pathlib.Path('docs/app');web.mkdir(parents=True,exist_ok=True)
(web/'index.html').write_text(html)
shutil.copy(mac/'mp4-muxer.js',web/'mp4-muxer.js')
if (web/'fonts').exists():shutil.rmtree(web/'fonts')
shutil.copytree(mac/'fonts',web/'fonts')
(web/'version.txt').write_text(ver+'\n')
print('mac page built, version',ver)
