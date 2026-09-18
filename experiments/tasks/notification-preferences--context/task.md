Build a notification preferences page for a workspace.

This project uses the Astryx design system. Use its installed components. Replace the marked task
code region in src/app.tsx and keep the markers. You can add supporting files under src/.

Recharts is already installed. Use it for any charts requested by this task.

Show a page title and a short description. Organize the settings into three sections:

Activity

- Comments and mentions: enabled initially.
- Project updates: enabled initially.

Workspace

- Invitations: enabled initially.
- Product announcements: disabled initially.

Email summary

- Enable email summary: disabled initially.
- When enabled, show a frequency choice: Daily or Weekly.
- Select Weekly initially. Keep the selected frequency when the summary is disabled and enabled
  again.

Each setting must have a visible label and a short explanation.

Add Save changes and Reset buttons. Keep both unavailable until the user changes a setting. Show an
Unsaved changes message while changes exist. If the user restores all saved values, remove the
message and disable both buttons.

Save changes must show a saving state for one second. Disable the settings and buttons while saving.
Then make the current values the saved values and show a confirmation message.

Reset must restore the most recently saved values and clear any messages. It must not always restore
the initial defaults.

Keep all state local. Do not call a network service or preserve settings after a page reload.

Do not add dependencies or change files outside src/. In your final response, state what you built
and how you checked it.
