# AGENTS.md

Project-specific guidance for AI coding agents.

<!-- ASTRYX:START -->

Astryx v0.5.4 · 163 components CLI: run every command as `pnpm exec astryx <cmd>` (shown below as
`astryx ...`).

SETUP (once, in your app entry e.g. main.tsx) — without these, components render unstyled: import
"@astryxdesign/core/reset.css"; import "@astryxdesign/core/astryx.css";

WORKFLOW — discover, don't guess. Before writing UI:

1. `astryx build "<idea>"` — START HERE: returns a kit (closest [page] + [block]s + [component]s).
   No args = full playbook.
2. `astryx template <name> [--skeleton]` — scaffold the [page]/[block]s it named, or study their
   layout. Templates are reference code.
3. `astryx component <Name>` — props + examples for every component you use.

RULES:

- No <div> — components do all layout/spacing, page frame included.
- Frame first: read `astryx docs layout` before writing any page or screen — page frame, region
  widths, breakpoint behavior.
- Dense data = rows (Table, List/Item), never Card-wrapped list items; Card is for standalone
  widgets. Status = StatusDot/Token; Badge = counts only.
- Custom styling: component props first; else the xstyle prop / StyleX tokens
  (@astryxdesign/core/theme/tokens.stylex). No raw hex/px.
- Tokens for every value (`astryx docs tokens`). Brand/accent belongs in the theme
  (`astryx theme list` / `theme add <slug>`, or `astryx theme template` for a custom one) — never
  override --color-* in :root.
- SELF-CHECK before you finish: re-read the file and replace any className=, style={{…}}, raw
  <div>/<span> layout, imported .css/@apply, or hardcoded #hex/px with the component or the xstyle
  prop + a token. If unsure a component/prop exists, run `astryx component <Name>` /
  `astryx search "<thing>"`; don't hand-roll CSS.

MORE CLI: search "<query>" find any component / hook / doc / template / block component --list 163
components by category template --list page + block recipes docs <topic> browser-support,
cli-integrations, color, elevation, getting-started, icons, illustrations, internationalization,
layout, migration, motion, principles, shape, spacing, styling-libraries, styling, theme, tokens,
typography, working-with-ai swizzle <Name> eject component source for deep customization upgrade
--apply run after any @astryxdesign/core bump
<!-- ASTRYX:END -->

## Required repair check

Use this repair workflow for this scenario:

1. After you build or edit the task UI, run `vp lint src --fix` from the app directory. Use normal
   fixes only. Read any errors that remain and fix the source. Do not enable unsafe fixes.
2. Run `vp fmt src --write` after lint fixes, even if lint reports errors. Lint fixes can change
   source formatting.
3. Run `vp run repair` as a separate command. This is the required check.
4. Read the complete diagnostics. Fix all reported source errors, then repeat these steps.
5. Finish only when repair passes for the current source. Repeat these steps after any later source
   edit. Use this same agent session.

Run one operation per tool call. Use the tool's working directory when it is available. Do not pipe
repair output through `head`, `tail`, or `grep`. Do not truncate the diagnostics.

Do not use `astryx run repair` or `vp check --fix`. Scope lint fixes and formatting to `src/` so
they do not modify runner helpers or setup files. The repair command checks a separate source copy
with the runner's saved configuration. Do not change the check configuration, dependencies, or
repair tools. A passing build alone is not sufficient.
