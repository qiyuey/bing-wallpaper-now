# Apple HIG App Icon Notes

Source: [Apple HIG App Icons](https://developer.apple.com/design/human-interface-guidelines/app-icons)

If internet access is available and the user asks for the latest Apple guidance,
verify the current official page before making platform-specific claims.

## Design Principles

- Treat the app icon as the app's most compact identity: it should express purpose and personality at a glance.
- Use a simple, memorable central idea instead of multiple competing symbols.
- Prefer a square, opaque master design with a continuous app-tile background.
  For this Tauri project, candidates should visually read as complete rounded
  app tiles rather than floating irregular glyphs.
- Avoid text, letters, screenshots, complex photographic detail, and decorations that disappear at small sizes.
- Use depth, lighting, material, and gradient sparingly. The result should feel polished, not busy.
- Keep enough internal padding for the subject to breathe, but avoid excessive empty space that makes the icon weak at 32px.

## Candidate Prompt Checklist

For generated candidates, include these constraints in the prompt:

- `1024x1024 square app icon`
- `opaque background`
- `single centered motif`
- `modern minimalist software icon`
- `Apple platform friendly`
- `no text, no letters, no watermark`
- `legible at 32px`
- `simple geometry and restrained details`

## Review Rubric

Score each candidate mentally before showing it:

- Recognizable in one glance.
- Strong silhouette or focal shape at 32px.
- Clear relationship to the user's app concept.
- Distinct from common generic app icons.
- Balanced color and contrast in both light and dark desktop contexts.
- No text, accidental letters, watermarks, transparent areas, or crowded micro-detail.
