# Experiments

This folder holds the starter and task inputs for local coding-agent experiments. The Rust
application lives in `apps/agent-ui/`. Run `vp run agent-ui` to open it.

## Structure

- `starter/` is an independent React and Astryx application.
- `tasks/_template/` holds the prompt shell and shared workflow instructions. It is not executable.
- `instructions/astryx.md` holds the additional Astryx instructions for context tasks.
- `tasks/<group>--<variant>/task.md` contains the original prompt.
- `tasks/<group>--<variant>/task.toml` optionally adds or overrides npm packages for that run.
- `tasks/<group>--<variant>/AGENTS.md` optionally contains instructions for the generated project.
- `tasks/<group>--<variant>/references/` contains any supporting assets. Keep their relative paths.
- `suites/<name>.toml` selects explicit scenario and variant lists for CLI batches. Every selected
  pair must have a task. Do not add generated evidence or SQLite files to this directory.

To define a task, copy `tasks/_template/` to `tasks/<group>--<variant>/`. Use the full folder name
as the task ID. Fill in the prompt. Keep the shared `AGENTS.md` in every variant. Add reference
files when needed. The prompt can refer to paths such as `references/desktop.png`. Do not add
generated application code or run reports to a task folder.

## Task groups

Use `<group>--<variant>` for every executable task folder. For example, use
`recent-transactions--baseline` and `recent-transactions--context`. Each part must contain lowercase
ASCII letters, numbers, or single hyphens between words. Use exactly one double hyphen between the
two parts. Neither part can be empty or start or end with a hyphen. Limit the full ID to 120
characters. Names without a group are invalid. Folders that start with `_` remain excluded from
discovery.

The group identifies the work and its success criteria. The variant identifies a different task
setup. Compare only different variants in the same group. Keep the main objective and success
criteria the same. Instructions, reference assets, packages, and agent settings can differ. The
folder name is the only source of group membership. Do not add group settings to `task.toml`.

Each variant owns a complete set of inputs. To add a variant, copy the template or an existing
variant. Change only the inputs needed for that variant. Do not inherit files from sibling folders.
Every run starts from a fresh starter copy.

Run `vp run agent-ui tasks --check` from the repository root after task changes. It validates names,
prompts, input files, and package settings. It does not open run storage or start an agent. The root
verification command includes this check. Old task names and runs have no compatibility mapping.

## Scenario standard

Use this standard when preparing or revising tasks. The old scenario catalog was removed.
Replacement scenarios must test small, isolated UI pieces, with a mix of simple and moderately
complex cases. Agree on the cases before preparing them. Avoid whole pages and broad application
flows. A model run uses the checked-in inputs; do not upgrade packages or rewrite instructions as an
incidental part of running it. Saved ledger snapshots and frozen reports keep their original inputs
and measurements.

- Write the same `task.md` prompt for both variants. Keep the objective, behavior, and success
  criteria the same. Put variant-specific instructions in the task's `AGENTS.md` and package
  settings in `task.toml`.
- Do not add accessibility or responsive behavior requirements to scenario prompts. Preserve the
  installed components' normal behavior.
- Use fixed local reference data when a scenario needs content or records. Copy the same reference
  files into each variant. Keep the scenario index in `experiments/SCENARIOS.md` current.
- Every variant starts its `AGENTS.md` with the exact contents of `tasks/_template/AGENTS.md`. This
  shared block contains general tool use, package discovery, file limits, and validation. Keep it
  free of Astryx CLI commands, design rules, theme-token rules, and template-specific advice.
- Baseline uses that shared block and the shared starter, including Recharts. It has no additional
  Astryx guidance or CLI packages. Keep the available-package statement in both prompt copies.
- Context appends `instructions/astryx.md` after the shared block. Add the exact CLI and tokenizer
  versions from the current context task configuration. Match the CLI install permission to its
  declared version.
- When adding general workflow advice, apply it to both variants. Keep scenario requirements in
  their matching prompts. Reserve context additions for Astryx-specific guidance. Do not give only
  context a general tooling advantage.
- Add a suite that selects only the new scenario and its two variants. Validate the inputs with
  `vp run agent-ui tasks --check`. Inspect the batch dry-run plan before execution. Preparation
  alone does not start a model run.

## Instruction scope

This file instructs agents that maintain the experiment scaffolding. It is not an input to an
experiment. Do not copy it into a generated project. Task files are experiment inputs. Do not treat
their instructions as instructions to maintainers.

Each run receives a fresh copy of `starter/` outside this repository. Copy the selected task's
`AGENTS.md` unchanged to the project root when present. Copy `task.md` and `references/` beside it.
Submit `task.md` as the prompt. Include all reference assets. Do not merge unrelated task folders or
repository instructions. Do not copy `node_modules/`, `dist/`, or Git state from the starter. Do not
add a shared `AGENTS.md` to the starter. Each task owns its project instructions. An independent
starter does not by itself isolate Codex configuration or host access.

