Build a recent transactions panel using the installed Astryx components.

Show the heading “Recent transactions” and the description “Your latest account activity.” Below it,
add a search field and a status filter with All, Completed, Pending, and Failed.

Display the eight transactions in `references/data.json` in a table. Include merchant, category,
date, status, and amount. Place a small icon beside each merchant. Align amounts to the right and
distinguish incoming payments from expenses. Amounts are in USD; positive values are incoming and
negative values are expenses. Start with all eight rows in the supplied order.

Search filters merchant names without regard to letter case. Search and status filters work
together. Show a clear empty state with a “Clear filters” action when nothing matches.

Build only this panel. Do not add navigation, summary cards, pagination, or transaction details.
Copy the reference data into `src/` and keep all state local.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. Do not call a network service or preserve
state after a page reload. In your final response, state what you built and how you checked it.
