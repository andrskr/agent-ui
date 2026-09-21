Build a two-step sign-in panel using the installed Astryx components.

The first step contains email and password fields, a show-password control, and a Continue button.
Start with empty fields and the password hidden. Continue requires a valid email and a nonempty
password. Show inline errors for invalid fields. Any valid email and nonempty password can proceed
in this demo.

The second step contains a six-digit verification-code input, a Verify button, and a Back button.
Show the entered email and a small demo hint with the code from `references/demo.json`. Accept only
that code. Show an inline error for incomplete or incorrect codes. Back returns to the first step
and keeps the email. Clear the code and its error when leaving the code step.

A correct code replaces the form with a signed-in success message that includes the email. This is a
local UI demo. Do not add real authentication, network calls, or persistence after a page reload.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
