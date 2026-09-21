Build a compact “Payout settings” form using the installed Astryx components.

Add a currency selector with USD, EUR, and GBP. Select USD initially.

Add a “Minimum payout” number input and slider. Keep their values in sync. Allow amounts from 50 to
10,000. Start at 2,500. Display the selected currency beside the amount.

Add an optional Notes field.

Add a “Save settings” button. Show an inline error if the amount is empty or outside the allowed
range. On a valid save, show “Settings saved.” Clear this message when a value changes.

Build only this form. Keep state local. Do not add navigation, network calls, or persistence.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
