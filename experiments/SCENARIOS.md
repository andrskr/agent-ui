# Scenario index

The experiment plan has 12 scenarios. Each has baseline, context, and repair variants. The first
three have frozen reports from earlier inputs. Support dashboard needs a fresh run after the
dependency and instruction update; its earlier run evidence remains in the ledger, and its generated
apps and report files were removed. The other eight scenarios have prepared inputs only. Preparation
does not start a model or create a ledger record.

Select one scenario by its suite name. The prompt links below point to the baseline copy. Each new
scenario has the same prompt and reference files in all three variants. This also applies to the
three older scenarios.

| #   | Scenario and prompt                                                          | Main UI coverage                                                              | Suite                      | Agent timeout per task         |
| --- | ---------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | -------------------------- | ------------------------------ |
| 1   | [Invite member](tasks/invite-member--baseline/task.md)                       | Form validation and submission                                                | `ui-evaluation`            | Keep the requested run setting |
| 2   | [Project list](tasks/project-list--baseline/task.md)                         | Table, search, filtering, and creation dialog                                 | `project-list`             | 1800 seconds                   |
| 3   | [Notification preferences](tasks/notification-preferences--baseline/task.md) | Settings, dependent controls, and saved state                                 | `notification-preferences` | 1800 seconds                   |
| 4   | [Support dashboard](tasks/support-dashboard--baseline/task.md)               | Metric cards, bar chart, donut chart, and period changes                      | `support-dashboard`        | 1800 seconds                   |
| 5   | [Documentation reader](tasks/documentation-reader--baseline/task.md)         | Grouped navigation, long content, section navigation, and code examples       | `documentation-reader`     | 1800 seconds                   |
| 6   | [Command palette](tasks/command-palette--baseline/task.md)                   | Search overlay, nested selection, and recent commands                         | `command-palette`          | 1800 seconds                   |
| 7   | [Media attachment manager](tasks/media-attachment-manager--baseline/task.md) | File input, thumbnail grid, progress, retry, selection, and preview           | `media-attachment-manager` | 2700 seconds                   |
| 8   | [Order detail workspace](tasks/order-detail-workspace--baseline/task.md)     | Master-detail layout, line items, totals, and event timeline                  | `order-detail-workspace`   | 1800 seconds                   |
| 9   | [Advanced data table](tasks/advanced-data-table--baseline/task.md)           | Combined filters, multiple sort fields, columns, pagination, and bulk actions | `advanced-data-table`      | 3600 seconds                   |
| 10  | [Simulated AI chat](tasks/simulated-ai-chat--baseline/task.md)               | Streaming, stop, retry, regenerate, Markdown, and scroll state                | `simulated-ai-chat`        | 3600 seconds                   |
| 11  | [Work board](tasks/work-board--baseline/task.md)                             | Dragging, ordering, moving cards, undo, and detail panel                      | `work-board`               | 3600 seconds                   |
| 12  | [Automation rule builder](tasks/automation-rule-builder--baseline/task.md)   | Dynamic condition rows, typed values, rule preview, and saved drafts          | `automation-rule-builder`  | 3600 seconds                   |

The timeouts are starting limits for future runs, not duration estimates. The larger limits allow
time for the more complex interactions and repair checks. Suite files select tasks only; they do not
store a timeout, model, or effort. Supply the timeout on the CLI. An explicit user setting overrides
these limits. Dashboard is the next planned scenario. The user can select any prepared scenario
independently.

## Prepared inputs

Every task contains `task.md`. Tasks can also contain `task.toml` and reference files. The nine
later scenarios include `references/data.json`; media attachment tasks also contain three PNG
fixtures. Fixture values and image bytes are identical across variants. The prompts tell the
experiment agent to copy needed reference files into `src/`.

- All variants use the shared starter with Astryx and Recharts. Package versions are pinned in
  `starter/package.json`. `react-is` matches React.
- All three variants have the same general workflow in their `AGENTS.md`. This includes tool use,
  component API checks, complete diagnostics, source limits, and validation.
- Baseline has only those general instructions. Its `task.toml` is absent or contains only a
  comment. It has no Astryx-specific instructions or CLI packages.
- Context appends the Astryx guidance and adds matching CLI packages.
- Repair adds the same context inputs plus the `root-quality` profile and required `quality` check.
  Its instructions require `vp lint src --fix`, `vp fmt src --write`, and `vp run repair`.

All current prompts omit accessibility and responsive behavior requirements. All repair tasks use
the current workflow, including the three older scenarios. Frozen reports and ledger snapshots keep
their original inputs. Each task owns complete inputs; it does not load a sibling task's files.

## Select and validate a run

Use the experiment-report skill when the user asks for a model run and report. Select the exact
model at that time. Omit `--effort` to use the runner's local model default unless the user requests
another value. `--effort default` instead asks the provider to choose.

These commands validate inputs and show a plan without starting a provider:

```sh
vp run agent-ui tasks --check
vp run agent-ui batch run --suite support-dashboard --provider claude --model MODEL_ID --timeout 1800 --dry-run
```

Replace `MODEL_ID` with the selected available model. Use the selected scenario's suite and timeout.
Only when execution is requested, replace `--dry-run` with `--record` and keep the same arguments.
Each of these suites selects exactly three tasks. There is no new suite that runs all scenarios.

Keep the execution summary's batch ID for reporting. A task run replaces its current generated
artifacts but keeps ledger history. Save approved reports and requested screenshots under
`reports/SCENARIO/MODEL-EFFORT/` using the experiment-report skill's rules. For two models, finish
the first model's report and requested screenshots before starting the next model. The next batch
replaces the same tasks' generated apps.
