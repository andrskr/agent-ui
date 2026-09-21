Build a plan-selection section using the installed Astryx components.

Show Starter, Pro, and Business side by side. Use the fixed data in `references/plans.json`. Each
plan shows its name, short description, price, feature list, and selection button. Mark Pro as
recommended. Make the selected plan clear.

Add a Monthly or Yearly billing choice. Start on Monthly with Starter selected. Monthly shows the
monthly charge. Yearly shows the total annual charge and the savings compared with 12 monthly
payments. Format all amounts in USD.

Selecting a plan updates a summary below the plans with the plan name, billing period, and amount
due for that period. Changing the billing period keeps the selected plan and updates the summary.

Build only the plan-selection section. Keep state local. Do not add checkout, network calls, or
persistence after a page reload.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
