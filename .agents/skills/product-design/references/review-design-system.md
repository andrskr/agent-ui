# Design-system review

The last gate before handoff. Run this for a structural visible change, as the last item of the
Verify checklist in `SKILL.md`, and for a full Review. It judges whether the rendered change
expresses the design system and the product decision, or merely resembles them.

This review assumes the rest of the Verify checklist already ran: viewports, states, keyboard, and
stress are covered there and in `interface-quality.md` and `resilience.md`. This file adds what they
cannot: the system-integrity judgment and the review discipline.

Review with fresh eyes. Your memory of building the change is not evidence, and it carries your
framing and your optimism. Reopen the rendered surface and judge what is there, not what you meant.
When a separate reviewer is available, give them only the evidence, never your build reasoning. The
reviewer reports findings and edits nothing. The builder applies the fixes after the review.

## Evidence first

Before any judgment, verify the evidence exists and is valid.

- Required: the rendered surface at compact and wide widths at minimum, with every materially
  changed state exercised. For a review of an existing surface with no change, exercise the states
  of the primary task instead.
- Valid: each screenshot shows what it claims, with no blank regions and the right viewport.
- A state nobody exercised is a state nobody reviewed. Source code alone proves behavior, never
  visual quality.
- When evidence is missing or invalid, the disposition is **recapture**: name what is missing,
  gather it, and only then review. A verdict built on broken evidence launders the breakage into an
  approval.

## Classify every element

For each salient element of the change, assign one class:

- **System**: the right design-system component, used through its API.
- **Adaptation**: a deviation that cites the evidence that forced it — product truth, an
  accessibility need, or an accepted decision. An uncited deviation is a defect, not a choice.
- **Drift**: a one-off rebuild of something the system owns. Custom markup where a component exists.
  A hard-coded value where a token exists. A near-duplicate of a shipped pattern.
- **Gap**: the system could not express the need, the escape hatch is deliberate, and the gap is
  recorded in `coverage-gaps.md` under design-system gaps. An unrecorded escape hatch is drift.

Check the claim, not the import. A system component with so many overrides that it behaves like
custom markup is drift under the system's name.

## Classify each drift before fixing

Name the cause, then fix at the narrowest correct level:

- **Missing token**: the system needs a reusable value. Promote it, or record the gap.
- **One-off implementation**: a shared component or pattern should replace it.
- **Conceptual mismatch**: the flow or hierarchy differs from comparable product areas. This is a
  product finding, not a styling fix.
- **Local defect**: the implementation is incomplete or inconsistent. Fix it in place.

Never create a system abstraction for one local exception.

## Find the systemic pattern

Three findings with the same shape are one systemic finding, not three defects.

- A recurring repository blocker goes to `coverage-gaps.md` under repository gaps.
- A recurring design-system limit goes to `coverage-gaps.md` under design-system gaps.
- A recurring judgment miss means a reference file needs a rule. Propose it through the Skill
  integrity process in `SKILL.md`.

This loop is the point of the review: each systemic finding makes the next build cheaper.

## Disposition

Open the review result with one of four words. The word is derived from the findings, never felt.

- **recapture**: the evidence check failed. Nothing reviewed on it binds.
- **ship**: no drift, no missing state, no uncited deviation.
- **fix**: material findings exist (a material finding is P0 or P1). List them in the Review output
  format in `SKILL.md`, ordered by user impact, and cap the list — a pile of P3 noise buries the P1
  that matters.
- **rebuild**: drift is the page, not the exception. Do not list patches: a fix list against a
  rejected structure launders the rejection into an approval. Name the regions to rebuild and the
  system parts to build them from.

Calibrate against the bar, not against the visible effort. A surface a design reviewer would send
back is **fix** at best, even when it works. With the findings, include one **keep** line: the thing
the change got right that the fixes must not dilute.

## Score the fixes

When fixes come back, score; do not re-hunt.

- For each material finding: **resolved**, **partial**, or **unresolved**, tied to what the new
  evidence visibly shows.
- A claimed fix you cannot see in the evidence is unresolved. A fix that moved elements without
  reaching the named quality is partial at best.
- Check for regressions the fix batch itself introduced. Do not reopen other findings.
- If any material finding is unresolved or partial, do not give a ship disposition. A ship earned
  here covers the scored fixes, not the whole surface. State that limit in the result.
