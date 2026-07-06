# Tauri v2 Icon Pipeline

This repository already has a Tauri icon generation pipeline. Reuse it instead of adding a new image conversion stack.

## Current Source Files

- `src-tauri/icons/icon.svg`: macOS source. The current pattern scales the icon
  to 75% with `translate(4, 4) scale(0.75)` to preserve macOS visual padding.
- `src-tauri/icons/icon-windows.svg`: Windows source. The current pattern uses the full 32x32 canvas.
- `scripts/generate-icons.mjs`: wraps `pnpm tauri icon` and handles Windows/macOS differences.
- `src-tauri/tauri.conf.json`: references generated bundle icons under `icons/`.

## Scratch Work

- Put generated candidates, previews, and intermediate rasters under `target/app-icons/<timestamp>/`.
- Do not write into `src-tauri/icons` until the user confirms a final icon.
- Keep candidate names stable and numbered so the user can choose by number.

## Install Steps

1. Recreate the confirmed design as simple SVG shapes.
2. Write the macOS source to `src-tauri/icons/icon.svg`, preserving the existing 75% padding strategy.
3. Write the Windows source to `src-tauri/icons/icon-windows.svg`, preserving full-canvas sizing.
4. Run:

   ```bash
   pnpm run icons
   ```

5. Verify the files required by `src-tauri/tauri.conf.json`:

   ```bash
   test -f src-tauri/icons/32x32.png
   test -f src-tauri/icons/128x128.png
   test -f src-tauri/icons/128x128@2x.png
   test -f src-tauri/icons/icon.icns
   test -f src-tauri/icons/icon.ico
   ```

6. Inspect `git diff -- src-tauri/icons` and report the changed source and generated assets.

## Notes

- `pnpm run icons` also refreshes `tray-icon-windows.png` from the generated 128x128 PNG.
- Keep `tray-icon-macos@2x.png` unchanged unless the user explicitly asks to redesign tray icons.
- Do not edit `scripts/generate-icons.mjs` or `src-tauri/tauri.conf.json` unless
  the existing pipeline no longer fits the requested platform target.
