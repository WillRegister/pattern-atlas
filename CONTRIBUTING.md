# Feedback and contributions

Pattern Atlas is a native Rust research workbench in active development. Advice, bug reports, interface suggestions and small focused contributions are welcome. Open a GitHub issue for discussion, or submit a pull request with a short explanation and relevant validation.

## Useful review questions

- Does the entity/date/location/session model remain understandable as datasets grow? How should keyed updates, deduplication and relationships between multiple entities evolve?
- Are the selected-versus-all date and location scopes clear? Can the temporal and comparison panels make large sets easier to explore?
- Are calendar, anniversary, timezone, coordinate and rounding conventions clearly represented and correctly calculated?
- Where should computation move out of immediate UI rendering as datasets and pair counts grow? Which indexes or caches would help first?
- Is the separation between `atlas-core`, `atlas-storage`, `atlas-astro` and the desktop interface suitable for adding a web/API interface later?
- Which accessibility and keyboard-navigation improvements would make the workbench easier to use?

Include the app version, operating system, steps to reproduce and expected versus actual behavior for bug reports. A small synthetic JSON example is especially helpful. Share a minimal example rather than a personal database or full workspace.

## Development

The current supported and tested target is Windows with stable Rust/MSVC and Visual Studio C++ build tools. See the README for prerequisites. From the checkout root:

```powershell
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p pattern-atlas --locked -- --data-dir .\data
```

`scripts/build.ps1` runs those checks, builds an optimized executable and copies canonical/versioned executables into `dist`. Normal tests run offline; the live geocoding check is explicitly ignored. Astronomy tests use bundled JPL reference fixtures. Existing UI event tests cover editing, undo, selection and calculation refresh.

Keep changes focused. Describe user-visible behavior, why it changes and how you checked it. For calculation changes, include independent reference values or meaningful boundary cases. For interface changes, a screenshot with synthetic data helps review. Preserve workspace migrations and Undo/Redo behavior when changing saved state.

## Project layout

- `crates/core`: reusable model, filters, imports, gematria, dates, geography and feature matching.
- `crates/storage`: SQLite workspace persistence and saved revisions.
- `crates/astro`: bundled Astronomy Engine C library, checked Rust interface and reference tests.
- `crates/desktop`: egui/eframe interface and CLI commands.
- `docs`, `examples`, `datasets`: import format, fictional examples and the sourced NFL dataset.
- `PLAN.md`, `GUIDE-*.md`: development direction and feature workflows.

`data`, `dist`, `target`, local QA output and credentials are excluded from Git. `Cargo.lock` is tracked for reproducible application builds. Keep third-party notices with their bundled code; see `THIRD_PARTY.md`. The application source is MIT licensed as declared in `Cargo.toml`; third-party components and source datasets retain their own terms.
