# Pattern Atlas 0.8 · live gematria and matching

Save and close the older app before opening `dist\pattern-atlas-0.8.0.exe` or `Open NFL.cmd`. This build uses the same adjacent `dist\data` workspace. It reads schemas 1–7 and saves schema 7; keep using 0.8 or newer after saving with this version.

## Live input and cipher columns

In **Gematria**, type a phrase or a number in **Live input**. The phrase's totals update immediately in the enabled ciphers. Hover a total for its letter contributions. The enabled cipher checkboxes control both the input totals and the saved-term columns. Letter scoring remains ASCII A–Z and case-insensitive; punctuation, spaces and digits inside a phrase contribute zero.

**Match input values from** chooses which input cipher supplies the target value. **All enabled ciphers** uses all enabled input totals. To match only Will's Ordinal value of 56, choose **Ordinal**. Clicking an input total also selects that cipher and copies its value into the toolbar's **Find number** field, highlighting the number in the other panels.

The saved-term matrix displays matching terms and aliases as rows, with enabled ciphers as columns. Matching cells are gold. Entity and type remain visible; click the entity name to open Records. **Browse all saved terms in scope** expands the complete matrix. The tables scroll vertically and horizontally rather than stopping after 500 rows. With empty Live input, the full saved-term matrix appears directly and retains the toolbar query's highlighting.

## Same cipher or cross cipher

- **Cross-cipher matches on:** a saved term can match a target value in any enabled cipher. For Will with Ordinal selected, another term scoring 56 in Reverse Ordinal is a match.
- **Cross-cipher matches off:** phrase-derived targets match only the corresponding cipher. Will's Ordinal 56 matches another term's Ordinal 56, while a 56 found only in Reverse Ordinal is excluded.
- **All enabled ciphers:** with cross-cipher off, each candidate cipher is compared to that same input cipher. With it on, every candidate enabled cipher is compared to any enabled input total.
- A disabled source cipher supplies no targets. Re-enable it or choose another source; a disabled candidate cipher never participates.

A numeric input such as `56` is a literal target, rather than a phrase whose digits score zero. Since a number has no originating cipher, **All enabled ciphers** searches the number in every enabled cipher in either mode. Choose a specific source cipher and turn cross-cipher off to lock a numeric target to that cipher. Literal saved numbers have no cipher and can match the numeric target in either mode.

Decimal and signed numeric inputs are accepted. Matching follows the existing rule: the displayed feature value, after toolbar rounding and decimals, must equal the entered target. For example, a rounded display of 56 matches input 56; it does not match input 56.4. Non-finite numbers and punctuation-only/no-A–Z phrases are rejected. Live inputs are limited to 4,096 characters. Invalid drafts can remain in the search field without becoming saved records.

## Save a phrase or number

Click **Save input as entity**, or press Enter in Live input. A new canonical entity is added to the current dataset with type **gematria** and a manual-input source. The input stays visible. Change **Entity type** beside the saved record to `person`, `team`, `place`, or another kind; **Edit record** opens the full editor. The type field remains available and focused while typing even if a data filter excludes the changed type from the result matrix.

Existing names and aliases are reused case-insensitively after trimming surrounding whitespace. Saving them does not create a duplicate or overwrite their existing type. When input is an alias, the displayed record and type belong to its parent entity, which is labeled explicitly. Input values and cipher totals are computed rather than stored as stale scores.

Saved numeric names/aliases appear in a separate **Number** column, with dashes in cipher columns. They contribute one **Gematria / Literal number** feature, even with every letter cipher disabled. Their digits do not create a collection of zero-valued scores. This numeric-term convention also applies to existing or imported numeric names and aliases and to entities whose type you later change.

Creation, type edits, input, source selection, cross-cipher mode and cross-domain filter changes support global Undo/Redo and persist with their dataset/session as appropriate. Saving does not erase Undo history. Other sessions keep their own live input and match settings.

## Cross-domain matches

The separate **Cross-domain matches** section uses the input's selected target values to find numeric properties in **Dates**, **Geography**, **Sky** and **Recorded data**. Each domain can be toggled. It shows the record, calculation, gold matching value and unit; hover the value for its raw number and explanation. One result from each matching domain is shown before repeated properties from the same domain.

Examples include longitude 56°, a 56-mile or 56-kilometer distance when that unit is enabled, 56 signed/total elapsed days, an anniversary countdown of 56 days, or numeric record metadata equal to 56. These are equal numerical representations across their labeled units.

Results use the same calculated feature set as Overview: the current dataset filters, selected/all-filtered Date and Earth analysis modes, active date/time, sky observer, unit toggles, rounding and pair limits. Broaden those scopes in Dates or Earth when you want more date/location comparisons. Cross-cipher locking affects gematria candidates; cross-domain properties match the selected numbers regardless of cipher. Results are drawn in a scrolling table, without a 500-result truncation.

The toolbar query stays independent while you type. Click a phrase total or **Highlight number everywhere** for a numeric input to promote one value to global highlighting. Overview and Comparison retain their existing purpose and preview caps.
