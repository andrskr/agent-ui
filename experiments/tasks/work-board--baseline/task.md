Build a project work board with three columns and a card detail panel.

This project uses the Astryx design system. Use its installed components. Replace the marked task
code region in `src/app.tsx` and keep the markers. You can add supporting files under `src/`.

Recharts is already installed. Use it for any charts requested by this task.

Use the nine cards and column order in `references/data.json`. Copy the data into `src/` before
using it. Preserve each card's ID, initial column, and initial position. The columns are To do, In
progress, and Done.

Show a page title, a short description, and the total card count. Each column has a heading and a
live count. Cards show title, priority, assignee, and due date. Keep each column's card area
independently scrollable so long columns do not move all other columns.

Allow the user to drag a card into another column or reorder it within its current column. Show a
clear destination indicator before dropping. Dropping inserts the card at the indicated position and
removes it from the old position. Support dropping into an empty column. Dropping outside a valid
destination or cancelling the drag makes no change. A card must never be duplicated or lost.

Provide a Move action on each card. It opens controls for destination column and position. Position
is a one-based insertion position in the destination after removing the moving card. Offer only
valid positions. Confirm uses the same movement rules as dragging. Cancel makes no change.

After a successful move, show a message with the card title and destination, plus Undo. Undo
restores the card's previous column and exact position. Only the latest move can be undone. A new
move replaces the previous undo action. A move to the same position makes no change.

Selecting a card opens a side panel with its full title, description, priority, assignee, due date,
and current column. Show its checklist and completed count. Checklist changes apply immediately to
that card and remain when the panel closes and reopens. Show the checklist count on the board card
too. Do not add title, description, or assignee editing.

If a move is made while a detail panel is open, keep the panel on the same card and update its
column. Closing the panel does not undo checklist or board changes. Undoing a move preserves
checklist changes.

Keep all state local. Do not add card creation, deletion, search, filters, swimlanes, or a network
service. Do not preserve changes after a page reload. Do not add dependencies or change files
outside `src/`. In your final response, state what you built and how you checked it.
