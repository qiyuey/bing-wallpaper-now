# Rust dependency decisions

Reviewed on 2026-09-24.

## Minimum Rust version

The application declares Rust 1.90 in `src-tauri/Cargo.toml`. The locked dependency
graph's highest declared requirement is `mac-usernotifications 0.3.1` (1.90),
followed by `notify-rust 4.18.0` (1.89). `image 0.25.10` requires 1.88.
Edition 2024 alone requires 1.85, so it does not determine this project's minimum.

Check the minimum toolchain locally with:

```bash
cargo +1.90.0 check --manifest-path src-tauri/Cargo.toml --locked --all-targets
```

CI and release builds continue to use stable Rust. The minimum version is a
compatibility declaration, not a pinned development toolchain. Recheck it when
updating dependencies; a newer stable build alone does not verify the minimum.

## Image codecs

`image` disables default features and enables only `jpeg` and `png`:

- Bing wallpaper downloads use JPEG; their temporary files are identified from
  headers before reading dimensions.
- Bundled macOS and Windows tray icons use PNG and are decoded to RGBA.

The resolved image features contain only these two codecs. Compared with the
previous default feature graph, Cargo metadata contains 66 fewer packages,
including AVIF, EXR, TIFF, WebP, and Rayon dependencies. This is a dependency-count
comparison, not a measured binary-size or build-time improvement.

Regression tests cover JPEG header/dimension reading and decoding all three
platform tray PNGs. Other image formats are no longer supported by this dependency
configuration; adding a new input format requires an explicit feature and test.

## Chrono to Jiff assessment

Decision: retain Chrono for now. Jiff is a viable future migration for scheduling
and time-zone handling, but there is no demonstrated correctness or performance
problem requiring a replacement today. No Jiff dependency was added.

The assessment uses Jiff 0.2.37 documentation and the current application code.
Chrono references occur in eight Rust source files, spanning scheduler logic,
runtime state, index serialization, UI timestamps, date keys, and tests.

| Area | Possible Jiff mapping | Compatibility requirement |
| --- | --- | --- |
| UTC index `last_updated` | `Timestamp` with Serde | Read existing v4/v5 RFC 3339 timestamps; retain the existing schema |
| In-memory last update time | `Timestamp`, converted to system zone when needed | Compare local calendar dates, not elapsed 24-hour intervals |
| Runtime timestamp strings | RFC 3339 parsing and explicit formatting | Accept existing offsets and fractional seconds; preserve the persisted format contract |
| Next-day 00:05 scheduler | Civil date plus explicit time-zone disambiguation | Define behavior for skipped and repeated local times |
| UI timestamp and date keys | `strftime` on local zoned time | Preserve `%Y-%m-%d %H:%M:%S` and `%Y%m%d` output |

### Benefits and costs

Jiff exposes time-zone and daylight-saving-aware arithmetic and explicit handling
of ambiguous local times. This fits the scheduler's next-day 00:05 calculation.
However, the existing Chrono implementation already constructs the next calendar
date and handles failed or ambiguous conversions. A migration should specify the
desired edge-case policy rather than assume the present implementation is broken.

Persisted instants should remain RFC 3339 compatible. Serializing Jiff `Zoned`
directly can add a bracketed IANA zone identifier, which must not silently replace
the existing format. Use `Timestamp` or an explicit serialization adapter where
appropriate, and decide whether preserving the original textual offset matters.

On Windows, Jiff normally embeds an IANA time-zone database because the OS lacks
the standard Unix database. Measure its binary-size impact and verify Windows
zone detection. macOS normally uses the system database. System-zone changes
while the app is running also need regression coverage.

The current macOS dependency graph uses Chrono only through this application, so
a complete migration could remove it there. A partial migration would keep both
libraries. No size or speed advantage is established without a prototype.

### Acceptance criteria for a future migration

1. Preserve index v4/v5 and runtime-state read/write compatibility, including
   non-UTC offsets, fractional seconds, invalid values, and future timestamps.
2. Test ordinary midnight, DST gaps/folds, and a skipped calendar date using
   explicit zones and fixed instants, independent of the host's current time.
3. Preserve catch-up intervals, hourly scheduling, retry behavior, and local date
   comparisons after a system time-zone change.
4. Compare macOS/Windows release artifacts and run native notification, startup,
   automatic update, and tray update regression paths.

### Sources

- [Cargo rust-version contract](https://doc.rust-lang.org/cargo/reference/rust-version.html)
- [Image features](https://docs.rs/crate/image/0.25.10/features)
- [Jiff overview and datetime types](https://docs.rs/jiff/0.2.37/jiff/)
- [Jiff platform and time-zone behavior](https://docs.rs/jiff/0.2.37/jiff/_documentation/platform/index.html)
