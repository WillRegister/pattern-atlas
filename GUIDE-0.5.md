# Pattern Atlas 0.5 · shared search, map management and NFL

Use `dist\pattern-atlas-0.5.0.exe`. This version reads workspace schemas 1–3 and saves schema 4, which adds session filters and entity home/default-location links. Keep using 0.5 after saving; older versions do not understand schema 4.

## Gematria matrix

The Gematria panel now has one row per term (canonical entity name or alias). Each enabled cipher has a column headed by its name, with the numerical total in its cell. Turning a cipher off removes its column immediately; re-enabling restores it in the standard cipher order. The parent entity is shown alongside aliases so repeated city names can be distinguished. Number-query matches are highlighted and shown first. Hover a total for per-letter contributions. Horizontal scrolling accommodates more cipher columns. The current table shows at most 500 terms; narrow the scope to inspect a larger dictionary. This presentation change does not alter cipher calculations or Comparison's property-level inspection.

## Remove or edit map locations

Click a marker to select the observer. **Remove selected location**, above the map, deletes it. You can also right-click a marker and choose **Remove [name]**. Overlapping points are listed separately in the context menu. The menu retains its target when you move the pointer into it. **Edit selected location** opens its record editor. Records still provides **Remove place**.

Removing a location clears home/default, event-location, observer and location-filter links while retaining entities and dates. **Undo / Ctrl+Z** restores the location and all links. Double-clicking an existing marker selects it rather than creating a duplicate; double-click empty world-map space to create a new location. **Fit locations** frames the locations in the filtered scope. Panning, zooming, world/globe switching and label toggles remain available.

## Shared session scope

**Search data** searches saved names, aliases, context, sources, tags and attributes, including related entity/event/home-place information. It is separate from **Find number**, which compares computed numerical properties.

Linked places contribute identity fields (name, coordinates, zone and attributes). Their source citations are searched as place records, so a shared stadium's citation of one team does not exclude another team when you type `"Los Angeles" -Chargers`.

- Terms are case-insensitive and combined with AND: `Seattle NFC`.
- Quotes preserve a phrase: `"New York"`.
- A leading minus excludes text: `"New York" -Jets`.
- Clear incomplete quotes or invalid date boundaries before saving.

Expand **Filter dataset scope** for kind, tag, conference, division, location, inclusive date range, linked/unlinked dates and sourced-only filtering. Conference/division choices are populated from entity attributes, so these controls work with the NFL dataset without hard-coded NFL lists in the UI.

The same scope drives Browse, date selection/summary, gematria, map markers and numerical comparisons. Filters are stored with each session and participate in global Undo/Redo. They never delete records. **Clear data filters** resets the filter and entity selection. Counts show matching versus total entities, dates and locations. Entity facets retain their connected event/home locations. Date-range/link filters restrict entities to those with matching dates. Location filtering accepts either an event's location or its entity's home/default place. Source filtering checks the record's own source field. The active observer and viewing time remain unchanged; its marker and sky calculations stay available even when the observer is outside the filtered scope, with a visible notice.

An already selected record stays editable if filtering excludes it. Location editors remain visible while you rename the selected place, preventing search from closing the field mid-edit. New places added in Records open their editor automatically. A home/default place is a separate entity field; it does not claim that historical events occurred at today's home venue.

## Browse and large datasets

The **Browse** tab offers All records / Entities / Dates / Locations, a local result search and **Open record** navigation. **Browse all saved records** temporarily bypasses session filters for browsing only. The result list renders visible rows rather than every saved record. Entity/date dropdowns and the Records place list also offer local search. Local browsing text is a scratch buffer with normal text undo.

Pairwise calculations happen after scope filtering. The default limit is 10,000 pairs per domain, configurable from 0 to 100,000. A notice states when pairs are omitted, and selection follows dataset order. Set the limit to zero to turn off pair comparisons while retaining individual-record features. Narrow the scope for targeted comparisons rather than treating the bounded sample as exhaustive. Other property tables show at most 500 rows and Comparison shows at most 200 groups; queried-number matches are prioritized before those caps. CLI analysis exports the calculated features under the same filters and pair limit.

CLI `--analyze` currently reads the first saved session; selecting another session in the native UI does not change that CLI target.

## NFL dataset and session

`datasets\nfl-teams-2026-10-13.json` contains all 32 franchises as reviewed on 2026-10-13: 16 AFC, 16 NFC, four per division, 30 distinct home-city reference locations (shared home cities are linked), and 33 dated historical milestones. Canonical names receive gematria directly; aliases include nickname, brand location and abbreviation, with familiar variants where useful. For example, Seattle Seahawks is the canonical name; Seahawks, Seattle, SEA and Hawks are aliases. City aliases can intentionally belong to several teams.

Team metadata distinguishes founding/origin year, a dated founding basis when available, franchise-award/admission date, first league season and the NFL profile's established year. It includes conference, division, home city/state and current stadium name. Some originals predate NFL admission; year-only origins remain year-only. Historical observations have unknown clock times and locations and are not attached to today's stadium. UTC/noon for unspecified historical clocks is a viewing convention.

The Rams' franchise date is explicitly disputed: the Hall of Fame reports February 12, 1937, while the Rams' narrative reports February 13. The file preserves the Hall date, the alternate date, both sources and a disputed status. No exact formation day is invented for year-only predecessor origins.

Home coordinates are approximate GeoNames city reference points obtained through Open-Meteo, not exact stadium positions. Venue municipality differs from the team name for several clubs: Dallas plays in Arlington, San Francisco in Santa Clara, the New York teams in East Rutherford, the Los Angeles teams in Inglewood, and New England in Foxborough. Distances between these records are city-reference distances. Provenance is stored per record; see [datasets/NFL-SOURCES.md](datasets/NFL-SOURCES.md).

**Open NFL.cmd** opens the named NFL session in 0.5. Its PowerShell script backs up the existing workspace, adds the supplied batch if absent, preserves other datasets/sessions and starts with Chicago's home-city point as a Central-time observer. Choose any other home location in the observer selector. Save/close existing Pattern Atlas windows first; the launcher checks for them. It never repeats an already installed batch. To install into another workspace without launching:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\open-nfl.ps1 -DataDir D:\MyAtlasData -NoLaunch
```

Imports can now supply an optional entity `place_key` referencing a place in the same file. Existing bulk files remain compatible. The copyable AI prompt and JSON schema describe this link. CLI `--ensure-data` skips an installed batch, `--session-name` names its new session, `--observer-place` chooses a named imported place, and `--open-session` selects a session when launching. These CLI writes are immediate saves; use saved history for recovery after restarting.
