Build a compact account settings interface using the installed Astryx components.

Add a left section list with Profile and Notifications. Start on Profile, with an initials avatar,
editable display name, email, and a short bio. Use the initial values in `references/account.json`.

Notifications contains three switches for product updates, weekly summaries, and security alerts,
plus a delivery choice of Email or In-app. Preserve edits when switching sections.

Add Save changes and Cancel actions for the settings. Save requires a nonempty display name and a
valid email. Show inline errors for invalid fields, and show Profile if it has errors. On a valid
save, keep the values as the new saved state and show “Changes saved.” Clear that message when a
value changes. Cancel restores all values to the last saved state and clears errors and feedback.

Build only the settings interface. Keep state local. Do not add network calls or persistence after a
page reload.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
