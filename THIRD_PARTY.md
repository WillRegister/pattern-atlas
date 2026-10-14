# Bundled astronomy and map data

## Optional city lookup (0.4)

Explicit city/state searches use Open-Meteo Geocoding: https://open-meteo.com/en/docs/geocoding-api . Its location data is based on GeoNames, under CC BY 4.0: https://creativecommons.org/licenses/by/4.0/ . Attribution is visible in Records/Earth and saved in place source notes. City points are approximate reference locations, not surveyed venues. Free endpoint terms are noncommercial: https://open-meteo.com/en/terms . Requests occur only on Search, run outside the UI thread, timeout in 15 seconds, are spaced ≥1 second in the process, and responses are cached locally. The optional `ATLAS_GEOCODING_URL` environment variable accepts a compatible endpoint. It is not used for bulk AI-data imports. Other app features remain offline.

## Astronomy Engine

Don Cross's MIT-licensed Astronomy Engine: https://github.com/cosinekitty/astronomy . Pinned commit `865d3da7d8112bbc7911238052c6af4aaf877181`. Bundled C/header files are in `crates/astro/vendor`; upstream notices and `LICENSE` are retained. Pattern Atlas's `atlas_shim.c` keeps C types behind a small checked ABI. Rust serializes engine calls because it has internal caches.

## Natural Earth

Natural Earth 1:110 million land polygons are bundled in `crates/desktop/assets/land.geojson`. Source: https://github.com/nvkelso/natural-earth-vector/blob/ca96624a56bd078437bca8184e78163e5039ad19/geojson/ne_110m_land.geojson . Pinned commit `ca96624a56bd078437bca8184e78163e5039ad19`. Public-domain terms: https://www.naturalearthdata.com/about/terms-of-use/ . These outlines are for orientation; a map click is not a surveyed coordinate and does not identify a timezone.

Earcutr 0.4.3 (ISC license) triangulates land polygons: https://github.com/frewsxcv/earcutr/ . Cargo.lock pins Rust dependency versions.

## Independent reference fixtures

Tests compare topocentric, airless azimuth/altitude against NASA/JPL Horizons for Sun, Moon and seven planets at Greenwich (longitude 0°, latitude 51.4779°, elevation 46 m), 2024-03-20 12:00 UTC and 2024-03-21 00:00 UTC. Requests/reference values are bundled in `crates/astro/tests/jpl-reference.json`; local raw QA responses are excluded from Git. Documentation: https://ssd-api.jpl.nasa.gov/doc/horizons.html . Tests and running app need no network.

The test tolerance is <0.05° in each angle for those 18 fixtures, not a universal accuracy guarantee. This build permits astronomical dates 1700–2200 and observer elevations −12,000 to 100,000 m. Calendar records retain their wider date range. UTC serves as the UT input, with Astronomy Engine's modeled terrestrial-time conversion. Optional atmospheric refraction is a standard model rather than measured weather.
