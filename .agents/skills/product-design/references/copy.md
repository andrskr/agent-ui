# Copy

Rules for the words in the product. Use this file when you write or edit any user-facing string: a
button, a label, an empty state, an error, a form, a tooltip, a placeholder, or an accessible name.
Edit the copy in context, not as isolated strings.

Two goals hold. Say the true thing in the fewest words. Sound like the product, not a template. The
"AI tells" section is the anti-slop core. One tell marks copy that a model wrote without thought.
Remove every one.

Contents: Scope, Voice, Banned words, Concision, AI tells, Edit in context, Interface copy,
Placeholders, Accessibility and localization, Longer text, Headings and lists, Terms and links,
Emphasis, Tone by moment, Pricing and limits, Verify copy, and Anti-patterns (the review checklist).

## Scope

- This file governs user-facing product copy.
- When the design-system docs and this file disagree on a term, the design system wins.

## Voice

- Write in active voice when the sentence has an actor. Test it: add "by monkeys" to the end. If the
  sentence still parses and the actor matters, rewrite it.
- Address the reader as `you`. Do not write `the user` or `one`.
- Use the imperative for an action. Write "Add a project", not "You can add a project".
- Write in the present tense. Use the future tense only for future behavior.
- Contractions are fine in product copy and warm writing.
- Limit `we`. Use `we` only for a deliberate first-party action, such as "we recommend". Never use
  `we` to mean `you`.
- Do not ask a rhetorical question. It reads as marketing.
- Keep a sentence under 20 words.
- Second-read test: read each sentence once, at speech pace. If you must read it again to parse it,
  name the subject, the action, and the result, then rewrite.

## Banned words

- `easy`, `simple`, `quick`. They pressure the reader and read as marketing. Replace each with a
  concrete fact: "one command", "the default", "most projects skip this".
- `very`, `just`, `really`, `simply`. They are filler. Cut them or rewrite the sentence.

## Concision

- Earn every detail. Cut a number, a name, or a detail when a general phrase would not change what
  the reader understands or does.
- Replace a weasel word with a specific claim. Do not write `significantly`, `many`, `often`,
  `typically`, or `generally`. Give the number.
- Replace a vague quantifier with a figure. Do not write `near-zero` or `most requests`. Give the
  value.
- Replace a metaphor verb with the literal step. Do not write `moves through`, `lands`, `carries`,
  or `hits`. Name the action.

## AI tells

These mark copy that a model wrote without thought. Each one is a defect. Remove every one.

- Summary transition. Do not open a paragraph by recapping the last one. Do not write "With this
  setup complete…" or "Now that we have explored…". Go straight to the next point.
- Stop-start fragments. Do not split one idea into choppy sentences. Turn "Previously this was
  manual. Now it is automatic. This saves time." into one sentence. A short sentence for emphasis is
  fine.
- Spec-sheet voice. Do not write like a datasheet. Rewrite `provides`, `is configurable`, and
  `is explicitly labeled`.
- Cold open. Do not start a body paragraph with a sentence that works as a heading. Carry the prior
  subject forward. Start with "Because…" or "Once…".
- Personified artifact. A machine does not perform a human action. Write "the browser fetches the
  URL", not "hand the browser a URL". Write "the token is stored", not "the token holds the value".
- Reused framing. The angle comes from this page, not a template. Do not write "The question most
  teams face is whether…".

## Edit in context

Read the entire interaction path, not one string. Find ambiguous verbs and nouns, internal jargon,
vague states, missing consequences, and inconsistent terms.

Set the message hierarchy for each state. Decide in order:

1. the one fact the user needs now;
2. the action available next;
3. the context that changes the decision;
4. the tone for the moment.

Say each idea once. If the heading already explains the state, make the intro add new information,
or delete the intro.

## Interface copy

Actions and navigation:

- Name the object, the scope, and the consequence of an important action. Do not write "Are you
  sure?".
