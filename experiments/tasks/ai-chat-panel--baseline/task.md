Build a compact “Writing assistant” chat panel using the installed Astryx components.

Start with this conversation:

User: “Help me write a short project update.”

Assistant: “Start with what changed, what comes next, and any blockers.”

Show distinct user and assistant messages with small avatars.

Add a text composer and Send button. On send, append the message and show “Assistant is typing…” for
one second. Then append this fixed reply: “The first draft is complete. Next, we will review it with
the team. There are no blockers.”

Clear the composer after sending. Disable Send when the input is empty or a reply is pending. Keep
the latest message visible.

Build only this chat panel. Use local state and simulated replies. Do not add navigation,
conversation history, attachments, network calls, or persistence.

Replace the marked task code region in `src/app.tsx` and keep the markers. You can add supporting
files under `src/`. Recharts is already installed for tasks that need charts; this task needs none.
Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