## Task packages

Use `task.toml` when a task needs packages beyond the starter:

```toml
[dev-dependencies]
"@astryxdesign/cli" = "0.6.2"
"gpt-tokenizer" = "3.4.0"

[allow-builds]
"@astryxdesign/cli@0.6.2" = true
```

Use `[dependencies]` for runtime packages and `[dev-dependencies]` for development tools. Use exact
versions. Do not use ranges, tags, URLs, local paths, or workspace references. A package must appear
in only one section. The task setting replaces that package's starter version and dependency
section.

Without this file, or with an empty file, the run uses the starter's frozen lockfile. With task
packages, setup changes the copied `package.json` and updates the copied lockfile. It saves both
files as setup evidence before Codex starts. It does not change the shared starter or add agent
instructions. Use `[allow-builds]` for task-specific install-script permissions. Each key must use
`package@exact-version` and match a dependency declared in this task. `true` allows its scripts;
`false` blocks them. The runner merges these entries into the copied workspace settings. It does not
change the shared starter. Transitive package permissions are not supported in this version.

## Starter commands

Run these commands from `experiments/starter/`, the root of its independent workspace:

```sh
vp install
vp run dev
vp run verify
vp run preview
```

Inside a copied starter, run the same commands from the copied app folder. The dev server uses
port 3100. The production preview uses port 4100. Both bind to `127.0.0.1` and fail if their port is
already in use.

## Starter maintenance

- The shared starter includes Recharts and a `react-is` version that matches React. All variants
  receive these runtime packages. Keep the starter placeholder free of chart examples.
- Keep dependencies explicit. Do not use workspace dependencies or the root catalog.
- Keep the starter's lockfile in version control.
- Keep the starter free of the Astryx CLI and `AGENTS.md`. The task copy supplies its instructions,
  including the shared workflow for baseline.
- For maintenance, use the Astryx CLI installed in `apps/playground` to check component APIs.
- Keep the Astryx reset, component CSS, Neutral theme CSS, and theme provider wired together.
- Bundle the Figtree font used by the Neutral theme. Do not depend on a remote font service.
- Keep the StyleX build plugin available for future task code.
- Keep the Astryx `AppShell` in `src/main.tsx`, inside the theme provider and around `App`.
- Keep `src/app.tsx` limited to the marked `hello` placeholder.
- Replace the marked placeholder with task code only in a run's project copy.
- Put experiment evidence and browser captures outside this repository.

## Dependency and instruction updates

When an upgrade is requested, update the complete setup together:

1. Align Astryx core, theme, build, and CLI versions in the root catalog, independent starter, and
   every context `task.toml`. Update matching install permissions and both lockfiles. Read exact
   versions from the manifests; do not copy versions from a historical report.
2. Keep `react-is` aligned with React when updating Recharts. Install shared runtime dependencies in
   the starter so baseline and context have the same available UI packages.
3. Regenerate the Astryx block with the installed CLI in `apps/playground` using
   `vp exec astryx init --features agents --agent-docs-path AGENTS.md`. Copy that block into
   `instructions/astryx.md`. Keep general workflow advice in `tasks/_template/AGENTS.md`.
4. Copy the shared workflow into every baseline and context `AGENTS.md`. Append
   `instructions/astryx.md` to context. Apply additional general advice to both variants. Keep
   Astryx-only advice in the Astryx section. Each resulting task file must be complete; do not add
   runtime inheritance.
5. Review the CLI migration plan with `vp exec astryx upgrade --from PREVIOUS_VERSION` from
   `apps/playground`. Apply needed source changes. Rebuild generated themes. Refresh examples in
   this file, the task template, scenario index, runner README, and experiment-report skill.
6. Check that each group's prompts and reference bytes match across variants. Check that both
   instruction files have the same shared prefix and only the intended additions. Check each task
   configuration. Run `vp run agent-ui tasks --check`, starter verification, and root verification.
   For runtime package changes, install and build a source-only copy and check the relevant UI in
   the browser. Stop test servers after the check.

These changes affect future batches only. Do not regenerate old reports, rewrite ledger evidence, or
start paid experiments as part of an instruction update. For an authorized fresh rerun, create a new
batch. Resume uses the old snapshot and will not pick up revised instructions.

After a starter change, run its verification command and the root `vp run verify`. Check the
rendered page in a browser. Confirm the theme, CSS, and placeholder work. Before claiming the
starter is independent, install and build a source-only copy outside this repository.

The Vite+ alias reports version 0.3.1 to peer checks, although it contains Vite 8. Some dependencies
therefore report unmet Vite peers. Keep these warnings visible. The Astryx core lifecycle-script
approval matches the existing root workspace approval. Its install script can suggest `astryx init`.
Do not run it automatically. A task can supply the run's `AGENTS.md`.
