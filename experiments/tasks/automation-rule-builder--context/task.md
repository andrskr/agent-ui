Build an automation rule builder for a support workspace.

This project uses the Astryx design system. Use its installed components. Replace the marked task
code region in `src/app.tsx` and keep the markers. You can add supporting files under `src/`.

Use the field options, sample tickets, and initial rule in `references/data.json`. Copy the data
into `src/` before using it. This is a local rule editor and preview, not an automation service.

## Rule editor

Show a page title, short description, and rule name field. Organize the editor as When tickets
match, Then, and Preview. Start with the supplied saved rule and no unsaved changes.

Use one condition group with an All or Any selector. Support one to five condition rows. Each row
has a field, operator, value control, and Remove action. Add condition creates an empty row. Disable
Add at five rows and Remove when only one row remains. Keep other rows' values when a row is added
or removed.

Support these field types and operators:

- Status: is or is not, with one status choice.
- Priority: is or is not, with one priority choice.
- Subject: contains or does not contain, with a text value.
- Age in days: greater than or less than, with a non-negative whole number.

Changing the field resets that row's operator to the first valid operator and clears its value.
Changing only the operator keeps the value. Text matching ignores letter case and trims spaces at
both ends. Numeric comparisons are strict. Use the sample ticket's ageInDays value; do not calculate
age from the current date.

The action has two choices: Assign to an owner or Set priority. Show the matching owner or priority
choice. Changing action type clears its target. Only one action is supported.

## Summary and preview

Show a readable sentence that describes the current draft's conditions, group mode, and action. Use
clear placeholders for incomplete values instead of silently dropping incomplete rows.

Test rule evaluates the current draft against the eight supplied sample tickets. Validate first.
Show errors beside incomplete or invalid fields and do not run a partial rule. A rule name that
contains only spaces is invalid. Subject values that contain only spaces are invalid. Zero is a
valid numeric value.

For a valid rule, show the matching count and a compact list of matching ticket IDs and subjects,
with the proposed action for each. Do not change the sample tickets. Show an explicit empty state
when nothing matches. Any later draft edit clears the previous preview and marks it as needing
another test.

## Save and reset

Show Save rule and Reset. Disable both when the draft matches the most recently saved rule. Show
Unsaved changes only while values differ. Comparing rules must use their values and condition order,
not temporary row IDs.

Save validates the whole draft, then shows a saving state for one second. Disable editing and repeat
submission while saving. On success, save trimmed name and subject text, make the current rule the
saved rule, and show a confirmation. Saving does not execute the rule on tickets.

Reset restores the most recently saved rule and clears validation messages, preview results, and
confirmation messages. It must not always restore the initial rule.

Keep all state local. Do not add nested condition groups, multiple actions, scheduling, or a network
service. Do not preserve changes after a page reload. Do not add dependencies or change files
outside `src/`. In your final response, state what you built and how you checked it.
