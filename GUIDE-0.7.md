# Pattern Atlas 0.7 · focus dates and anniversaries

Save and close any older Pattern Atlas window, then open `dist\pattern-atlas-0.7.0.exe`. The same adjacent `dist\data` workspace is used. `Open NFL.cmd` also targets 0.7.0. Schemas 1–5 upgrade to schema 6; after saving in 0.7, continue using 0.7 or newer.

## Three dates, one focused view

Open **Dates**. Choose a saved **Focus date** and optionally a **Compare date**. Both searchable dropdowns include the record's date, entity and event label. Choosing either leaves the active date, time, observer and source records unchanged. **Previous date** and **Next date** navigate the focus through the filtered saved dates. **Swap dates** exchanges focus and comparison.

Set the **active date** in the top toolbar to your reference day: for example, a game day. The panel displays its weekday. The main table then shows both selected saved dates against that active reference:

- Original date to active date, as total days plus the chosen calendar breakdown.
- Previous anniversary's actual date and elapsed distance from it to the active date.
- Next anniversary's actual date and distance from the active date to it.

Three compact cards compare the two original dates, their two previous anniversaries and their two next anniversaries. The anniversary occurrences are chosen around the active reference, which can be any valid date. Duplicate selections do not create a self-comparison pair. A future event has no previous anniversary until it occurs; its first future occurrence is its recorded date. On an anniversary itself, the previous and next occurrence are both today, with zero days since/until.

**Years**, **Months**, **Weeks** and **Days** recompose every interval in the focused view. Total days stay visible for quick comparison. For example, turning Years off rolls the year portion into calendar months; turning Years and Months off allows a weeks/days breakdown. Calendar months use month-end clamping. An omitted Days toggle does not discard residual days. Signed original-to-active counts are negative when the saved event is later than the active date. The pair cards show absolute gaps.

**Include end date** applies to original event spans. Anniversary elapsed/countdown intervals and anniversary pair gaps always use civil-day offsets, so an occurrence today is still zero days away. The February 28 / March 1 policy handles February 29 anniversaries in non-leap years. These are civil calendar comparisons of recorded dates; source times and timezones are not silently converted into different event dates.

## Explore every relationship without a giant list

Expand **Explore focus against the dataset**. Each matching saved event is measured against the focus date, including that event's previous/next anniversary around the focus. The first row is a special **Focus to active** relationship, keeping the active reference in the same browser.

Search by entity, label, date, context or source. **Only number matches** narrows rows to intervals containing the toolbar's query in their total-day count or enabled calendar components. The browser scrolls through all matching rows without a 500-row display cap. Long cells truncate in the row; hover for the full interval and anniversary date. **Compare** brings a saved row into the main three-date view. Shared data filters still apply; a selected date excluded by them remains selected and editable, with a notice explaining why its calculations are hidden.

The browser and the focused summary have different references: browser saved-date rows point to the **focus**, while the main summary points to the **active** date. Their captions and the browser's reference date make that distinction explicit.

## Analysis scope, editing and persistence

**Selected dates only** limits the date features used by Overview, number search, CLI analysis and the cross-domain Comparison matrix to focus, comparison and their relationships to active. **All filtered dates** uses the broader saved-date scope and its bounded pair calculations. This choice does not change entity, location or sky scope. The date-analysis summary reports the number of dates and calculated saved-date pairs. Pair limit bounds the global saved-date pair engine; focused detail cards and the browser calculate their requested intervals directly.

Global features now include days since the previous anniversary, days until the next, enabled anniversary calendar components, previous-/next-anniversary pair spans and the pair's calendar components. Existing original-date span features remain. Overview displays query hits; the Comparison matrix groups matching displayed numbers. Current Overview and matrix display caps remain.

Existing sessions retain all-filtered analysis when upgraded, and their active saved event (or first saved date) seeds the focus. New sessions start with selected-date analysis. Imported datasets seed focus from their first saved date. Focus, comparison, unit toggles and analysis scope belong to the session, survive saving/reopening and support global Undo/Redo.

**Use focus as active** deliberately sets the toolbar's date/time and observer from the focus record, preserving the existing unknown-time/noon rules. The Sky panel retains its separate saved-date activation control. **+ New date** creates a dataset record and activates/focuses it, as before. Expand **Selected date · entity, location and context** to edit the focus or open its details in Records. Removing a date or its parent entity clears all affected saved session references; Undo restores the data and selections.

## Seattle / Arizona example

Set active to **Sunday, October 18, 2026**, choose **Seattle Seahawks · franchise awarded — 1974-06-04** as focus and **Arizona Cardinals · franchise awarded — 1920-09-17** as comparison. This is an example reference date; no game schedule is asserted.

| Relationship | Days |
|---|---:|
| Seattle original to active | 19,129 |
| Arizona original to active | 38,747 |
| Original saved-date gap | 19,618 |
| Seattle since previous / until next anniversary | 136 / 229 |
| Arizona since previous / until next anniversary | 31 / 334 |
| Previous-anniversary / next-anniversary pair gaps | 105 / 105 |

The Cardinals' 1898 founding is year-only metadata. The saved 1920 date is explicitly the franchise award; the program does not invent a day/month for 1898. Unknown historical dates need a sourced record before exact anniversary calculations are possible.
