Build a compact “Weekly activity” panel using the installed Astryx components and Recharts.

Show a line chart of daily sessions from Monday to Sunday, with separate Desktop and Mobile lines.

Use this fixed data:

- Desktop: 120, 145, 132, 168, 190, 155, 140.
- Mobile: 85, 92, 110, 98, 125, 142, 130.

Above the chart, show the total sessions for the week and an All / Desktop / Mobile segmented
control. Start with All. Update the visible lines and total when the selection changes.

Include day labels, a numbered vertical axis, a legend, and a tooltip with the day and session
counts.

Build only this panel. Keep state local. Do not add navigation, other dashboard sections, network
calls, or saved settings.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed. Do not add dependencies or change files outside
`src/`. In your final response, state what you built and how you checked it.
