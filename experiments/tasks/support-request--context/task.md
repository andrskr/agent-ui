Build a compact two-column support request form using the installed Astryx components.

Put the fields on the left and a live request summary on the right. Group the fields under Contact
and Request details. Contact has Name and Email. Request details has Subject, Category, Priority,
and Description. Categories are Account, Billing, and Technical issue. Priorities are Low, Normal,
and High. Start with empty text fields, Account selected, and Normal priority.

The summary shows the current name, email, subject, category, and priority. Use “Not provided” for
empty text values. Below it, show “Our team usually replies within one business day.”

Add a Submit request button. Require all text fields and a valid email address. Show inline errors
for invalid fields after submission. Preserve entered values on an invalid submission. On a valid
submission, show “Request submitted” with a read-only summary and a Submit another request button.
That button returns to the initial empty form.

Build only this form and its summary. Keep state local. Do not add navigation, attachments, network
calls, or persistence.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