- Use a specific verb and object when the outcome is not obvious.
- Label the outcome, not the gesture. Write "Delete project", not "Click here".
- Use the same noun and verb for one concept across the product.
- Label a destructive action with a verb and a noun. The undo-versus-confirmation judgment lives in
  `product-judgment.md`. When you must confirm, name the action on both the message and the button.
  Do not use `Confirm`, `OK`, `Yes`, `No`, or `Submit`.
- Give every control a full action name. An icon-only control still needs a label and an accessible
  name.
- Name the condition that enables a disabled control. Do not leave the reader to guess.

Forms:

- Use a persistent label. A placeholder is an example, not a label.
- State format and eligibility rules before submission, not after.
- Explain why you request information only when the reason is not obvious.
- Treat required and optional fields consistently.
- Write validation as what needs attention and how to fix it. Do not blame the user. Keep the
  message near the field. Announce the error accessibly.

Errors and permissions:

- Answer three things: what failed; why, when known and useful; and how to recover, or what
  alternative remains.
- Do not show an internal code as the main message.
- Do not promise a cause or a fix the system cannot know.
- Treat privacy, payment, deletion, access loss, and blocked work seriously. Warmth is welcome. A
  joke is not.

Loading, empty, and success:

- Name the real operation in a loading label. Set an honest expectation when the wait matters.
- End a loading label with an ellipsis: "Loading…", "Saving…".
- Keep a control label stable during loading. Use the component's busy state, not a new word.
- Give each empty-state variant its own message and next action. The variants are mapped in
  `resilience.md`.
- Show real absence. For an empty value, write "Not provided" or the real state. Do not show a blank
  slot.
- Write an empty state as a next step, not an apology.
- Confirm the outcome in a success message. Add the next consequence only when it changes what the
  user does. Keep routine success brief.

Help and instructional text:

- Write helper text that answers an implicit question. Do not restate the control.
- Use progressive disclosure for uncommon detail.

## Placeholders

- Text placeholder: use `snake_case` and describe the value, such as `your_access_token_here`. The
  reader can double-click to select it before pasting.
- Number placeholder: count up, such as `1234567890123`. It reads as fake and predictable.
- Never use `<TOKEN>`, `xxx`, or a generic ALL_CAPS token. `your-token` fails because it is generic;
  `your_access_token_here` names the value.

## Accessibility and localization

- Keep the voice consistent. Let the tone adapt to the moment.
- Use plain language. Do not flatten a term the audience genuinely knows.
- Write a complete, translatable message. Do not build a message from concatenated fragments.
- Keep variables and numbers structured so a translator can reorder them.
- Allow the text to expand. Do not abbreviate early.
- Make alt text carry the image's information. Use empty alt for decoration.
- Keep a screen-reader name aligned with the visible label and the outcome.
- Do not rely on punctuation, color, or an icon alone to carry the message.
- Keep a short terminology glossary when one concept drifts across the product.
- Do not vary a word for literary effect in an interface.

## Longer text

Some surfaces carry more than a line: a help panel, an onboarding step, an empty-state body, or a
settings description.

- Lead with the point. Put the key sentence first, not last.
- Front-load each sentence. State the outcome or the action, then the detail.
- Keep a paragraph to two to four sentences. Split a longer one, or one that covers two ideas.

## Headings and lists

- Write a page or section heading in sentence case: "Configure environment variables".
- Write a navigation label in title case: "Configuring Environment Variables".
- Make a heading descriptive. The reader guesses the content from the heading alone. Do not use a
  single generic word such as "Overview" or "Notes".
- Convert three or more list-shaped items to a list.
- Use a bullet list for an unordered set. Use a numbered list for a sequence.
- Introduce a list with a colon.
- End a list item with a period only when it is a full sentence.
- For a term and its description, use `- **Term**: description`.

## Terms and links

- Spell out an acronym on first use: "Content Security Policy (CSP)".
- Define a term the first time you use it. Link to its reference.
- Make the link text name the destination. Never use a bare URL. Never use "here" or "click here".

## Emphasis, punctuation, and units

- Use **bold** for a UI element or a critical fact. Do not use bold for tone. If you reach for bold
  to add weight, the sentence is weak. Rewrite it.
- Use `inline code` for a path, a file extension, an identifier, or a short snippet. If it looks
  wrong in a normal font, use code.
