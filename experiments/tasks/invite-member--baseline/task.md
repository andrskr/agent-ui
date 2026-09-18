Build a form that invites a person to a workspace. Put it in the marked task code region of
`src/app.tsx` and keep the task code markers.

This project uses the Astryx design system. Build the interface from the components it installs.
Decide yourself which ones fit.

Recharts is already installed. Use it for any charts requested by this task.

The form must:

- show a title and one line of help text
- take an email address
- let the user choose one role: Admin, Editor, or Viewer
- have a send button and a cancel button
- clear the form and any messages when the user selects Cancel
- show an error message under the email field when the address is not valid
- keep the send button unavailable until the email is valid and a role is chosen
- show a progress state while it sends, then a confirmation message
- keep the form controls unavailable while it sends

Hold the state in the component. Do not call a network service. Use a one second delay for the send.
Do not change any other application file.

In your final response, state what you built and how you checked it.
