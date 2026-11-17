# Frontend quality checks

Use Node.js 26+ and npm 12+:

```bash
npm ci
npm run test:browser:install
npm run check
```

On Linux CI, install browser system dependencies too:

```bash
npm exec -- playwright install --with-deps chromium
```

## Test environments

`npm run test:frontend` runs both Vitest projects and enforces their combined
V8 coverage thresholds. `npm test` and `npm run check` also include both projects.

- `unit`: jsdom tests for hooks, state, event handling, and component logic.
- `browser`: `src/**/*.browser.test.{ts,tsx}` tests running in headless Chromium
  through the Playwright provider, with the React Vite plugin and application CSS.

WallpaperGrid tests run in the browser. They use real element dimensions and
ResizeObserver to verify responsive columns, non-overlapping rows, and scrolling
through a virtualized collection. Only native Tauri APIs are mocked; image URLs
use an inline fixture, so these tests do not require Bing or the desktop app.

Browser tests have their own setup and render helper. Do not import the jsdom
setup or helper into browser tests: those replace browser globals and dimensions.

```bash
npm run test:browser
npm run test:frontend -- --project=unit --coverage.enabled=false
npm run test:e2e:web
```

The first two commands are focused runs without coverage enforcement. The full
frontend run enforces coverage across both projects. Existing Playwright E2E tests
remain separate and exercise the application through a mocked Tauri bridge;
neither browser suite replaces native macOS/Windows regression checks.

## Formatting

`npm run format` and `npm run format:check` use Oxfmt. `.oxfmtrc.json` retains
80-column formatting and disables automatic import and package field sorting.
Oxlint continues to handle linting, and markdownlint checks Markdown separately.

## Dependency overrides

The version-scoped overrides in `package.json` keep the current Markdown tooling
on patched transitive dependencies while upstream packages still pin older
versions:

- `markdownlint@0.42.0`: RE2JS 2.10.0 includes the lookbehind matching
  denial-of-service fix.
- `markdownlint-cli@^0.50.0`: minimatch 11.0.9 includes the glob-pattern amplification
  fix. The repository's Markdown file selection and exclusions are checked by
  `npm run lint:md`.
- `micromark-extension-math@3.1.0`: KaTeX 0.19.1 includes the trust-restriction bypass fix.

Revisit these overrides when upgrading the parent packages. Keep the existing
micromark and flat-cache overrides until their affected dependency paths no
longer need them; do not replace the overrides with `--legacy-peer-deps`.
