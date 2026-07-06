# Apple Icon Composer Integration

Sources:

- [Creating your app icon using Icon Composer](https://developer.apple.com/documentation/Xcode/creating-your-app-icon-using-icon-composer)
- [Icon Composer](https://developer.apple.com/icon-composer/)
- [WWDC25: Create icons with Icon Composer](https://developer.apple.com/videos/play/wwdc2025/361/)

Use this reference only when the user asks for Icon Composer, Liquid Glass,
Apple 26+ appearance modes, or Apple-platform-specific polishing.

## Role in This Tauri Project

Icon Composer is useful as an Apple-platform design and preview stage. It does
not replace the existing Tauri v2 icon pipeline, because this project still
needs static `.icns`, `.ico`, and PNG outputs from `pnpm run icons`.

Use it as an optional branch:

1. Generate and refine 5-candidate rounds as usual.
2. After final confirmation, decompose the approved icon into simple layers.
3. Export numbered layer assets for Icon Composer under
   `target/app-icons/<timestamp>/icon-composer-layers/`.
4. Use Icon Composer to tune Liquid Glass, depth, shadows, dark mode, mono, or
   tinted appearances.
5. Export/render flattened previews for review.
6. Recreate the approved static identity in `src-tauri/icons/icon.svg` and
   `src-tauri/icons/icon-windows.svg`, then run `pnpm run icons`.

Do not make `.icon` the only deliverable for this Tauri desktop app.

## Layer Asset Rules

- Use a 1024x1024 canvas for iPhone, iPad, and Mac-oriented artwork.
- Keep layers flat, simple, and editable. Let Icon Composer add glass, shadow,
  refraction, and specular effects.
- Split layers by Z-depth and color control:
  - background fill or gradient
  - main foreground motif
  - secondary highlight or accent
  - optional depth/detail layer
- Do not export the rounded-rectangle or circular platform mask.
- Prefer SVG for simple vector layers. Use PNG only for raster-only details.
- Number layers in visual stack order so Icon Composer imports them predictably,
  for example `01-background.svg`, `02-main-glyph.svg`, `03-highlight.svg`.

## Local Tooling

On macOS with Xcode installed, Icon Composer may be available at:

```bash
/Applications/Xcode.app/Contents/Applications/Icon\ Composer.app
```

The command-line renderer may be available at:

```bash
/Applications/Xcode.app/Contents/Applications/Icon\ Composer.app/Contents/Executables/ictool
```

Check it with:

```bash
"/Applications/Xcode.app/Contents/Applications/Icon Composer.app/Contents/Executables/ictool" --help
```

Example render command from `ictool --help`:

```bash
"/Applications/Xcode.app/Contents/Applications/Icon Composer.app/Contents/Executables/ictool" \
  input-document.icon \
  --export-image \
  --output-file output.png \
  --platform iOS \
  --rendition Default \
  --width 1024 \
  --height 1024 \
  --scale 2
```

Do not assume `xcrun ictool --help` behaves like the app-bundled executable; in
some Xcode versions it returns an `actool`-style plist error for `--help`.

## When to Use It

Use Icon Composer when:

- The user explicitly asks for it.
- The final icon should be tested in Default, Dark, Mono, Tinted, or Clear modes.
- The design needs Apple Liquid Glass tuning rather than a flat static SVG only.
- Marketing previews for Apple platforms are needed.

Skip Icon Composer when:

- The user only needs a normal Tauri desktop icon update.
- The design is intentionally flat and already works well at 32px and 128px.
- The environment lacks Xcode/Icon Composer and the user did not request manual
  Apple-platform polishing.
