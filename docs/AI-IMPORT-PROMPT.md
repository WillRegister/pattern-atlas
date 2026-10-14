Prepare an import file for the native Pattern Atlas research app.

My dataset request: [DATASET REQUEST]

First establish a precise scope (e.g. NFL teams for one season, games for one season, a roster as of a stated date, or a dictionary). Research current facts using authoritative sources. Never invent dates, birth times, locations, statistics or source URLs. If you cannot browse, ask for source data and state that limitation. Describe omissions and unresolved facts separately before delivering the file. Do not claim complete NFL coverage without defining and checking the scope.

Return one valid UTF-8 JSON object, without Markdown fences, comments or trailing commas. Use this exact structure:

{
  "format": "pattern-atlas-data",
  "version": 1,
  "batch_id": "unique-descriptive-batch-v1",
  "dataset": {
    "name": "Dataset name",
    "context": "Scope, as-of date, sources, omissions and uncertainty",
    "entities": [
      {"key": "team-a", "name": "Team A", "kind": "team", "aliases": [], "tags": ["sports"], "context": "Description", "source": "Verified source URL and retrieval date", "attributes": {"season": 2026}}
    ],
    "places": [
      {"key": "venue-a", "name": "Venue A", "latitude": 37.0, "longitude": -93.0, "elevation_m": 0, "timezone": "America/Chicago", "context": "Illustrative coordinates only; replace with verified coordinates and note precision", "source": "Verified coordinate source URL", "attributes": {}}
    ],
    "observations": [
      {"key": "event-a", "label": "Event label", "date": "2026-10-01", "time": null, "timezone": "America/Chicago", "entity_key": "team-a", "place_key": "venue-a", "context": "Time unknown; example only", "source": "Verified event source URL", "attributes": {"home_score_points": 56, "away_score_points": 21}}
    ]
  }
}

Rules:
- The example values are fictional format illustrations. Replace all of them with the requested, sourced data.
- Keep format and version exactly as shown. batch_id is a stable identifier for this file; repeating it is blocked to prevent duplicate uploads. Give separately uploaded chunks distinct batch IDs.
- Keys are readable strings, unique within each array. References must point to keys in THIS file. The app generates UUIDs. References to previous imports are not supported yet.
- Arrays may be empty. For a dictionary, create one entity per word (kind "word"), with no places or observations unless supplied. Do not turn definitions into aliases; aliases are alternate names that also receive gematria calculations.
- Entities can be word, person, team, organization, game or any other descriptive kind. A date has one primary entity; use attributes/context for opponents or additional participants for now.
- In Pattern Atlas 0.5+, an entity can include optional "place_key": "venue-a" to link a home/default place from this file. This is separate from an observation's event place. Do not attach a founding event to today's home venue unless that historical location is actually sourced. Conference/division facets read string attributes named "conference" and "division"; tags provide additional filter categories.
- Dates must be YYYY-MM-DD. Times must be HH:MM:SS in the stated IANA timezone or null if unknown. Do not guess a time; the app separately assumes noon for sky viewing. Do not add fake dates to undated records. Omit records with an unknown date from observations and explain the omission.
- Use IANA timezones such as America/Chicago (US Central, handles CST/CDT), America/New_York or UTC. Never use CST, EST or a fixed UTC offset. Ambiguous/nonexistent daylight-saving local times are rejected; explain them rather than inventing a correction.
- Coordinates are decimal degrees, latitude -90..90, longitude -180..180. West/south are negative. Place timezone is required. Prefer venue coordinates; mark city reference points as approximate. Do not invent coordinates or elevation. elevation_m can be omitted if unknown (defaults to zero; explain that assumption in context).
- source and context preserve provenance and uncertainty. Include direct source URLs and an as-of date, especially for rosters, teams and schedules that change.
- attributes is an object for extra facts: scores, jersey_number, season, capacity, definitions, team names, etc. Store numbers as JSON numbers with descriptive keys including units (e.g. height_cm, score_points). Numeric attributes become number-search/comparison features; strings, arrays and objects are stored and shown as metadata. Nested numeric fields currently do not become comparison features.
- No unsupported fields outside attributes. Do not include computed gematria, date spans or sky positions; the app computes those.
- Imports create new datasets by default. Appending retains old records and creates new ones; it does not update or merge records by name. Use self-contained chunks and avoid repeating entities across chunks intended for the same dataset.
- Check that the JSON parses, that each reference exists, and that the counts match the defined scope. If available, validate locally with pattern-atlas.exe --validate-data file.json before upload.
