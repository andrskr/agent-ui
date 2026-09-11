# Experiments

This folder holds the starter and task inputs for local coding-agent experiments. The runner and the
task commands are not implemented yet.

## Structure

- `starter/` is an independent React and Astryx application.
- `tasks/_template/` is an empty task template. It is not an executable task.
- `tasks/<task-id>/task.md` contains the original prompt.
- `tasks/<task-id>/AGENTS.md` contains the instructions for the generated project.
- `tasks/<task-id>/references/` contains any supporting assets. Keep their relative paths.

To define a task, copy `tasks/_template/` to `tasks/<task-id>/`. Use the folder name as the task ID.
Fill in the prompt and the project instructions. Add reference files only when the task needs them.
The prompt can refer to paths such as `references/desktop.png`. Do not add generated application
code or run reports to a task folder.

## Instruction scope

This file instructs agents that maintain the experiment scaffolding. It is not an input to an
experiment. Do not copy it into a generated project. Task files are experiment inputs. Do not treat
their instructions as instructions to maintainers.

Each future run receives a fresh copy of `starter/` outside this repository. Copy the selected
task's `AGENTS.md` unchanged to the project root. Copy `task.md` and `references/` beside it. Submit
`task.md` as the prompt. Include all reference assets. Do not merge unrelated task folders or
repository instructions. Do not copy `node_modules/`, `dist/`, or Git state from the starter. Do not
add a shared `AGENTS.md` to the starter. Each task owns its project instructions. An independent
starter does not by itself isolate Codex configuration or host access.

## Starter commands

Run these commands from the repository root:

```sh
vp -C experiments/starter install
vp -C experiments/starter run dev
vp -C experiments/starter run verify
vp -C experiments/starter run preview
vp -C experiments/starter exec astryx component Text
```

Inside a copied starter, use the same commands without `-C experiments/starter`. The dev server uses
port 3100. The production preview uses port 4100. Both bind to `127.0.0.1` and fail if their port is
already in use.

## Starter maintenance

- Keep dependencies explicit. Do not use workspace dependencies or the root catalog.
- Keep the starter's lockfile in version control.
- Use the installed Astryx CLI to check component APIs and setup instructions.
- Keep the Astryx reset, component CSS, Neutral theme CSS, and theme provider wired together.
- Bundle the Figtree font used by the Neutral theme. Do not depend on a remote font service.
- Keep the StyleX build plugin available for future task code.
- Keep the Astryx `AppShell` in `src/main.tsx`, inside the theme provider and around `App`.
- Keep `src/app.tsx` limited to the marked `hello` placeholder.
- Replace the marked placeholder with task code only in a run's project copy.
- Put experiment evidence and browser captures outside this repository.

After a starter change, run its verification command and the root `vp run verify`. Check the
rendered page in a browser. Confirm the theme, CSS, and placeholder work. Before claiming the
starter is independent, install and build a source-only copy outside this repository.

The Vite+ alias reports version 0.3.1 to peer checks, although it contains Vite 8. Some dependencies
therefore report unmet Vite peers. Keep these warnings visible. The two Astryx lifecycle-script
approvals match the existing root workspace approvals. The Astryx CLI can suggest `astryx init`
because the starter has no shared agent instructions. Do not run it automatically. The task supplies
the run's `AGENTS.md`.
