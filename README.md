# NovaStar Wall Calculator

LED wall planning tool for NovaStar processors. Covers data and power routing, per-line loading, printable wiring plans and test cards.

- **Screens and projects:** multiple screens per project, multiple saved projects, and a shared panel library.
- **Data:** main and backup processors, auto routing (snake or raster) or hand-drawn custom paths. Shows port load per line and pixel load per processor.
- **Power:** circuits by breaker size and headroom, auto or custom paths, phase balance and minimum supply size.
- **Test cards:**
  - static patterns: engineering grid, colour bars, solid colours
  - moving patterns: scrolling bars, checkerboard, spectrum
  - data and power maps, and wiring-plan cards
  - logo, sweep bars, timecode, frame counter and FPS
- **Output canvases:** place several screens on one raster, Pixl Grid style, for fullscreen output, PNG or MP4 export.
- **Print:** canvas layouts plus every screen's data, backup and power plans.

## Repository layout

| Path | What it is |
|---|---|
| `src/novastar-wall-calculator.html` | **Master source.** One self-contained page; every build is generated from it |
| `docs/` | Installable web app (PWA), served by GitHub Pages |
| `mac/` | Electron shell for the macOS app (`app/main.js`, `app/package.json`, fonts, icon) |
| `tools/build_app.py` | Rebuilds `docs/` from the source |
| `tools/build_mac.py` | Rebuilds `mac/app/index.html` (local fonts, local MP4 encoder) from the source |
| `BUILD.md` | Full build notes, storage keys, release checklist and changelog |

## Using it

- **Web app:** enable GitHub Pages (Settings → Pages → *Deploy from a branch*, `main`, `/docs`). Open the Pages link in Chrome or Edge and click **Install**.
- **Mac app:** download the DMG from **Releases**, drag it into Applications, and on first launch use *System Settings → Privacy & Security → Open Anyway*.

## Making changes

1. Edit `src/novastar-wall-calculator.html`.
2. From the repo root, run `python3 tools/build_app.py`, and `python3 tools/build_mac.py` if you're doing a Mac build.
3. Bump the `VERSION` in `docs/sw.js` (the build script does this automatically) and add a line to the changelog in `BUILD.md`.
4. For a Mac release, follow the packaging steps in `BUILD.md` and attach the DMG to a GitHub Release.

Saved projects live in the browser or app storage under `nsWallCalc.*` keys. Keep those keys stable so updates never lose anyone's work.

Panel and processor figures in the generic presets are starting points. Always check the manufacturer's datasheet before signing off a plan.
