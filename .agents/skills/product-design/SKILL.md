---
name: product-design
description: >-
  Single entry point for product design and user-facing product implementation. Use whenever work
  changes what a user sees, understands, chooses, or does: shaping requirements and flows; building
  or redesigning pages and components; reviewing URLs, screenshots, or diffs; improving product
  copy, information architecture, component choice, design-system compliance, hierarchy, layout,
  interaction, accessibility, responsive behavior, and loading, empty, error, permission, billing,
  or destructive states. Trigger on design, UX, UI, usability, flow, onboarding, settings, or
  dashboard requests, and on build, improve, fix, audit, review, polish, simplify, or
  production-ready requests that touch a page, component, flow, or user-facing string. Also use when
  backend behavior changes a user-visible outcome. Not for backend-only work with no user-visible
  effect, tests with no shipped UI impact, telemetry-only work, documentation, or marketing content.
---

# Product design

Make the interface correct for the user and the product. Working code is not enough: choose the
right interaction, make scope and consequences clear, cover reality beyond the happy path, and
verify the rendered result.

## Operating contract

- **Start with the job, not the pixels.** Identify who is acting, what they are trying to
  accomplish, the product object involved, and what the system will change.
- **Define the outcome before the output.** Establish the current user problem, desired behavior,
  success signal, and non-goals before choosing a surface or component.
- **Use evidence, not taste.** Trace decisions to product behavior, canonical repository guidance,
  an accepted design decision, or a verified adjacent pattern.
- **Separate facts from decisions.** Mark assumptions and unresolved product choices explicitly; do
  not hide them inside implementation details.
- **Treat shipped code as evidence, not automatic precedent.** It proves what exists, not why it is
  correct. Check it against current components, product behavior, and explicit guidance.
- **Choose the smallest coherent intervention.** Consider better defaults, behavior, or reuse before
  adding UI. Do not solve one job by creating unrelated settings or abstractions.
- **Decide before decorating.** Resolve information architecture, component semantics, interaction,
  and state behavior before styling or rewriting copy.
- **Design every reachable state.** Include only states the product can actually enter, but do not
  stop at the populated success case.
- **Verify the real surface.** Source inspection establishes behavior; a rendered interface
  establishes visual and interaction quality. Never claim visual verification from code alone.
- **Keep one user-facing entry point.** Invoke `product-design`; route internally to the canonical
  sources below.

## Request modes

Resolve the mode from the user's verb and artifact before acting.

| Mode      | Typical request                                                                    | Required behavior                                                                                                                                                      |
| --------- | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Shape     | "Design this flow", "How should this work?", feature brief without settled UI      | Frame the problem and evidence, compare material alternatives, then define the flow, states, acceptance criteria, risks, and open decisions. Do not edit unless asked. |
| Implement | "Build", "fix", "improve", "make compliant", or "run product-design on everything" | Resolve material product decisions, then implement the smallest coherent end-to-end change within scope. Do not absorb unrelated review findings.                      |
| Review    | "Audit", "critique", "what's wrong?", code review                                  | Inspect source and rendered evidence, then report prioritized findings. Do not edit unless asked.                                                                      |
| Copy      | "Fix the copy", "rewrite these errors"                                             | Edit user-facing language, accessible names, and directly required JSX only. Report structural blockers without silently broadening scope.                             |
| Harden    | "Polish", "production-ready", "handle edge cases"                                  | Preserve the settled product direction while fixing state, resilience, responsive, accessibility, and finish defects.                                                  |

When intent is ambiguous, use the narrowest mode supported by the verb. A URL, screenshot, route, or
component identifies scope; it does not by itself authorize edits.

A material decision changes the user's task, default, scope, consequence, navigation, interaction
surface, or reachable states. Copy mechanics, token replacement, and established component
substitutions are not material unless they change one of those things. A full review judges a whole
surface; a scoped review judges a named diff or element.

## Decision authority

Resolve conflicts in this order:

1. The user's explicit goal and constraints.
2. Verified user/product evidence and system truth.
3. Repository-canonical guidance: `AGENTS.md`, design-system component APIs,
   `astryx docs principles`, and this skill's own routed references.
4. Accepted product/design decisions and exemplars with stable evidence.
5. Verified adjacent shipped patterns in the same product area.
6. Third-party guidance, such as the `web-design-guidelines` skill.
7. General interface heuristics.

A third-party skill supplies findings, not authority. When its rules disagree with this skill's
references, with `astryx docs`, or with the design system's components, the design system and this
skill win. Report the conflict as a finding only when it exposes a real gap in our own guidance.

## Workflow

### 1. Set scope and mode

