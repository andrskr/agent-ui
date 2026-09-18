# AGENTS.md

Project-specific guidance for AI coding agents.

## Experiment workflow

- Work from the generated app directory. Read `task.md`, `package.json`, and the task's reference
  files before editing. Keep changes within the paths allowed by the task. Do not add dependencies.
- Recharts is installed for charts. Import its chart components from `recharts`. Use it when the
  task requests a chart. Do not add unrelated features.
- If you use example code, check its imports and dependencies first. Functions defined inside an
  example are local helpers, not necessarily installed component APIs. Adapt only the parts needed
  for the task.
- Read each needed component's props and import information. Reuse that information. If a prop is
  unclear, read the installed type declaration. Do not assume that two components accept the same
  props.
- Run one operation per tool call. Use the tool's working directory when available. If the tool
  already starts in the app directory, do not add `cd`. Use Read for package metadata and type
  declarations. Avoid shell loops and `node -e` for discovery.
- Read complete diagnostics. Do not pipe checks through `head`, `tail`, or `grep`. If a command is
  blocked, use the permitted tool for the same operation. Do not repeat the blocked command.
- Scope lint fixes and formatting to `src/`. Do not use `vp check --fix`, unsafe fixes, or change
  runner configuration. Run `vp fmt src --write`, then `vp run verify`. Fix source errors and repeat
  the checks after edits.

## Astryx integration

- Astryx provides the UI components. Recharts provides chart components; do not search for them with
  `astryx component`. Use theme tokens for chart colors. Chart coordinates, dimensions, angles, and
  data values do not require design tokens.
- Astryx templates are references. Use a skeleton to inspect a layout. Do not copy a whole page.
  Template helpers such as `MetricCard` are local functions, not Astryx component APIs.

<!-- ASTRYX:START -->

Astryx v0.6.2 · 164 components CLI: run every command as `pnpm exec astryx <cmd>` (shown below as
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

MORE CLI: search "<query>" find any component / hook / doc / template / block component --list 164
components by category template --list page + block recipes docs <topic> browser-support,
cli-integrations, color, elevation, getting-started, icons, illustrations, internationalization,
layout, migration, motion, principles, shape, spacing, styling-libraries, styling, theme, tokens,
typography, working-with-ai swizzle <Name> eject component source for deep customization upgrade
--apply run after any Astryx or integration dependency bump
<!-- ASTRYX:END -->
