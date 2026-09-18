Build a simulated AI chat interface for a workspace assistant.

This project uses the Astryx design system. Use its installed components. Replace the marked task
code region in `src/app.tsx` and keep the markers. You can add supporting files under `src/`.

Use the fixed greeting, suggestion prompts, response variants, and failure message in
`references/data.json`. Copy the data into `src/` before using it. This is a fake assistant. Do not
connect to a model, call an API, or add a real AI service.

## Conversation and composer

Show a chat header, a New chat action, a message area, and a multiline composer with Send. Start
with the supplied greeting and three suggestion buttons. A suggestion fills the composer without
sending. Hide suggestions after the first user message.

Disable Send for text that contains only spaces. Sending adds the trimmed user message, clears the
composer, and starts an assistant response. Distinguish user and assistant messages visually. Allow
only one active response. Disable Send and suggestion actions while it is active, but allow the user
to type the next draft in the composer.

## Simulated generation

Choose a response case from the submitted text without regard to letter case. A message containing
the word code uses the code case. A message containing the word plan uses the plan case. Other
messages use the general case. A message containing the word fail uses the failure behavior below
instead. Show a small example hint so the user can discover code, plan, and fail.

Show Thinking for 600 milliseconds, then reveal the selected response in chunks of 12 characters
every 80 milliseconds. Show a visible generation state and replace Send with Stop generating while a
response is active. These are local timers, not measured model behavior.

Stop generating cancels the pending delay or stream. Keep text already received and mark the
response Stopped. If no text arrived, show Response stopped. Do not add more text after stopping.

For fail, show Thinking and then the supplied error after 600 milliseconds. Keep the user message
and show Retry. Retry streams the general response successfully without adding another user message.
Disable Retry while a response is active.

## Response actions and content

Support the Markdown forms used in the supplied responses: paragraphs, headings, bold text, bulleted
and numbered lists, inline code, and fenced code blocks. Render text safely. No raw HTML support is
needed. Keep partial content readable while it streams.

Completed assistant messages have Copy. Code blocks have their own Copy code action. Show Copied for
two seconds after success and a failure message if copying fails.

The latest completed or stopped assistant response has Regenerate when no response is active. It
replaces that response in place, keeps its user message, and alternates between the two supplied
variants for the same case. It uses the same delay, stream, and Stop behavior. Do not branch the
conversation or duplicate the user message.

## Scroll and reset behavior

Follow incoming text while the message area is at the bottom. If the user scrolls up, preserve their
position and show Jump to latest. That action returns to the bottom and resumes following. Sending a
new message also returns to the bottom.

New chat clears messages, errors, draft text, and pending timers. Restore the greeting and
suggestions. No text from the previous chat may appear afterward. Do not add conversation history,
file attachments, model selection, or persistent storage.

Do not add dependencies or change files outside `src/`. In your final response, state what you built
and how you checked it.