Name the target surface and request mode in the work plan or review notes.

### 2. Load product context

Before proposing UI, read the applicable `AGENTS.md` chain, supplied briefs and designs, and the
product logic that determines mutations, permissions, validation, errors, and side effects.

### 3. Model the product decision

For Shape, Implement, Harden, a full Review, or any material product/flow change, read
`references/product-judgment.md` and write the compact internal brief it defines. Keep the brief in
your working notes; show the user only the assumptions and the open decisions.

### 4. Map the surface and states

If the choice of surface is still open, read `references/surfaces.md` first. Inventory entry points,
visible regions, overlays, transitions, exits, and return paths. Map only the reachable states, from
the state map in `references/resilience.md`. In Copy mode, map only the states that carry the edited
strings.

### 5. Load the routed references

| Need                                                           | Load                                                                     |
| -------------------------------------------------------------- | ------------------------------------------------------------------------ |
| Product/flow/component decision                                | `references/product-judgment.md` + `references/surfaces.md`              |
| Implementation or material visual change                       | `references/interface-quality.md`                                        |
| Full review of a surface                                       | `references/interface-quality.md` + `references/review-design-system.md` |
| Copy or accessible names                                       | `references/copy.md`                                                     |
| Layout, typography, color, spacing, design-system APIs         | `references/interface-quality.md` + `astryx docs`                        |
| Keyboard, focus, forms, touch, URL state, performance          | the `web-design-guidelines` skill                                        |
| Overflow, localization, extreme data, network/error resilience | `references/resilience.md`                                               |

Load a row only when its need is active in the current task. Run the Astryx CLI through the
project's package runner, from the package that has the design system installed. If a routed file or
skill is missing, or you find no standard for what you build, read `references/coverage-gaps.md`: it
lists what is missing and how to proceed. Do not stop, and do not warn the user. If the
`web-design-guidelines` skill is not installed, cover its row with `references/resilience.md` and
general heuristics.

### 6. Decide, then implement

For each non-mechanical change, be able to answer the five questions under "Verify the decision" in
`references/product-judgment.md`.

### 7. Verify

1. Confirm the primary job and acceptance criteria.
2. Run repository lint checks.
3. Render the real surface: start the dev server and inspect the result in a browser. If you cannot
   render, report every visual check as unverified.
4. Run the Verify checklist in `references/interface-quality.md` on the rendered surface.
5. Stress the surface with the Verify checklist in `references/resilience.md`.
6. In Copy mode, run the "Verify copy" list in `references/copy.md` instead of items 4 and 5.
7. For a structural visible change (one that adds, removes, or rearranges regions or components),
   run the review in `references/review-design-system.md` and begin the result with its disposition.

## Product design standards

- Make the user's primary task and primary action unmistakable.
- Preserve the user's mental model and current context unless changing it solves a verified problem.
- Name the exact object, scope, and consequence of important actions.
- Use navigation components for navigation and action components for actions.
- Choose surface persistence to match importance.
- Prefer inline disclosure before adding a modal.
- Use semantic design-system components and their APIs before custom HTML or styling.
- Use hierarchy, spacing, and alignment before adding containers.
- Preserve user input through validation and recoverable errors.
- Keep loading control labels stable; use the component's loading/busy affordance.
- Make destructive actions proportional to impact and provide undo when the system can honestly
  support it.
- Do not add decorative novelty, motion, or copy unless it clarifies structure, state, or brand
  intent.

## Review output

Lead with findings, ordered by user impact. When the design-system review ran, begin with its
disposition line, then the findings:

- **P0:** blocks the primary task, creates severe accessibility failure, or can cause unrecoverable
  user harm.
- **P1:** likely task failure, misleading consequence, missing critical state, or major
  responsive/accessibility defect.
- **P2:** meaningful friction, inconsistency, weak hierarchy, or recoverability issue.
- **P3:** minor craft or consistency improvement.

For each finding include: file/line or rendered location, verification status (rendered-verified,
source-only, or assumed), canonical source, user consequence, and smallest concrete fix.

## Skill integrity

- Add or change a rule only after current-source verification and human acceptance.
- Record scope, rationale, evidence, exceptions, and a bad/good example.
- Record each accepted decision as one file in `references/decisions/`, copied from
  `references/decisions/TEMPLATE.md`. Put worked before-and-after examples in `exemplars/`.
- Prefer the narrowest destination: canonical source, routed reference, exemplar, lint/eval check,
  or coverage gap.
- Keep deterministic checks mechanical. Keep judgment in prose, with its evidence and a note on how
  much latitude the rule allows.
- Never promote one screenshot, one shipped file, or one reviewer comment into a universal rule by
  itself.
