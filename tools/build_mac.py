import pathlib,re
src=pathlib.Path('src/novastar-wall-calculator.html').read_text()
out=pathlib.Path('mac/app')
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
(out/'index.html').write_text(html);print('ok',len(html))