- Do not use an em dash or a hyphen as punctuation. Use a colon, a comma, or a period. Or rewrite
  the sentence.
- Use curly quotes in rendered copy: the paired double and single marks, not the straight marks.
- Use the ellipsis character `…`. Do not use three dots.
- Use a non-breaking space inside a unit or a shortcut, such as `10&nbsp;MB` and `⌘&nbsp;K`.
- Use `&` instead of "and" only where space is tight, such as a navigation label or a button.
- Write a data size with a non-breaking space and the unit's standard symbol: `64 KB`, `200 ms`.
  Write seconds bare: `30s`. Keep the format the same across the product so the reader can scan.

## Tone by moment

Keep the voice consistent. Match the tone to the moment:

- Routine action: calm and terse. The user wants to finish a task, not read.
- Onboarding or first use: warm and encouraging. Set no traps.
- Error or failure: plain and direct. Name the fix. Do not apologize, and do not joke.
- Destructive or high-risk action: serious and exact. Name the object and the consequence.
- Success: brief. Confirm the outcome, then step out of the way.
- Help or explanation: clear enough that the user can repeat it. Use an example.

## Pricing and limits

- Give the full detail. Too much is better than too little.
- Use a table.
- Never assume the reader knows the pricing model or the unit of billing.

## Verify copy

Read the flow in context and test:

- comprehension with no hidden product knowledge;
- a clear next action at each error, empty state, and decision point;
- factual accuracy and consistent terms;
- scanning at the target width and at 200% zoom;
- long names, translation expansion, plurals, and dynamic values;
- accessible names and announced state changes;
- tone that fits the consequence.

The final copy is as short as it can be without losing meaning or recovery.

## Anti-patterns

Flag each of these on sight. This is the review checklist for Copy mode and Review mode. It mirrors
the rules above: when you change a rule, change its checklist entry too.

Language and voice:

- A banned word: `easy`, `simple`, `quick`.
- A filler word: `very`, `just`, `really`, `simply`.
- Passive voice where the actor matters (the "by monkeys" test).
- `we` standing in for `you`.
- A rhetorical question.
- A weasel word instead of a specific claim: `significantly`, `many`, `often`, `typically`,
  `generally`.
- A vague quantifier without a figure: `near-zero`, `sub-second`, `most requests`.
- A metaphor verb instead of the literal step: `moves through`, `lands`, `carries`, `hits`.

AI tells:

- A summary transition that recaps the last paragraph, such as "With this setup complete…".
- Stop-start fragments that split one idea into choppy sentences.
- Spec-sheet voice: `provides`, `is configurable`, `is explicitly labeled`.
- A cold-open paragraph whose first sentence has no antecedent.
- A personified artifact that performs a human action, such as "hand the browser a URL".
- Reused template framing, such as "The question most teams face is whether…".

Interface copy:

- `Confirm` or `OK` on a destructive action, instead of a verb and a noun.
- An icon-only control with no label and no accessible name.
- A disabled control with no stated reason.
- A placeholder used as the only label.
- A blank slot instead of "Not provided" or the real state.
- An empty state written as an apology, or one that does not name which empty it is.
- An error that blames the reader, or shows an internal code as the main message.
- Invented or fake loading progress.
- `Loading...` instead of `Loading…`.
- A message built from concatenated fragments that cannot translate.
- A word varied for literary effect in an interface.

Mechanics:

- Title case in a page or section heading.
- A generic subheading: `Overview`, `Caveats`, `Notes`.
- Bold used for emphasis, not for a UI element or a critical fact.
- A generic placeholder: `<TOKEN>`, `xxx`, `your-token`, `ABC123`.
- An em dash or a hyphen used as punctuation.
- A straight quote instead of a curly quote.
- Three dots instead of the ellipsis character.
- A bare unit such as `64KB` or `200MS`, instead of `64 KB` or `200 ms`.
- An acronym used before it is spelled out.

Structure and links:

- A paragraph over four sentences, or one that covers two ideas.
- A bare URL, or "here" or "click here", as link text.
- A sentence that needs a second read to parse.
