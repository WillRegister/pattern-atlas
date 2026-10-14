# Pattern Atlas 0.6 · focused location comparison

Run `dist\pattern-atlas-0.6.0.exe`, or use `Open NFL.cmd` for the installed NFL session. The executable uses the same adjacent `dist\data` workspace as previous builds. This version reads schemas 1–5 and saves schema 5. After saving with 0.6, keep using 0.6 or newer.

## Choose the points you want

Open **Earth**. Choose saved locations in the **A** and **B** dropdowns. **Include A/B** temporarily hides either point's detail card and excludes it from the focused pair, without deleting the saved record. Selecting one point displays its coordinates, timezone, elevation and expandable context/source. Selecting two distinct enabled points also displays their distance, central angle and coordinate differences in compact cards. The previous long geography-property list is gone.

**Selected A/B only** makes Overview, number search and Comparison use those enabled locations, their pair calculations and their numeric recorded attributes. Other entities, dates and sky calculations retain their own scope. **All filtered locations** restores broader location analysis, including pairwise calculations across all locations that pass the session's data filters. Earth still shows the chosen pair's details, while the map shows the broader set.

Existing sessions retain their previous all-filtered analysis mode when upgraded. Their sky observer seeds A; choose B and click **Selected A/B only** to focus the analysis. New sessions start in selected mode. No locations are deleted or automatically filtered out of the saved dataset when you select a pair.

Dropdowns include all saved locations. A point hidden by the existing data filters is marked **(filtered out)** and excluded from the pair until you clear or adjust the filters. The counts next to Search data describe available filtered records; the location-analysis summary describes the actual calculated location scope. A/B selection is separate from the existing location facet, which can also restrict related entities and dates.

## Map and sky

**Connect A/B** draws the same spherical great circle used to calculate their distance, on both World map and Globe. World-map routes break at the dateline; the globe only draws the visible hemisphere. Exactly antipodal points have multiple shortest paths; the renderer chooses one stable path. Switching to Globe centers the view on the chosen pair. **Fit locations** frames the currently displayed analysis locations.

**Map click selects** chooses whether clicking a saved marker changes the sky observer, A or B. Choosing A/B leaves the sky observer unchanged and enables the selected point. Double-clicking empty world-map space creates a saved point and assigns it to the chosen target.

**Observe A/B** sets that point as the sky observer and switches to its timezone while preserving the active UTC instant. Sky, the wheel and day/night shading use this observer and the active time. The observer can remain visible as a separate marker even when it is outside the selected pair. The sidebar's observer controls remain available.

**Edit A/B** opens Records at the chosen location. **Remove A/B** deletes the saved point and clears its home, event, observer, filter and comparison links across sessions. Dates and entities remain. Right-click marker removal and the existing observer removal button are also available. Global Undo restores the point and every link; selections, inclusion toggles and analysis-mode changes can also be undone. Saved A/B choices belong to each session and survive reopening.

## Values and Overview coverage

Distance units are individually toggleable: kilometers, miles, meters and feet. Toolbar Raw/Nearest/Floor and Decimals settings apply after conversion, and the number-query highlight uses the same displayed values. Hover a coordinate or calculated value for its exact result. Elevation, timezone and context/source are record metadata.

Signed differences mean **B minus A** in selected mode. All-filtered pair order follows dataset order. Raw absolute longitude difference and shortest longitude separation are distinct; the latter wraps around the dateline. Distance is a spherical surface estimate using mean radius 6371.0088 km, rather than a road route or ellipsoidal survey.

Overview displays **number-query hits**, rather than listing every calculated property. Comparison groups matching displayed numerical values across the enabled domains. Both use the location mode chosen in Earth and the session filters. Pair limit bounds calculations per domain (default 10,000); the scope summary shows calculated versus possible location pairs. A limit of zero disables pair calculations while retaining individual coordinates. Overview shows at most 500 hit rows and Comparison at most 200 groups, so those panels are previews of the calculated results.

Examples with the NFL dataset: choose **Seattle, Washington** and **Glendale, Arizona**, include both, select **Selected A/B only**, then use **Fit locations**. Change Find number to `14` with Nearest / 0 decimals to highlight the rounded absolute latitude separation. The included NFL coordinates are home-city reference points, not exact stadium coordinates.
