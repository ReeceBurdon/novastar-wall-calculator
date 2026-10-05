# NovaStar Wall Calculator: source and build notes

`src/novastar-wall-calculator.html` is the **single source of truth**. Every build is generated from it:

| Output | How it is made |
|---|---|
| Claude artifact (preview) | Publish the HTML as-is |
| Standalone file | The HTML as-is (prints and downloads work when not framed) |
| PWA folder | Wrap in a proper `<head>` with manifest + service worker; swap the CDN muxer for local `mp4-muxer.js` |
| Mac DMG (arm64) | Wrap like the PWA but with local fonts (Fontsource woff2) and a CSP; Electron shell below |

Current version: **1.3.0** (2 Oct 2026).

## Mac build pipeline (runs on Linux, no Mac needed)

1. Build `app/index.html`: strip Google Fonts links, add `@font-face` rules pointing at `fonts/*.woff2` (from npm `@fontsource/barlow`, `@fontsource/barlow-condensed`, `@fontsource/jetbrains-mono`, latin 400/500/600, 500/600/700, 400/600), replace the jsDelivr mp4-muxer script with `mp4-muxer.js` (npm `mp4-muxer@5.2.2`, `build/mp4-muxer.js`), add the CSP meta `default-src 'self' 'unsafe-inline' data: blob:`.
2. `app/` also holds `main.js`, `preload.js`, `package.json`, `mp4-muxer.js` and `fonts/`. These are in the repo under `mac/app/`, which is the source of truth for them.
3. Icon: 1024 px PNG drawn with Pillow (dark rounded tile, 4×3 coloured panels, white snake route) saved as `icon.icns`.
4. Package: `npx electron-packager app "NovaStar Wall Calculator" --platform=darwin --arch=arm64 --electron-version=44.5.1 --icon=icon.icns --app-bundle-id=uk.co.reeceburdon.wallcalc --app-version=<version> --app-category-type=public.app-category.productivity --overwrite --out=dist` (`@electron/packager@18`).
5. Ad-hoc sign (needed on Apple Silicon): `rcodesign sign "NovaStar Wall Calculator.app"` (apple-codesign 0.29.0, GitHub releases).
6. DMG: stage the `.app` plus a symlink `Applications -> /Applications`, then `xorrisofs -D -l -V "NovaStar Wall Calculator" -no-pad -r -dir-mode 0755 -o raw.iso stage` and `dmg raw.iso out.dmg` (libdmg-hfsplus from github.com/fanquake/libdmg-hfsplus, built with cmake).
7. The DMG is about 123 MB. Chat uploads are capped, so deliver it to the Mac via the device bridge in 19 MB parts (`split -b 19m`), joined with `cat part-* > file.dmg`.

### Shell files

`mac/app/main.js` is the Electron main process. It handles the window, menus, external links, self-update from GitHub Pages and the live output window. `mac/app/preload.js` exposes `window.nswcDesktop` to the page, which lists displays and opens or closes the output.

## Storage keys (localStorage)

`nsWallCalc.output` (live output source, offset and display) and `nsWallCalc.rev` (bumped on every save; the output window watches it).

`nsWallCalc.projects` (all projects), `nsWallCalc.current`, `nsWallCalc.library` (panel library), `nsWallCalc.logo`, `nsWallCalc.tab`. Older keys `nsWallCalc.v1/v2/v3` are migrated on load. Keep these keys stable so updates never lose saved work.

## Release checklist

1. Edit `novastar-wall-calculator.html`, bump the version here.
2. Syntax-check the script and run it headless (Playwright) for the changed feature.
3. Republish the artifact, rebuild the PWA folder and DMG.
4. Update this Project's copy of the HTML and this file.

## Mac app updates

From 1.1.0 the Mac app updates itself. `tools/build_mac.py` writes `docs/app/` (page, fonts, MP4 muxer, `version.txt`). On launch, and via *Check for Updates…* in the app menu, the app compares `version.txt` with its own `<meta name="app-version">`. If Pages has a newer one, it downloads it into its data folder and offers a restart. Bump the `app-version` meta (and the version shown in the top bar) in the source for every release. A new DMG is only needed when `mac/app/main.js` changes.

## Live output

The output is a second window running the same page with `#output`. It never writes storage. It polls `nsWallCalc.rev` every 250 ms and redraws the chosen canvas or test card from the saved project, so any edit in the main window shows up on the output. In the Mac app the window is borderless and fills the chosen display (simple fullscreen on macOS, always on top when it's on a different display from the editor). Esc on it, or *View › Stop live output* (⇧⌘O), closes it. In a browser it opens as a pop-out window placed on the chosen display (via the Window Management API where allowed) and fills it on first click.

## Changelog

- **1.3.0**, 5 Oct 2026: Live output. A bar under the screens sends any canvas or test card to another display in its own window while you keep editing; changes appear on the output immediately. It supports X/Y offset and switching the source while live. The tab buttons are renamed "Fullscreen here". Needs the 1.3.0 Mac app (new DMG) for display selection; older app shells show a prompt to update.

- **1.2.1**, 2 Oct 2026: load bars, line swatches and role pills now print even with background graphics off, and the printout always uses the light palette regardless of dark mode. Summary tiles wrap instead of being cut off on paper.

- **1.2.0**, 2 Oct 2026: processors are now shared across the project. Each screen ticks the processors it runs from, and ports are allocated across all screens: hand-picked ports are reserved first, then screens fill free ports in screen-list order, with backups mirrored. The processor table shows ports and pixel load across all screens and who shares each unit. Added a port map on the Data tab and a project port map page in the printout. Older projects are migrated automatically (each screen's processors become project processors).

- **1.1.0**, 2 Oct 2026: undo/redo for drawn data and power paths (buttons plus ⌘Z / ⇧⌘Z). Added seam & alignment grid, gamma & greyscale ramps, and edge & overscan test patterns. Added heat output in BTU/h per screen and project. The Mac app now updates itself from GitHub Pages. The version is shown in the top bar.

- **1.0.0**, 2 Oct 2026: first release. Covers multi-screen projects, saved projects, panel library, main/backup processors with custom data paths, custom power paths, printable plans, test cards (static, moving and wiring-plan cards), output canvases, MP4 export and the Mac app.
