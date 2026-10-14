# Dates, Earth and Sky — version 0.3

Open `dist/pattern-atlas-0.3.0.exe`, using your existing data directory. Double-clicking either the versioned or canonical executable uses the same adjacent `dist/data` folder. Save and close older app windows before opening the new build.

## Dates and shared records

In **Dates**, choose a saved dataset date, step through dates, or use **+ New date**. Enter a label and date, with optional source time, entity and location. **Add date to dataset** creates a canonical record; a date can be unlinked initially. Leave its time blank when unknown.

Expand **Selected date · entity, location and context** to edit the record, or use **Details in Records**. Records now has **Unlinked dates** and an entity selector in each date editor, so you can attach a previously entered date to a person/event later. Context, source, location and timestamps are the same data in both panels. Undo/redo covers creation, linking, editing and removal.

Changing the saved-date selection or pressing **Use as active date/time** sets the session's viewing date/time and linked observer. Edits to that source's time/date/location refresh an active linked reference. Entering a custom toolbar time or using sky time controls detaches the reference and preserves the original source date. Dates are compared against the active civil reference date.

A date-only source is viewed using **local noon**, in its linked location's timezone when available, otherwise its recorded timezone. Its source time stays unknown. **Assume noon**, **Assume midnight**, the time slider, and explicit fields let you explore other assumptions. A viewing assumption does not supply a missing historical timestamp.

## Earth

The offline world map includes Natural Earth land/coastlines and a latitude/longitude grid. Drag to pan; scroll to zoom. **Globe** switches to an orthographic coastal-outline view that rotates by dragging. **Center observer** brings the active location into view.

Click a dataset point to observe the sky there. Double-click the flat map to add an approximate location; name it and fill in timezone, coordinate precision and evidence in Records. Existing distance/unit/delta calculations remain available below the map. Solar day/night shading and the subsolar marker follow the active time; shading uses a geometric terminator with a twilight fade.

## Sky

Astronomy Engine calculates Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus and Neptune. The above-horizon wheel looks upward (east left); the below-horizon wheel looks downward (east right). Both use north azimuth 0° and east 90°. Rings mark altitude 0°, 30°, 60° and zenith/nadir 90°.

Ecliptic mode shows **geocentric longitude of date** independently of observer location. Full ecliptic latitude remains in the numerical table. Optional standard refraction changes the horizon presentation; geometric properties remain separately inspectable. Lunar illumination/phase, azimuth/altitude, RA/declination, ecliptic coordinates, body distances and angular separations feed global search and the Comparison matrix's **Sky** column. Hover numerical values for raw measurements and calculation context.

Use the saved-date chooser, hour/day buttons or local-time slider to explore a moment. **Live now** updates positions without rewriting historical timestamps or adding clock ticks to undo history. Turning it off freezes the viewing moment as one session edit. Live mode is a transient view, rather than a saved preference. Local-clock stepping rejects DST ambiguity/nonexistent times; use explicit UTC to resolve those transitions.

Sky dates currently support **1700–2200**. Calendar records support their wider range. This pass renders solar-system bodies; it has no star/constellation catalog, rise/set calendar, terrain/weather visibility model or automatic IP/geocoding lookup. Above horizon does not guarantee naked-eye visibility in daylight. See [THIRD_PARTY.md](THIRD_PARTY.md) for licenses, models and independent JPL fixtures.

## Existing data and programmatic access

Workspace **schema 2** permits unlinked dates and stores the active saved-date reference/refraction setting. Version 0.3 reads old schema-1 data and saved revisions while preserving IDs, entity links and unknown times. Saving/exporting writes schema 2; use version 0.3 for those files.

CLI `--analyze` now includes astronomical features when the saved session has a valid observer/time. The shared Rust core remains available without GUI/SQLite; its optional `astronomy` feature enables the bundled native C backend. A later local web/API adapter can use that same native engine. A browser-only WASM build will need a separate astronomy adapter/toolchain.
