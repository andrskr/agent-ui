Build a project list for a workspace. Replace the marked task code region in `src/app.tsx` and keep
the task code markers. You can add supporting files under `src/`.

This project uses the Astryx design system. Build the interface from its installed components.
Choose the components that fit the task.

Recharts is already installed. Use it for any charts requested by this task.

Show a page title, a short description, and a Create project button. Show the projects in a table
with four columns: Project name, Owner, Status, and Last update. Use text as well as color for
status. Keep the rows in the order below. Do not add pagination, sorting, row selection, or a detail
page.

Use these eight initial projects:

| Project name        | Owner       | Status    | Last update |
| ------------------- | ----------- | --------- | ----------- |
| Website refresh     | Alex Morgan | Active    | 2026-09-17  |
| Customer portal     | Sam Chen    | Active    | 2026-09-16  |
| Mobile onboarding   | Jordan Lee  | Paused    | 2026-09-15  |
| Billing update      | Alex Morgan | Completed | 2026-09-14  |
| Help center         | Sam Chen    | Active    | 2026-09-13  |
| Analytics dashboard | Jordan Lee  | Paused    | 2026-09-12  |
| Brand guidelines    | Alex Morgan | Completed | 2026-09-11  |
| Partner directory   | Sam Chen    | Active    | 2026-09-10  |

Above the table, add a search field and a status filter. Search project names without regard to
letter case. Ignore spaces at the start and end of the search text. The filter has All statuses,
Active, Paused, and Completed. Select All statuses at first. Apply search and filter together. Show
the number of matching projects. When no projects match, show an empty state and a Clear filters
button that resets both controls.

Create project opens a dialog with a project name field, an owner choice, and Create and Cancel
buttons. The owner choices are Alex Morgan, Sam Chen, and Jordan Lee. Both fields are required.
Start with both fields empty. On submit, show a message next to each missing field. A name that
contains only spaces is not valid. Keep entered values when validation fails.

For valid input, show a saving state for one second and prevent duplicate submissions. Then close
the dialog, add the project at the top of the list with status Active and today's date, clear the
search and status filter so the new row is visible, and show a confirmation message. Trim the saved
name. Cancel or close discards the draft. Opening the dialog again starts with empty fields.

Keep all data in local React state. Do not call a network service or save data across page reloads.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
