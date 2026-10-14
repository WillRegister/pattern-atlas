# Pattern Atlas 0.4 · editing and data imports

Save and close an older running app before opening `dist\pattern-atlas-0.4.0.exe`. Keep using the same data directory. Double-clicking the versioned executable uses `dist\data`, just like the normal executable. This build reads old schema-1/2 workspaces and saves schema 3, which preserves recorded attributes and imported batch IDs. Export a backup before moving it back to an older build; older builds cannot read schema 3.

## Temporal workspace and place editing

The new-date workflow explicitly schedules refreshes when opened, canceled, saved or rejected. After **Add date to dataset**, the date is selected and becomes the active viewing date; the next frame refreshes the saved-date selector, active clock and numeric features. Unknown source time remains unknown; noon is a viewing assumption. Undo removes the addition and recovers the previous workspace.

Place editor headers and name fields now have stable IDs independent of their labels. Typing a new name retains the open section and keyboard focus.

## City and state to coordinates

Expand **City / state lookup** in Records or Earth. Enter City, State/region (full name or US abbreviation) and a two-letter country code (`US` by default; blank searches globally). Click **Search city**. Select a result after checking the state/country. **Add as new place** stores latitude, longitude, timezone and available elevation. Choose it as the active observer in the sidebar to use it in the map/sky. The lookup itself does not change the viewing clock.

To update an existing location, expand its editor in Records and select **Find city coordinates for this place**. The lookup section opens above. Enter city/state, search, then click **Apply to [place name]**. Coordinates, timezone and elevation change; your label and other metadata stay intact. Source/context receive the lookup provenance. Undo can recover the old coordinates.

Only an explicit search sends the city/state/country to [Open-Meteo](https://open-meteo.com/en/docs/geocoding-api). Searches run in a worker thread, time out after 15 seconds, are limited to one request per second in the process, and reuse a bounded cache in `city-search-cache.json` beside the database. City results are GeoNames reference points, not exact street addresses or stadium coordinates. Attribution is shown in the UI and stored in source notes. Free service is for [noncommercial use](https://open-meteo.com/en/terms); paid/self-hosted deployments can set `ATLAS_GEOCODING_URL` to a compatible search endpoint. Offline maps, sky calculations and manual coordinates still work without city lookup.

## US Central timezone

Use **America/Chicago**, or the **US Central · CST / CDT** timezone preset in the active clock, date form and place editor. It applies daylight-saving rules for each date: CST is UTC−6 and CDT is UTC−5. Avoid the ambiguous abbreviation `CST` or a year-round fixed offset. Other IANA timezone names remain editable.

## Add bulk data

Use the new **Import data** tab:

1. Describe what to collect in the AI request field and click **Copy AI import prompt**. Paste it into an AI chat with research capability. Define a scope such as one NFL season, teams as of a specific date, or one word list. Review its sources and uncertainty.
2. Paste the JSON response, or enter a local `.json` file path and select **Load JSON file**. **Load example JSON** demonstrates the structure using explicitly fictional sports records.
3. Click **Preview import**. Review counts, sample records, coordinates, timezones and sources. Invalid JSON, unknown fields, duplicate keys, broken references, invalid dates/zones/coordinates and ambiguous/nonexistent known local times are rejected.
4. Default import creates a new dataset and session while retaining all existing data. **Append to current dataset** adds separate records; it does not merge or update by name. Repeated entity names are flagged in the preview. References point only to records in the same file.
5. Click **Import reviewed data**. The whole import is a single global undo step. **Save workspace** commits it. Undo/redo remains available after saving.

`batch_id` identifies the file, and a repeated ID is blocked across the workspace. Undo an import before retrying it. Give distinct uploads unique batch IDs. Use Overview's advanced workspace import only for a complete exported backup: it replaces the in-memory workspace rather than adding records.

The import format is separate from internal UUID-based workspace backups. See [docs/import.schema.json](docs/import.schema.json), [examples/import-demo.json](examples/import-demo.json) and [docs/AI-IMPORT-PROMPT.md](docs/AI-IMPORT-PROMPT.md). Human-readable entity/place keys are translated into UUIDs. Optional `attributes` objects preserve extra fields such as jersey numbers, scores, season, venue capacity, definitions and participant names. Top-level numeric attributes participate in global number search and Comparison as **Data / recorded** features. Other attributes remain visible in Records. Nested numeric values are retained but are not indexed as comparison features. Attribute units should be in field names (`height_cm`, `home_score_points`). No computed gematria/date/sky values need to be imported.

For NFL data, represent teams/players/games as entities, venues as places and dated games/founding dates/birthdays as observations. One observation has a primary entity and optional place; other participants can be recorded in attributes/context. Formal relationship edges, multi-entity events and structured statistic units are future work. Unknown dates should not be fabricated; preserve the fact in metadata until a date is sourced. Break large date collections into research scopes: pair comparisons currently grow quadratically.

## Dictionary conversion in PowerShell

Prepare UTF-8 `words.txt`, one word or phrase per line. Blank lines are skipped and case-insensitive duplicate words are removed; original spelling/order is retained.

```powershell
cd D:\Decoder\pattern-atlas
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\words-to-import.ps1 `
  -InputPath .\words.txt -OutputPath .\words-import.json `
  -DatasetName "My dictionary" -BatchId "my-dictionary-v1" -Source "My source"
.\dist\pattern-atlas-0.4.0.exe --validate-data .\words-import.json
```

Then load `words-import.json` in Import data. The converter refuses to overwrite an existing file. It does not create fictional dates or coordinates. Words become entities of kind `word` and receive the selected gematria calculations.

CLI validation does not open or modify a database. CLI bulk import creates a new dataset and session in the chosen database and saves immediately; unlike the GUI, it has no per-run Undo, but saved history remains available. Use a disposable data directory to evaluate it first:

```powershell
.\dist\pattern-atlas-0.4.0.exe --validate-data .\examples\import-demo.json
.\dist\pattern-atlas-0.4.0.exe --data-dir .\scratch-data --import-data .\examples\import-demo.json
```

## Suggested next passes

1. Search and filter saved dates/entities/places by text, tags, source and kind; virtualized tables and bounded/lazy pair comparisons for large datasets.
2. Import updates keyed to existing records, duplicate resolution, progress/error reports, formal relationships and multi-entity events.
3. Richer temporal tools: add/subtract calendar durations, date-number breakdowns, pinned comparisons and explicit DST offset choice for ambiguous times.
4. Sky additions: rise/set, stars and constellations; exact location timezone lookup and great-circle map arcs.
5. Broader/custom cipher tables and calculation fixtures, saved query/cipher presets, shareable reports, and a web/API adapter using the same storage/model.

Existing map/sky views and the native layout remain in use. This pass does not claim Gematrinator cipher parity or a complete NFL database.
