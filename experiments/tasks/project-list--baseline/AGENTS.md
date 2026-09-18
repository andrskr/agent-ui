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
  runner configuration. If this file has a Required repair check, follow it. Otherwise, run
  `vp fmt src --write`, then `vp run verify`. Fix source errors and repeat the checks after edits.
