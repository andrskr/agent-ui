Build a support overview dashboard for a workspace.

This project uses the Astryx design system. Use its installed components. Replace the marked task
code region in `src/app.tsx` and keep the markers. You can add supporting files under `src/`.

Use the fixed data in `references/data.json`. Copy the data into `src/` before using it. Do not
replace the supplied data with random values or values based on the current date.

Show a page title, a short description, and a period selector with This week and Last week. Select
This week initially. Show the dates of the selected period beside the selector.

Show three metric cards: Tickets created, Tickets resolved, and Average first response time.
Calculate Tickets created from the daily values. Use the supplied resolved count and response time.

Below the cards, show a vertical bar chart of tickets created from Monday through Sunday and a donut
chart of tickets by category. Each chart must have a title and a short explanation of what it
measures.

Use the same vertical scale for the bar chart in both periods. Start the scale at zero. Show day
labels and exact values without requiring hover. Add a tooltip with the day and value for each bar.
Keep category colors consistent between periods. The donut legend must show each category name,
count, and percentage rounded to one decimal place. Show the total ticket count in the donut center.

Changing the period must update all cards, charts, dates, and labels together. Do not show values
from different periods at the same time. Do not add editable widgets, extra filters, tables, or
chart drill-down pages.

Keep all state local. Do not call a network service or preserve the selected period after a page
reload. Do not add dependencies or change files outside `src/`. In your final response, state what
you built and how you checked it.
