# Pattern Atlas 0.9 · comparison and active-date properties

Open `dist\pattern-atlas-0.9.0.exe` or `Open NFL.cmd`. Save and close an older window before opening the same workspace. This version retains schema 7 and uses the same adjacent `dist\data` directory; it reads schemas 1–7.

## One expandable comparison list

**Comparison** now shows each repeated numerical value once. Its row contains the number of properties in Gematria, Dates, Geography, Sky and recorded Data, plus total domains and properties. Click the arrow or the value to expand the matching records directly underneath. The separate second inspection list has been removed.

Expanded details retain the record, domain, calculation, displayed value and unit. Hover values to inspect raw values and explanations. The toolbar's **Find number**, rounding and decimal settings still apply. Query matches appear first. The existing preview limit is 200 groups; every shown group can expand. A group's details retain the existing 500-property preview limit. CLI `--analyze` exports all properties in the current analysis scope.

## Active-date calendar properties

In **Dates**, below the active-reference controls, four tiles show:

- **Day of year:** January 1 is day 1; February 29 is included in leap years.
- **Days left in year:** civil days after the active date through December 31, excluding the active day. December 31 is zero.
- **Days in year:** 365 or 366 under Gregorian leap-year rules.
- **ISO week number:** Monday-based weeks; week 1 contains January 4.

Expand **More calendar fields** for calendar year, month number, day of month, weekday number (Monday 1 through Sunday 7), and ISO week year. The ISO week year can differ from the calendar year near New Year.

These values follow the active civil date in the toolbar, including Live now in the active timezone. Selecting a focus/comparison date retains its existing behavior; **Use focus as active** also updates the properties. Calendar properties do not depend on saved-date selection or dataset-record filters. Interval unit toggles and **Include end date** do not change these calendar facts.

All nine values participate in **Find number** highlighting, Overview, Comparison, Gematria's **Cross-domain matches** when Dates is enabled, and CLI analysis. For example, use active date `2026-02-25` and search `56`: Day of year is highlighted and can match Will's Ordinal 56. Active date `2026-11-05` has 56 days left in the year. Zero-valued collisions still require **Include zero-valued collisions** in Comparison.

The properties are calculated from the active date rather than added as dataset records. No date numerology has been added in this pass.
