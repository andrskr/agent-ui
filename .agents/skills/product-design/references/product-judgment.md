# Product judgment

The decision before the pixels. Load this for Shape, Implement, Harden, a full Review, or any
material product or flow change. Working code answers "does it run". This file answers "should it
exist, and in this form".

This file covers the brief, the questions, and the trade-offs. For the words, see `copy.md`. For the
visual finish, see `interface-quality.md`. For the reachable states, see `resilience.md`.

## Write the brief

Write a compact internal brief before you choose a surface or a component. Keep it small: three to
five bullets when the task is settled; the full structure for an ambiguous, multi-surface, or
flow-level change. Do not restate the conversation.

- **User**: who acts, in what situation and state of mind.
- **Job**: what they try to accomplish, in their words, not the feature's name.
- **Current behavior**: what the product does today. Verify it in code or on the rendered surface.
- **Desired outcome**: the behavior after the change.
- **Success signal**: what would show the change worked.
- **Non-goals**: what this change does not try to solve.
- **Object**: the product object the action touches.
- **Scope**: how far the action reaches — this item, the project, or the account.
- **Action**: what the system will change.
- **Consequence**: what the user gains or loses, and what they must understand before they act.
- **Reversibility**: whether the user can undo it, and how far the undo reaches.
- **Permissions**: who can see or do this, and what everyone else sees instead.
- **Open decisions**: the product choices the brief cannot settle. Name each one. Never hide one
  inside an implementation detail.

If a field reads like a mood, the decision is not made yet. Each field must survive the question
"how do you know?".

## Ask what changes the result

Ask only when a material field is unknown and the answer would change what you build.

- Ask the two or three questions that most change the result, then wait. One round is the default.
- Assert the likely reading and invite correction. Do not turn an obvious fact into a menu, and do
  not dump a questionnaire.
- A precise request may need only a compact confirmation. A sparse request earns one round.
- Never ask for pixel values or component names. Those decisions are this skill's job.
- When no one can answer, do not stall. Mark each assumption plainly, proceed, and list the
  assumptions in the output.

Questions that earn their round:

- What problem must this surface solve, and for whom?
- What is the one thing the user must understand or do here?
- What real content and data must it carry — the minimum, the typical, and the maximum?
- What must remain untouched? What would make a polished result feel wrong?
- Which constraints bind: platform, performance, accessibility, localization?

## Find the essence

- Each surface has one primary user goal. Find it before you add anything.
- Separate necessary from nice-to-have. Ask what 20 percent delivers 80 percent of the value.
- Simplicity removes obstacles between the user and the goal. It does not remove features, and it
  does not remove information a decision needs.
- Match complexity to the task. A complex domain deserves a structured surface, not a hollow one.
- Mystery is not minimalism. A surface so bare that the next step is unclear failed the same test as
  a cluttered one.

## Choose the smallest coherent intervention

Work down this ladder. Stop at the first rung that solves the job.

1. A better default. The user does nothing and gets the right outcome.
2. A behavior change inside the existing surface.
3. Reuse of an existing surface or pattern.
4. New UI.

- Remove, hide, or combine before you add. Can the flow lose a step?
- Use progressive disclosure: keep the default path simple, keep the advanced path reachable.
- Prefer a strong default over a new setting. A setting transfers your unmade decision to every
  user, forever.
- Be ready to cut a good idea to keep the primary path clear. When you cut scope, record why, and
  where it could return.
- An addition inside an established surface inherits that surface: its layout, its patterns, its
  conventions. Never turn a local addition into a redesign.

## Weigh consequence and reversibility

- Name the exact object, scope, and consequence in the interface. "Delete project Alpha and its 14
  deployments", not "Are you sure?".
- Match friction to impact. A routine, reversible action gets a direct path. An action that is hard
  to reverse gets a proportional guard.
- Prefer undo over confirmation when the system can honestly support it. A confirmation interrupts
  everyone; an undo costs only the user who erred.
- The worst moment and the last moment define how the user remembers the flow. Spend reassurance at
  the high-stakes moments, and end on a clear outcome.

## Compare real alternatives

For a material decision, compare two or three genuinely different approaches, not variants of one.

- Judge each on two axes: does the user recognize their task, and is the outcome clear.
- Losing to the simpler option is a valid outcome, and the common one.
- Trace the winner to evidence: product behavior, canonical guidance, an accepted decision, or a
  verified adjacent pattern. Taste alone does not decide.

## Walk the task as other users

Walk the primary task through two or three of these lenses. Pick the ones that fit the surface.

- **The expert**: skips instructions, wants keyboard paths and batch actions, abandons a slow or
  patronizing flow.
- **The first-timer**: reads every label literally, hesitates before an unfamiliar control, breaks
  on jargon, abandons rather than figures it out.
- **The assisted user**: keyboard and screen reader only. A visual-only signal does not exist for
  them.
- **The stress tester**: edge data, refresh mid-flow, the back button, the same record in two tabs.
- **The interrupted mobile user**: one thumb, a slow connection, leaves mid-flow and returns later.

Report the exact element that fails a lens, not a generic concern. To set severity, ask: would this
user contact support about it? If yes, it is at least P1.

## Verify the decision

Before you implement, answer for each non-mechanical change:

- What user problem does this solve?
- Why this surface, and why this component?
- What consequence must the interface communicate?
- What evidence supports the decision?
- What is the smallest coherent change?

After you implement, check the outcome, not the output: the primary task got faster or clearer, the
necessary features stayed reachable, and every assumption and open decision is listed in the
handoff.
