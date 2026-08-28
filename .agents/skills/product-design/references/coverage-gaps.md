# Coverage gaps

What has no standard yet. To name a gap is progress. An unnamed gap gets a different answer from
every person who hits it. This file is the roadmap for the skill.

Every reference `SKILL.md` routes to exists. The finer grain does not yet: exemplars, recorded
decisions, and some surface specifics. This is expected, not a fault. Do not stop when guidance is
missing. Do not warn the user that the skill is incomplete. Degrade gracefully instead.

When you find no standard for what you build, fall back in this order:

1. The rules in `SKILL.md`.
2. `astryx docs <topic>` and `astryx docs principles`.
3. The closest design-system template (`astryx template --list`).
4. General interface heuristics.

This order covers a missing standard. When sources conflict, use the Decision authority list in
`SKILL.md`.

Build the surface. Then, if you set a new standard, write the missing file. The next person inherits
it.

Three kinds of gap live here: missing references, repository gaps, and design-system gaps. The other
sections record status, not gaps.

## Missing references

None. Every reference `SKILL.md` routes to exists.

Two planned references were retired instead of written, per the no-duplication rule:

- `component-guide`: the Astryx CLI already answers "which component". `astryx template --list`
  descriptions carry the "what"; `surfaces.md` carries the "when".
- `design-guidelines`: `astryx docs` topics carry the token and system rules; `interface-quality.md`
  carries the craft judgment on top.

## Surface guidance

`surfaces.md` covers the judgment layer for the surfaces we expect to meet: navigation, table or
list, form, settings, modal, empty state, first use, motion, and charts. The compositions themselves
stay in the design system on purpose. Discover them through the Astryx CLI; the commands and the
no-copy rule live in `surfaces.md` ("Discover the building blocks first").

Split a surface into its own file only when its section outgrows `surfaces.md` with judgment the CLI
cannot carry.

## Supporting material

- Skill-local `AGENTS.md`. No file. The repository `AGENTS.md` at the root and
  `apps/playground/AGENTS.md` cover the chain today.
- `exemplars/`. Empty. Add one worked before-and-after for each pattern as we build.
- `references/decisions/`. `TEMPLATE.md` exists. No decisions recorded yet; add one file per
  accepted decision.

## Repository gaps

A repository-level blocker: a lint rule with no supporting module, a missing shared contract, or a
build constraint. Record each one here with the date you verified it.

None recorded yet.

## Design-system gaps

What the design system could not express. Add one entry each time you must use a `style` or
`className` escape hatch. This list is what we take back to the design-system team.

```markdown
## <what you tried to build>

- File: <path>
- Wanted: <the interface behavior>
- Tried: <the props and compositions that failed, and why>
- Used: <the escape hatch>
- Ask: <the component or prop the design system needs>
```

## Sticky (frozen) columns in children-mode Table

- File: `apps/playground/src/service-jobs/service-jobs-table.tsx`
- Wanted: freeze 0-2 columns at the start and/or end edge of a `Table`, with the frozen columns
  offset by the pixel width of whatever precedes them, while composing the table in children mode
  (manual `TableHeader`/`TableBody`/`TableRow`/`TableCell`) because the selection checkbox column,
  sortable headers, and collapsible group-header rows all needed hand-rolled markup already.
- Tried: `useTableStickyColumns` — it returns a `TablePlugin`, which only applies in data-driven
  mode (`data`/`columns` props); it has no effect on manually-rendered `TableRow`/`TableCell`
  children. `xstyle` — StyleX styles are compiled statically from `stylex.create()`; the sticky
  offset is a per-column pixel number computed at render time from the active column order and the
  user's sticky-edge count, which no static class can express.
- Used: the `style` prop directly on `TableHeaderCell`/`TableCell` (`position: sticky`, computed
  `left`/`right` in px), which the design system's own styling docs document as a supported escape
  hatch ("style is merged after StyleX inline styles, so consumer values win on conflict"). Silenced
  `oxlint-plugin/no-style-escape-hatches` for the file with a comment pointing here.
- Ask: either let `useTableStickyColumns` (and `useTableGroupedRows`/`useTableSelection`) apply to
  children-mode rows too, or ship a children-mode-friendly way to compute sticky offsets (e.g. an
  exported `useStickyColumnOffsets(columns, stickyStart, stickyEnd)` hook returning per-key
  `CSSProperties`) so a manually-composed table doesn't need a raw `style` prop for this.
