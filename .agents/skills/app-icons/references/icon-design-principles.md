# Icon Design Principles

Apple sources:

- [Apple HIG: App icons](https://developer.apple.com/design/human-interface-guidelines/app-icons)
- [Apple WWDC25: Say hello to the new look of app icons](https://developer.apple.com/videos/play/wwdc2025/220/)
- [Apple WWDC25: Create icons with Icon Composer](https://developer.apple.com/videos/play/wwdc2025/361/)
- [Apple Xcode: Creating your app icon using Icon Composer](https://developer.apple.com/documentation/Xcode/creating-your-app-icon-using-icon-composer)

Historical but still useful Apple principle reference:

- [Apple WWDC17: App Icon Design](https://developer.apple.com/videos/play/wwdc2017/822/)

Use this file before creating candidate prompts or judging generated results.
Do not import non-Apple design-system rules into app icon direction. Add extra
guidance only when it supports the Apple app icon goal.

## Non-Negotiable Direction

An icon is a recognizable visual symbol. An app icon is the face of the app:
it must express purpose and personality at a glance, not merely decorate a
rounded rectangle.

Apple's guidance is the primary source:

- An effective icon expresses a single concept in a way people instantly
  understand.
- A unique, memorable app icon expresses the app's purpose and personality and
  helps people recognize it at a glance.
- The icon must be simple, meaningful, beautiful, and immediately recognizable.

Extra working rule for this skill:

- The rounded app tile is the container. The internal symbol is the icon.
- If the internal symbol cannot be named in 1-3 ordinary words without reading
  the prompt, regenerate it.
- If a candidate is only a beautiful material tile, glow, fold, gradient, or
  abstract surface, it is not yet an icon.

Before generating images, write a one-line icon thesis:

```text
<app purpose> -> <recognizable metaphor> -> <single glyph shape>
```

Bad thesis:

```text
daily wallpaper app -> beautiful abstract wallpaper mood -> colorful polished scene
```

Good thesis:

```text
daily wallpaper app -> fresh desktop each day -> calendar-page desktop glyph
```

## Design Principles

- Use one primary metaphor. A secondary detail is allowed only if it clarifies
  the first metaphor.
- Prefer recognizable object/action metaphors over abstract material effects:
  page, screen, aperture, calendar tile, stack, switch, brush, frame, or refresh
  surface.
- Keep first-round exploration inside one coherent product identity unless the
  user asks for unrelated concept directions.
- Use a reasonable Apple app-icon complexity budget: usually 2-4 layers and
  3-6 major shapes, plus at most one accent. This is enough for Apple-style
  material and depth without becoming a feature pile.
- Every candidate must have a complete rounded-rectangle app tile/backplate.
  The internal symbol can be abstract, but the outer icon must not be a floating
  irregular shape.
- Make the silhouette bold enough to recognize as a black shape.
- Prefer simple geometric construction: circle, square, rectangle, arc, fold,
  aperture, page, panel, or check mark.
- Use high contrast and a limited palette. Color should reinforce identity, not
  carry the entire meaning.
- Design for 32px first, then enrich for 1024px. Details that vanish at 32px are
  decoration, not icon content.
- Keep the mark front-facing and stable. Avoid dramatic perspective unless it is
  essential to the metaphor.
- Use Apple-like depth and material deliberately. Subtle bevels, layered
  surfaces, soft shadows, and simple translucency are acceptable; rendered
  objects, scenes, and complex glass sculptures are not.
- Treat platform-specific effects, including Liquid Glass, tinting, dark
  variants, and clear appearances, as finish layers over a strong base glyph.
  Do not let the effect become the icon's only identity.
- Keep visual mass centered and balanced. Leave clear internal negative space.
- Use a grid or key-shape mindset: consistent stroke widths, repeated radii, and
  aligned edges.
- Do not make an SF Symbol-like control icon. A good app icon can be symbolic
  and simple while still having presence, material, and a designed background.
- Make every candidate nameable in five words or fewer, such as `calendar
  desktop`, `refresh page`, or `adaptive screens`.

## Recognition Tests

Run these tests before showing candidates:

- Name test: can the symbol be named in 1-3 ordinary words?
- Silhouette test: does the main idea remain visible as a black shape?
- Thumbnail test: does it read at 32px and 128px?
- No-prompt test: would a user see a symbol, not just a decorative tile?
- App-fit test: does the symbol plausibly connect to fresh desktop wallpaper?

Reject candidates that fail the name test or no-prompt test, even if they look
polished.

## Icon-Likeness Gate

Reject and regenerate a candidate if any item is true:

- It looks like a wallpaper, illustration, splash screen, badge, or app preview.
- It lacks a complete app-tile backplate and reads as a standalone irregular
  glyph on empty space.
- It is only a rounded rectangle with glow, fold, gradient, or abstract material
  and no independently recognizable symbol.
- The internal symbol cannot be named in 1-3 ordinary words.
- It contains a scene, horizon, landscape, tiny UI, thumbnail collage, or
  unrelated feature pile.
- It needs the app name to explain what it is.
- It relies on fine texture, small glints, complex translucency, or many small
  overlapping details.
- It has no strong silhouette when imagined as a black shape.
- It becomes an indistinct blob at 32px.
- It looks like a generic AI-generated 3D object, SF Symbol-like control icon,
  or single-color control glyph instead of an Apple-style product app icon.

## Prompt Pattern

Use this pattern for each candidate:

```text
Create a 1024x1024 Apple-style product app icon, not an illustration and not an
SF Symbol-like control icon.
One metaphor: <metaphor>.
Primary symbol: <shape>.
The internal symbol must be nameable in 1-3 ordinary words and recognizable
without reading the app name.
Style: Apple-platform-friendly app icon with a strong silhouette, centered
composition, refined material, subtle depth, high contrast, and opaque square
background.
Outer form: complete rounded-rectangle app tile/backplate; no floating
irregular icon silhouette.
Small-size requirement: must remain recognizable at 32px and 128px.
Detail budget: one primary symbol, 2-4 layers, 3-6 major shapes, at most one
accent, no unrelated secondary metaphor.
Avoid: text, letters, scene, landscape, photo, UI screenshot, tiny details,
decorative abstract sculpture, complex 3D render, SF Symbol-like control icon,
flat-only system icon, watermark, platform logos, third-party logos,
rounded-corner mask baked into the artwork.
```

## Candidate Diversity

For an open-ended first round, prefer one product identity explored across
complexity levels:

- Candidate A: minimal tile, one strong symbol, lowest detail.
- Candidate B: simple material tile, clearer depth and one accent.
- Candidate C: balanced Apple app icon, 2-3 layers and polished lighting.
- Candidate D: richer Apple material treatment, still one metaphor.
- Candidate E: boldest composition, strongest personality, still small-size safe.

Use unrelated metaphor exploration only when the user explicitly asks for broad
concept directions. If doing that, keep each candidate Apple-first and avoid
combining multiple product features into one icon:

- Candidate A: time/daily cadence.
- Candidate B: desktop/apply action.
- Candidate C: refresh/sync.
- Candidate D: multi-display orientation.
- Candidate E: local/private cache or saved wallpaper.

For refinement rounds, keep the chosen metaphor fixed and vary only controlled
axes: silhouette, live area, color, stroke/mass, and one accent detail.
