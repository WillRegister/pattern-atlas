# Pattern Atlas — project plan

Working name; a local workspace for exploring numerical properties of named entities, events, language, places and the sky.

## Architecture decision

Build a Windows executable with egui/eframe. Keep a pure Rust `atlas-core` crate for models and calculations, an `atlas-storage` SQLite adapter, and thin interface adapters. The desktop app, CLI and eventual local web API must call the same engine. WSL is an optional Linux development target; a shell script starts a compiled program but does not replace compiling it. The Windows app requires neither WSL nor a browser. Use a pinned, toolchain-compatible GUI release and a checked-in Cargo.lock.

## Data and interaction model

- A workspace owns datasets and research sessions.
- A dataset owns entities (people, organizations, phrases, events), places and labeled dated observations. UUIDs identify records; display names may change.
- Entity records carry aliases, context, tags and source notes. A dated observation references its entity and optionally its place. Birth, founding, publication and event dates retain different labels.
- Date precision is explicit: date only, or local date/time plus IANA timezone. Unknown time remains unknown. A session may assume local noon, midnight or a custom time for sky exploration; this does not alter the source observation. Ambiguous/nonexistent DST times need resolution rather than silent guessing.
- A session references a dataset and saves selected entities, active place, active date/time, number query, enabled ciphers, units, rounding and temporal display controls. Switching sessions switches the lens, rather than duplicating the dataset.
- Numeric features retain owner, domain, calculation, unit, raw value and display transformation. Search and comparisons use the same feature collection. A displayed rounded match remains inspectable as a rounded match. Zero collisions are opt-in. Signed longitude and shortest angular separation are separate properties.
- Initial persistence: versioned workspace JSON documents inside SQLite transactions, optimistic revision checks and revision history. This makes the first schema easy to evolve. Indexed record tables, dataset merging, selection graphs, FTS context search and read-only API queries come after the model settles. JSON is the interchange format, not the only storage interface.

## First build acceptance criteria

1. Native window can create/edit entities, context, places and labeled dates; sessions reuse a dataset.
2. Save/reopen and JSON export/import preserve IDs, relationships, unknown times and settings; failed validation never overwrites saved data.
3. Toggle a starting cipher registry with per-letter explanations; searching 56 finds Will's ordinal total and an illustrative 56-degree longitude under the current transformation.
4. Show elapsed days, calendar years/months/weeks/days, pairwise date spans and days to annual recurrence. Calendar months are not fixed 30-day periods. End inclusion and leap-day policies are explicit.
5. Show spherical distance in km/mi/m/ft, raw latitude/longitude, signed coordinate deltas, shortest longitude separation and central angle. Test dateline and identical points.
6. Show shared values in a domain matrix with every contributing feature accessible.
7. Build a runnable Windows executable and provide PowerShell launch/build commands.

## Build in focused passes

### 1. Data backbone and native workbench — current

Implement the first acceptance criteria, a coordinate plotting surface and a documented future sky panel. Tests cover calendar boundaries, coordinate conventions, numeric transformations, validation and storage conflicts. Example dates/coordinates are labeled illustrative, not biographical assertions.

### 2. Earth and sky — next

