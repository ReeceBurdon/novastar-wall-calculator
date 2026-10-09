"""Build the Windows app's page from the master source.
Runs tools/build_mac.py (the Windows app uses the same page as the Mac app: local fonts, local MP4 encoder),
then copies docs/app/ into windows/dist/, which Tauri bundles into the installer.
Run from the repo root: python3 tools/build_windows.py"""
import pathlib,shutil,subprocess,sys
subprocess.run([sys.executable,'tools/build_mac.py'],check=True)
dist=pathlib.Path('windows/dist')
if dist.exists():shutil.rmtree(dist)
shutil.copytree('docs/app',dist,ignore=shutil.ignore_patterns('version.txt'))
print('windows page built in',dist)
