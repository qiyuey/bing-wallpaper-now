---
name: app-icons
description: "迭代生成并安装软件 App 图标。Use when the user asks to create, generate, refine, redesign, or replace an app icon, software icon, desktop app icon, Tauri icon, Tauri v2 icon, Apple-style icon, or says app-icons, icon design, icon 设计, 软件图标, 应用图标, or Tauri 图标. Creates exactly 5 candidates per design round, supports user-guided refinement loops, and installs only the confirmed final icon into this Tauri v2 project's icon pipeline."
---

# App Icons

## Goal

Design modern, minimalist app icons through an iterative 5-candidate workflow,
then install the confirmed final design into this Tauri v2 project.

## Required References

- Read `references/icon-design-principles.md` before generating any icon candidates.
- Read `references/apple-hig-app-icons.md` before generating or judging icon concepts.
- Read `references/tauri-v2-icon-pipeline.md` before installing a final icon into `src-tauri/icons`.
- Read `references/icon-composer.md` when the user asks for Icon Composer,
  Liquid Glass, Apple 26+ appearances, or Apple-platform-specific polishing.

## Workflow

### 1. Ground the Brief

- Inspect the app context from `package.json`, `src-tauri/tauri.conf.json`, and
  the current source icons when working in this repo.
- Use the user's icon content as the primary brief. If the brief lacks a core
  subject or metaphor, ask one concise question before generating.
- Translate the app into one icon thesis before prompting:
  `<app purpose> -> <user-recognizable metaphor> -> <single glyph shape>`.
- Default style: Apple-first modern app icon, icon-first, one clear metaphor,
  refined material/depth, restrained details, no text.
- Treat the app identity as one family first. For open-ended first rounds,
  vary complexity levels inside the same identity instead of making five
  unrelated feature-thesis combinations.
- Create a scratch directory under `target/app-icons/<timestamp>/` for candidates
  and previews. Do not overwrite `src-tauri/icons` before the user confirms the
  final design.

### 2. Generate 5 Candidates

- Every design round must contain exactly 5 candidates.
- Generate 1024x1024 square, opaque, no-text candidates with one recognizable
  central idea and a reasonable app-icon complexity budget.
- Every candidate must read as a complete rounded-rectangle app tile with a
  continuous outer backplate/background. Do not show a floating irregular glyph
  or cutout silhouette as the whole icon.
- Prompt for Apple-style app icons, not SF Symbol-like control icons or illustrations: use
  words like `Apple-style product app icon`, `one primary symbol`, `strong
  silhouette`, `refined material`, `subtle depth`, and `small-size legibility`.
  Avoid `SF Symbol-like icon`, `single-color control glyph`, `flat-only`,
  `concept art`, `scene`, `photorealistic`, `highly detailed`, and broad
  decorative mood language.
- Make the 5 candidates meaningfully different by complexity level, material
  treatment, and symbol refinement within the same identity. Do not make them
  five unrelated app-feature metaphors unless the user explicitly asks for
  broad concept exploration.
- Save candidates with stable names such as `round-01-candidate-01.png` through
  `round-01-candidate-05.png`.
- Create a quick thumbnail proof for each candidate at 32px and 128px before
  showing it. Reject and regenerate any candidate that reads as an illustration,
  abstract decoration, or vague wallpaper tile at those sizes.
- Show the 5 candidate images with absolute paths, then summarize each candidate
  in one short line covering concept, strength, and risk.
- Ask the user to choose one candidate number for refinement, request a fresh
  5-candidate round, or confirm a final only if they explicitly say the current
  candidate is final.

### 3. Refine the Chosen Direction

- When the user chooses a candidate, generate the next round as exactly 5 variants of that chosen direction.
- Preserve the selected visual identity while varying controlled axes:
  composition, color palette, depth, motif simplification, and small-size clarity.
- Keep each round in a new numbered set such as `round-02-variant-01.png` through `round-02-variant-05.png`.
- Repeat: show 5 variants, ask the user to select one, then refine again.
- Continue until the user explicitly confirms the final design with language like "终稿", "确认", "安装这个", "use this", or "final".

### 4. Install the Final Icon

- Before installation, read `references/tauri-v2-icon-pipeline.md`.
- If the user asks to use Icon Composer, or the final concept benefits from
  Apple-specific Liquid Glass previewing, read `references/icon-composer.md`
  and prepare layered source assets before writing the Tauri source SVGs.
- Convert or redraw the confirmed final into the project's source SVGs:
  - `src-tauri/icons/icon.svg`: macOS source with the existing 75% padding strategy.
  - `src-tauri/icons/icon-windows.svg`: Windows source using the full canvas.
- Prefer clean geometric SVG reconstruction over noisy raster tracing. If the
  generated design is too complex for SVG, simplify it while preserving the
  approved identity.
- Run `pnpm run icons` from the repo root.
- Verify that all icon files referenced by `src-tauri/tauri.conf.json` exist after generation.
- Report changed files and any visual tradeoffs made during SVG reconstruction.

## Quality Gates

- Do not install before explicit final confirmation.
- Do not produce a one-off single candidate during design rounds.
- Do not show candidates that fail the icon-likeness gate in
  `references/icon-design-principles.md`.
- Avoid text, letters, badges, tiny details, complex photos, transparent
  backgrounds, copied third-party logos, purely decorative abstraction,
  SF Symbol-like control styling, multi-feature concept piles, floating
  irregular icon silhouettes, and disconnected feature-thesis mashups.
- Optimize for recognition at 32px and 128px, not only for the 1024px master.
- Keep the final icon visually simple enough to survive Tauri's generated `.icns`, `.ico`, and PNG outputs.