Bundle a licensed offline world outline with latitude/longitude grid; add point selection, pan/zoom, arcs, globe view and solar day/night shading. Add a tested ephemeris backend (evaluate Astronomy Engine's MIT-licensed C implementation behind a small safe Rust adapter against an independently generated reference fixture). Require coordinates, elevation, time precision and timezone. Render horizon sky wheel with N/E/S/W, altitude rings, Sun, Moon and planets; distinguish below-horizon bodies from bodies overhead. Add an ecliptic wheel as a separate coordinate system. Active location/time drives both panels. Include UTC/local displays, now/live mode, event stepping, sliders, sunrise/set, lunar phase, angular separations, and a documented accuracy/range/refraction policy. Optional IP lookup returns an approximate suggestion that the user can replace. Do not infer precise event times from a date alone.

### 3. Temporal and cipher completeness

Use the public Gematrinator calculator as a behavior reference: independent year/month/week/day toggles, duration variations, include end date, date +/- duration, history and date-number breakdowns. Build an inventory of ciphers, verify each alphabet mapping with fixtures, add case-sensitive, exception, Latin, Greek, Hebrew and custom tables. Current first-build ciphers are a subset; parity is not claimed. Add word/phrase highlighting, name variants and saved cipher presets.

### 4. Pattern workspace

Add facets for entities/tags/events/source quality/domain/unit; scalable pair matrices; saved numeric queries; pinning; side-by-side sessions; precision sweeps and assumption sensitivity. Add full context search and linked notes. Inspect matches with formulas and source records, and distinguish same-domain and cross-domain collisions. Keep measurements separate from interpretations. A match is a shared numerical property, not an inferred causal relationship.

### 5. Multiple ways to access the same data

CLI JSON import/export/analyze ships with the first desktop binary. Add a local read-only HTTP API and web client against the same storage revision protocol; then authenticated write routes. WASM can run the pure engine with a browser storage adapter. Package portable Windows release and later installer; build Linux separately under WSL. Add explicit export bundles with dataset, session settings and calculation versions. Import the old Nexus JSON through a dedicated mapping adapter, preserving the old files.

## Layout direction

Left: datasets, sessions and entity selection. Top: global number search, active date/time/location and transformation controls. Main tabs: Overview, Records, Gematria, Dates, Earth, Sky and Comparison. The same entity/number selection should eventually highlight across every panel. Recompute when data/settings change, not on every animation frame. Use a searchable feature index as datasets grow.

## Reference material

- `D:\ag393\nexus-rust` and `D:\ag393\matrix` remain inspiration and migration inputs.
- https://docs.rs/eframe/0.33.3/eframe/ (native and WASM rendering)
- https://gematrinator.com/date-calculator (date UI behavior)
- https://gematrinator.com/ciphers (cipher inventory)
- https://github.com/cosinekitty/astronomy (ephemeris candidate)

## Next review

### Editing pass completed (0.2)

Alias/tag/session/entity/place removal controls and workspace-wide undo/redo are implemented. Deletions clean dependent references and can be reversed. Undo retains 100 steps during a run and groups typing; saving preserves the undo timeline. The last 50 saved revisions are available in Overview for recovery after restart. Retained datasets remain accessible from the new-session dataset picker. The existing native layout is retained. The Earth/sky pass remains the next major feature milestone.

Use one real entity with a birth/event date and one real place. Check whether editing, provenance, date presentation and numeric search feel right before expanding the panels. Then implement the Earth/sky pass with reference fixtures; do not polish a fake astronomy visualization.

### Date / Earth / Sky pass completed (0.3)

Saved-date selection, unlinked date creation, shared metadata editing and Records navigation are implemented. Schema 2 upgrades schema-1 datasets while preserving IDs and unknown source times. The offline map/globe includes Natural Earth outlines, grid, observer selection, new-point creation and solar shading. Astronomy Engine supplies nine bodies, above/below horizon wheels, ecliptic longitudes, lunar phase/illumination, refraction, time stepping and live viewing. Sky properties participate in global search, CLI analysis and comparisons. Eighteen independent JPL reference positions pass the angle-tolerance checks. Live viewing does not alter source records or add undo ticks. See GUIDE-0.3.md and THIRD_PARTY.md.

Next extensions: stars/constellations, rise/set and event calendars, fuller globe textures, geocoding suggestions, broader ephemeris fixtures and the later web/API adapter. Great-circle map arcs are implemented in 0.6.

### Editing / bulk data pass completed (0.4)

Stable place-header IDs preserve focus while renaming. New-date and other view/status changes explicitly schedule repaint. Explicit city/state lookup adds coordinates, elevation and IANA timezone with reviewable results and stored provenance. US Central presets select America/Chicago. A separate bulk JSON format links readable keys within a file, validates atomically, prevents repeated batch IDs and supports preview, new datasets or append, plus global undo. A copyable AI research/format prompt, JSON schema, fictional sports example, PowerShell word-list converter and CLI validate/import commands are included. Schema 3 retains extra JSON attributes; top-level numeric attributes join number search and comparisons.

Next priority: text/facet search, virtualized results and lazy pair comparisons for larger data; then keyed update imports and relationship/multi-participant records. See GUIDE-0.4.md.

### Shared search / NFL / gematria matrix pass completed (0.5)

Saved session filters apply shared case-insensitive text/phrase/exclusion search and kind/tag/conference/division/place/date/source/link facets across record browsing and calculated domains. Scope is computed before bounded pair calculations; active observer/time remain independent. Browse renders visible rows and opens selected records. Map selected-point removal, persistent right-click targets, undo/link cleanup, duplicate double-click suppression and fit-to-filtered-locations are implemented. Gematria now renders one term per row and enabled cipher columns, preserving highlighting and letter breakdowns. A sourced 32-team NFL dataset contains aliases, alignment, home-city reference coordinates and distinct origin/franchise/first-season information, plus a named session launcher. Schema 4 retains entity home/default-place links and session filters.

Next: virtualized gematria/property matrices beyond their current visible caps, indexed record text for very large datasets, keyed update imports/duplicate resolution, multi-participant relationships, exact venue coordinates, and the temporal/cipher/sky extensions described above.

### Focused location comparison completed (0.6)

Schema 5 saves per-session A/B locations, inclusion flags and selected/all-filtered analysis mode. Earth replaces its long feature list with individual location cards and compact pair metrics. The shared analysis engine uses the same ordered selection for Geography and numeric place attributes, while entities/dates and sky keep their independent scope. Legacy sessions retain broader analysis and seed A from the sky observer. Removal cleans comparison references across sessions; undo and persistence cover selections. Map clicks can target Observer/A/B, Observe A/B retains the UTC instant when changing timezone, and map/globe render sampled great-circle routes with seam/hemisphere clipping. Globe entry and fit center on the relevant points. Overview states calculated location/pair coverage and its number-hit purpose. See GUIDE-0.6.md.

### Focused temporal workspace completed (0.7)

Schema 6 saves independent focus/comparison observation references and selected/all-filtered date analysis. Focus selection no longer activates or replaces the viewing clock. A compact three-date summary displays original-to-active spans, previous/next anniversary dates and countdowns, and three pair cards compare original, previous-anniversary and next-anniversary gaps. Unit toggles recompose every shown interval while preserving total days; anniversary countdowns exclude end adjustment. A virtualized, searchable browser measures every filtered saved date against the focus and includes a focus-to-active row, number filtering and promotion into comparison. Shared features include previous anniversary and calendar-component values, with bounded global pair calculations. Migration, deletion cleanup, save/restore and Undo cover date selections. Sky retains saved-date activation. See GUIDE-0.7.md.

### Live gematria / matching completed (0.8)

Schema 7 saves live input, source-cipher selection, same-/cross-cipher mode and cross-domain toggles per session. A shared core search plan distinguishes phrase-derived targets from finite literal numbers and uses the existing displayed-value rounding rules. Gematria retains cipher columns while showing live totals, virtualized matching term/alias rows, expandable full term browsing and numeric properties across Dates/Geography/Sky/Data under existing scopes. New inputs save as gematria entities with deduplication by saved name/alias and an inline type editor that remains available through scope changes. Numeric saved terms contribute one literal Gematria feature, independently of enabled letter ciphers. Clicked totals promote their number to global highlighting. See GUIDE-0.8.md.

### Comparison / active calendar properties completed (0.9)

Comparison uses one expandable row per numerical group, retaining per-domain counts and matching-property inspection. The second inspection list is removed; every shown group is expandable. Dates shows active-date day of year, days left in year, year length and ISO week tiles, with further calendar fields in an expandable section. The shared feature engine includes all nine facts independently of saved-date/filter scope, making them available to global number search, comparisons, cross-domain gematria and CLI export. Leap-year and ISO week-year boundary conventions are tested. Schema remains 7. Date numerology is explicitly deferred. See GUIDE-0.9.md.
