Build a support ticket workspace centered on an advanced data table.

This project uses the Astryx design system. Use its installed components. Replace the marked task
code region in `src/app.tsx` and keep the markers. You can add supporting files under `src/`.

Recharts is already installed. Use it for any charts requested by this task.

Use the 48 fixed tickets in `references/data.json`. Copy the data into `src/` before using it. Do
not generate random values or use the current date. Keep each ticket's ID stable after updates.

## Data and layout

Show a page title, a short description, and the total ticket count. Put search, filtering, sorting,
and column controls above the table. Show columns for ticket ID, subject, customer, status,
priority, assignee, created date, and last updated date. Show Unassigned for a null assignee.

Show status and priority with text as well as color. Keep ticket IDs and selection controls visible
during horizontal table scrolling. Keep column headers visible during vertical table scrolling. Keep
long subjects within their column and make the complete text available on hover.

## Search and filtering

Search ticket ID, subject, and customer without regard to letter case. Ignore spaces at the start
and end of the search text. Search applies as the text changes.

Provide a filter panel with multiple status choices, multiple priority choices, multiple assignee
choices including Unassigned, and an inclusive created-date range. Either date boundary can be
empty. Within one field, match any selected value. Across fields, require all conditions to match.
No selection within a field means no restriction. Apply search together with the filters.

The panel has Apply and Cancel actions. Edits do not affect the table until Apply. Cancel or
dismissal discards unapplied edits. Reopening the panel shows the applied values. Prevent Apply when
the start date is later than the end date. Show a field error.

Show one removable chip per selected status, priority, and assignee, plus one chip for the date
range. Removing a chip updates the results. Clear all resets both search and applied filters. Show
matching count and total count. If nothing matches, show an empty state with Clear all and keep the
controls available.

## Sorting and columns

Provide a sort panel with up to three different sort fields. Show their precedence and allow each
field to move up or down, change direction, or be removed. Changes apply immediately. Support
priority, status, created date, and last updated date. Prevent duplicate fields.

Ascending priority order is Low, Medium, High, Urgent. Ascending status order is Open, In progress,
Waiting, Resolved. Sort dates chronologically. Use ticket ID ascending as the final tie-breaker,
including when no sort fields remain. Initially sort by last updated date, newest first. Reset
sorting restores that rule. Show active fields, directions, and precedence outside the panel.

Allow the user to hide or show Customer, Assignee, Created date, and Last updated date. Keep other
columns visible. Hiding a column does not remove its sort rule or filter. Restore columns shows all
columns again.

## Pagination and selection

Provide page sizes of 10, 20, and 50. Start with 10. Show the visible range, matching count, current
page, and total pages. For zero results, show 0 results and disable page navigation.

Apply search and filters first, sorting second, and pagination last. Reset to page one when search,
applied filters, sorting, or page size changes. Unapplied filter edits do not reset the page.

Support row selection with checkboxes. The header checkbox selects or clears only the current page.
Show a mixed state when some visible rows are selected. Selection uses ticket IDs. Preserve
selection across page, page-size, column, and sorting changes. Clear selection when search or
applied filters change. Show the total selected count and a Clear selection action.

## Bulk action

When rows are selected, show an action to change their status. Show the chosen status and selected
count in a confirmation dialog. Cancel makes no changes. Confirm shows a saving state for one
second. Disable table controls and dialog dismissal while saving to prevent duplicate submission or
changes to the selection.

After saving, update all selected tickets and their last updated date to the fixed bulk-action date
in the data. Clear selection and show a message with the number of selected tickets updated. Reapply
filters and sorting. If the current page no longer exists, move to the last available page. Handle
zero remaining results with the empty state.

Keep all state local. Do not call a network service or preserve changes after a page reload. Do not
add ticket creation, ticket detail pages, or export features. Do not add dependencies or change
files outside `src/`. In your final response, state what you built and how you checked it.
