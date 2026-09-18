# Scenario index

The new plan will contain 12 small, isolated UI cases. One case is prepared and has completed Opus
4.8 and Sonnet 5 reports. The remaining eleven cases have not been selected. The previous catalog
and results were cleared on 18 September 2026.

| #   | Scenario                                                           | UI coverage                                                                        | Suite                 | Agent timeout per task |
| --- | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------- | --------------------- | ---------------------- |
| 1   | [Recent transactions](tasks/recent-transactions--baseline/task.md) | Eight-row table, merchant icons, status labels, amounts, search, and status filter | `recent-transactions` | 1800 seconds           |

Each scenario has baseline and context variants with identical prompts and reference data. Use the
shared starter and the instruction rules in [AGENTS.md](AGENTS.md). Do not add accessibility or
responsive behavior requirements. Agree on each new case before preparing it.

The timeout is an upper limit, not an expected duration. "You know the drill" runs the selected
tasks, saves reports, and captures each task's initial UI. Stop all capture servers afterward. Do
not perform manual UI interactions or behavior tests unless separately requested. A report-only
request does not include screenshots.

Use the experiment-report skill. Run models sequentially. Save reports under
`reports/recent-transactions/MODEL-EFFORT/report.html`. Do not start a model until the user requests
a run. Keep failed results in the ledger and report; do not retry without authorization.

Reports: [Opus 4.8 high](../reports/recent-transactions/opus-4-8-high/report.html) and
[Sonnet 5 high](../reports/recent-transactions/sonnet-5-high/report.html). All four attempts passed
automated checks. Initial-state screenshots are saved beside both reports. No manual interaction
tests were performed.
