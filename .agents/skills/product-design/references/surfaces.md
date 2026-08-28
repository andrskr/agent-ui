# Surfaces

Per-surface judgment: when a surface fits the job, and what it must communicate. This file does not
list components or compositions. Those live in the design system and change with it. Discover them;
do not expect them here.

For the words on each surface, see `copy.md`. For the visual finish, see `interface-quality.md`. For
the reachable states, see `resilience.md`.

## Discover the building blocks first

Before you build any surface, check what the design system already ships. Run the CLI through the
project's package runner, from the package that has the design system installed:

- `astryx docs` lists the reference topics. `astryx docs <topic>` prints one.
- `astryx template --list` lists page and block templates with descriptions. Filter with
  `--type page` or `--type block`.
- `astryx template <id> --skeleton` prints a template's layout structure with spacing annotations.
  Read it to learn the composition without injecting code.
- `astryx template <id> <path>` injects the template as a starting point. Give the path relative to
  the project root.
- The `<id>` is a kebab-case or PascalCase identifier, not the display name, and `--list` does not
  print it. If you pass a wrong name, the error prints every valid id.

Start from the closest template and adapt it. Do not rebuild a composition the system already
solved. And do not copy the catalog into a skill file: the CLI is the source of truth, and a copy
starts to rot the day it is written.

## Navigation

- Always show location: an active state, a breadcrumb, or both.
- The label the user clicks must match the title of the page they land on.
- Group items by the user's model of the product, not by the team that built each part.
- Keep the visible items at one level to about five. Group the rest under a clear category.
- Two levels of depth are comfortable. A structure deeper than that needs search, or a clear signal
  of what each branch holds.

## Table or list

- Choose a table when the user compares items across attributes. Choose a list when the user scans
  items and acts on one. Do not choose a card grid because it fills space.
- Match density to use frequency: a daily tool earns density, an occasional flow earns room.
- Make the sort order and every active filter visible. An active filter changes what "empty" means.
- Keep one or two row actions visible. Put the rest in a menu. Show bulk actions only when a
  selection exists.
- Right-align numbers and use tabular numerals so columns can be compared by eye.

## Form

- Ask only for what the outcome needs. Collect the rest later, in context.
- Prefill what the system already knows. Default what most users pick.
- When a field is unusual, say why you ask.
- Validate a field when the user leaves it, not on the first keystroke. After an error, revalidate
  as the user types the correction.
- Group fields by meaning. In a long flow, give each step one decision.
- Name the submit action after the outcome: "Create project", not "Submit".

## Settings

- A setting stores a standing preference. A one-off choice belongs in the flow, not in settings.
- Every setting is a decision you moved to the user. Justify each one against the ladder in
  `product-judgment.md` ("Choose the smallest coherent intervention") before adding it.
- Group settings by user goal, not by internal system structure.
- Show the current value on the row.
- Make the save model explicit and consistent per surface: changes apply immediately, or a save
  action exists. Never mix the two silently.
- Put irreversible or account-wide settings in their own region, with guards proportional to their
  blast radius.

## Modal

A modal spends the user's attention by force. It has two honest jobs: protect focus for a short
task, or make a consequence unavoidable. Anything else works better inline.

- Prefer inline disclosure first. A flow of several steps needs a page, not a modal.
- Never stack a modal on a modal.
- Name the primary action with its verb and object: "Delete project", never "OK" or "Yes".
- "Cancel" must return the world unchanged, every time.
- For a destructive confirm, name the object, the scope, and the consequence (see
  `product-judgment.md`). Reserve type-to-confirm for the largest blast radius.
- Escape, the scrim, and focus return must behave the same across every modal in the product.

## Empty state

An empty state is not a gap. It is the surface teaching itself. Every empty state carries:

1. What will appear here.
2. Why it is worth having.
3. One clear action to get started.

Add an illustration or icon only when it earns its place. Read `astryx docs illustrations`.

Match the tone to the variant (the variants are mapped in `resilience.md`):

- First use: sell the value and offer a template.
- Cleared by the user: use a light touch, not a sales pitch.
- No results: suggest a different query, or clearing the filter.

## First use and guidance

- Get the user to the first moment of real value as fast as possible. Do not teach the product;
  prove it is worth their time.
- Teach a feature at the point of use, not in an upfront tour. Empty states are the primary
  onboarding surface.
- Make every tour and hint skippable, and never show a dismissed hint again.
- Guide the user to accomplish something real, with real functionality, not a detached tutorial
  mode.
- Do not patronize. Assume the user can figure out standard patterns.

## Motion

For the craft rules, see the Motion section in `interface-quality.md` and `astryx docs motion`. This
section decides when to animate at all. Motion has five jobs:

- acknowledge an action;
- make a state change legible;
- preserve continuity through navigation or layout change;
- direct attention at a meaningful moment;
- carry the product's one authored moment.

A static area is not a reason to animate. Take every duration and easing from the motion tokens;
`astryx docs motion` prints them and the bands they serve. Long feedback reads as latency.

## Charts and data display

- Every chart answers one question the user actually asks. Name the question before choosing the
  chart.
- A number without a comparison is decoration. Show the change, the target, or the peer.
- Label data directly when the series are few. A legend is a lookup cost.
- A chart has loading, empty, and error states like any surface. A dashboard of skeletons must not
  shift when data lands.
- Do not reach for the hero-metric template by reflex; `interface-quality.md` lists it under Refuse.
