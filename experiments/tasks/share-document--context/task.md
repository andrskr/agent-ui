Build a document-sharing dialog using the installed Astryx components.

Show the dialog open when the page loads. Use the title “Share Q4 launch plan”.

Show three people with initial avatars, names, and roles:

- Maya Chen — Owner
- Liam Patel — Editor
- Nora Reed — Viewer

Keep the owner’s role fixed. For the other two people, allow switching between Editor and Viewer.

Below the people list, add a “General access” section with two options:

- Restricted — Only the people listed above can access.
- Anyone with the link — Anyone with the link can view.

Start with Restricted. Update the description when the selection changes.

Add a “Copy link” button that copies https://example.com/documents/q4-launch-plan and shows “Link
copied” after a successful copy.

Allow closing the dialog. Show a “Share document” button that opens it again and keeps the current
selections.

Build only this dialog and its opening button. Do not add a document editor, navigation, or an
invitation form. Keep all state local. Do not call a network service or preserve state after a page
reload.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
