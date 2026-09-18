# Experiments

This folder holds the starter and task inputs for local coding-agent experiments. The Rust
application lives in `apps/agent-ui/`. Run `vp run agent-ui` to open it.

## Structure

- `starter/` is an independent React and Astryx application.
- `tasks/_template/` is an empty task template. It is not an executable task.
- `tasks/<group>--<variant>/task.md` contains the original prompt.
- `tasks/<group>--<variant>/task.toml` optionally adds or overrides npm packages for that run.
- `tasks/<group>--<variant>/AGENTS.md` optionally contains instructions for the generated project.
- `tasks/<group>--<variant>/references/` contains any supporting assets. Keep their relative paths.
- `suites/<name>.toml` selects explicit scenario and variant lists for CLI batches. Every selected
  pair must have a task. Do not add generated evidence or SQLite files to this directory.

To define a task, copy `tasks/_template/` to `tasks/<group>--<variant>/`. Use the full folder name
as the task ID. Fill in the prompt. Remove `AGENTS.md` for a bare Astryx task. Add reference files
when needed. The prompt can refer to paths such as `references/desktop.png`. Do not add generated
application code or run reports to a task folder.

## Task groups

Use `<group>--<variant>` for every executable task folder. For example, use
`invite-member--baseline`, `invite-member--context`, and `invite-member--repair`. Each part must
contain lowercase ASCII letters, numbers, or single hyphens between words. Use exactly one double
hyphen between the two parts. Neither part can be empty or start or end with a hyphen. Limit the
full ID to 120 characters. Names without a group are invalid. Folders that start with `_` remain
excluded from discovery.

The group identifies the work and its success criteria. The variant identifies a different task
setup. Compare only different variants in the same group. Keep the main objective and success
criteria the same. Instructions, reference assets, packages, and agent settings can differ. The
folder name is the only source of group membership. Do not add group settings to `task.toml`.

Each variant owns a complete set of inputs. To add a variant, copy the template or an existing
variant. Change only the inputs needed for that variant. Do not inherit files from sibling folders.
Every run starts from a fresh starter copy. A `repair` name does not start a second agent pass or
reuse another run's output.

Run `vp run agent-ui tasks --check` from the repository root after task changes. It validates names,
prompts, input files, and package settings. It does not open run storage or start an agent. The root
verification command includes this check. Old task names and runs have no compatibility mapping.

## New scenario standard

Use this standard when the user asks to prepare a new baseline/context/repair scenario. Keep
existing scenario inputs and saved reports unchanged unless the user asks to revise them.

- Write the same `task.md` prompt for all three variants. Keep the objective, behavior, and success
  criteria the same. Put variant-specific instructions in the task's `AGENTS.md` and package or
  check settings in `task.toml`.
- Do not add accessibility or responsive behavior requirements to new scenario prompts. Preserve the
  installed components' normal behavior. This rule does not revise existing scenario inputs.
- Use fixed local reference data when a scenario needs content or records. Copy the same reference
  files into each variant. Keep the scenario index in `experiments/SCENARIOS.md` current.
- Baseline uses the starter without added agent instructions or context packages.
- Context includes the Astryx project guidance and its required packages. Use the current context
  task as a reference for exact package versions and install permissions. Check that the guidance
  matches the selected package version. Copy all required inputs into the new task folder.
- Repair includes the same context inputs, the `root-quality` setup profile, and
  `[repair] check = "quality"`. Append the repair instructions below to its own `AGENTS.md`.
- Add a suite that selects only the new scenario and its three variants. Validate the inputs with
  `vp run agent-ui tasks --check`. Inspect the batch dry-run plan before execution. Preparation
  alone does not start a model run.

Copy this block into each new repair task's `AGENTS.md`. These are instructions for the experiment
agent, not commands to run while preparing the scenario:

```md
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
```

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
"@astryxdesign/cli" = "0.5.4"

[allow-builds]
"@astryxdesign/cli@0.5.4" = true
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

- Keep dependencies explicit. Do not use workspace dependencies or the root catalog.
- Keep the starter's lockfile in version control.
- Keep the baseline starter free of the Astryx CLI and shared agent instructions.
- For maintenance, use the Astryx CLI installed in `apps/playground` to check component APIs.
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
therefore report unmet Vite peers. Keep these warnings visible. The Astryx core lifecycle-script
approval matches the existing root workspace approval. Its install script can suggest `astryx init`.
Do not run it automatically. A task can supply the run's `AGENTS.md`.

## Setup profiles and repair

A task can select reusable setup profiles and require an in-pass check:

```toml
[setup]
profiles = ["root-quality"]

[repair]
check = "quality"
```

Profiles live in `experiments/profiles/<name>/profile.toml`. They declare exact package versions,
install permissions, file copies, and named checks. Profile sources are relative to the repository
root. Destinations are relative to the run workspace. `replace = true` is required to replace a
starter file. Conflicts between profiles are errors. Exclusions accept a relative file or directory,
or `**/*.suffix`. Links and paths outside the source root are not allowed. Task prompts,
`AGENTS.md`, reference assets, package manifests, and runner files are reserved destinations. Each
task still owns its complete prompt and instructions. Profiles do not inherit sibling tasks.

The root-quality profile copies the shared Vite+ policy and the custom Oxlint source package. It
adds the exact plugin dependency. The profile configuration composes this policy with the starter
build configuration. It maps the application source path to `src`; rule settings stay the same.

Repair is opt-in. The runner adds `vp run repair` and a required-check instruction to the submitted
prompt. The task's AGENTS.md remains a separate, unchanged input. The command requests the named
check from the runner. The runner checks a source snapshot with saved dependencies, without network
access. It returns diagnostics to the same agent session. No second agent pass starts. A failed
check must be fixed and run again. A successful first check is valid. Direct `vp check` or
`vp run verify` commands do not satisfy the recorded repair requirement.

Setup runs the complete check before the agent starts. Repair fails setup if the unchanged starter
cannot pass. Source and public assets can change during the task. Files outside `src/` and `public/`
must keep their setup hashes. The final source must match the latest passing repair check. The
runner then checks it again and copies the verified build to `app/dist`. Missing checks, failed
checks, changed setup files, and source changes after a pass prevent Ready. Git staging and hooks
are not required.

The first repair command sandbox uses macOS Seatbelt. Other platforms reject repair during task
validation. Baseline and context tasks keep their current provider behavior. Check logs, command
results, source hashes, and preflight/final results are saved under `evidence/repair`. In-pass check
time is part of total agent time. The controller uses a private dependency copy and an empty check
home. Keep run storage outside the agent's writable workspace and temporary roots.
