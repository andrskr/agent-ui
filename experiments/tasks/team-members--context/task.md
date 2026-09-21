Build a team members table using the installed Astryx components.

Use the 12 members in `references/members.json`. Show name, email, role, status, and joined date.
Show an initials avatar beside each name. Start with all members visible and no rows selected.

Add search by name or email, role and status filters, and sorting by name or joined date in either
direction. Combine search and filters. Roles are Admin, Editor, and Viewer. Statuses are Active and
Inactive.

Include row checkboxes and a select-all checkbox for the visible rows. When rows are selected, show
their count and a Deactivate action. This action sets selected members to Inactive and clears the
selection. Clear the selection when search or filters change.

Each row has a menu to change its role to Admin, Editor, or Viewer. Update the row after a choice.
Show an empty state when no members match the filters.

Build only the table and its toolbar. Keep state local. Do not add navigation, network calls,
pagination, or persistence.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
