# Scenario index

The new plan will contain 12 small, isolated UI cases. Eight cases are prepared. All eight cases
have completed Opus 4.8 and Sonnet 5 reports. The remaining four cases have not been selected. The
previous catalog and results were cleared on 18 September 2026.

| #   | Scenario                                                           | UI coverage                                                                             | Suite                 | Agent timeout per task |
| --- | ------------------------------------------------------------------ | --------------------------------------------------------------------------------------- | --------------------- | ---------------------- |
| 1   | [Recent transactions](tasks/recent-transactions--baseline/task.md) | Eight-row table, merchant icons, status labels, amounts, search, and status filter      | `recent-transactions` | 1800 seconds           |
| 2   | [Share document](tasks/share-document--baseline/task.md)           | Dialog, initial avatars, role selectors, access selection, and copy-link feedback       | `share-document`      | 1800 seconds           |
| 3   | [Weekly activity](tasks/weekly-activity--baseline/task.md)         | Two-series line chart, weekly total, segmented filter, legend, and tooltips             | `weekly-activity`     | 1800 seconds           |
| 4   | [File attachments](tasks/file-attachments--baseline/task.md)       | Drop zone, file rows, validation, simulated upload progress, retry, and removal         | `file-attachments`    | 1800 seconds           |
| 5   | [AI chat panel](tasks/ai-chat-panel--baseline/task.md)             | Message layout, avatars, text composer, simulated reply, and waiting state              | `ai-chat-panel`       | 1800 seconds           |
| 6   | [Payout settings](tasks/payout-settings--baseline/task.md)         | Currency selector, linked number input and slider, notes, validation, and save feedback | `payout-settings`     | 1800 seconds           |
| 7   | [Team members](tasks/team-members--baseline/task.md)               | Table, combined filters, sorting, row selection, bulk deactivation, and role menus      | `team-members`        | 1800 seconds           |
| 8   | [Support request](tasks/support-request--baseline/task.md)         | Two-column form, grouped fields, live summary, validation, and submission state         | `support-request`     | 1800 seconds           |

Each scenario has baseline and context variants with identical prompts and reference data. Use the
shared starter and the instruction rules in [AGENTS.md](AGENTS.md). Do not add accessibility or
responsive behavior requirements. Agree on each new case before preparing it.

The timeout is an upper limit, not an expected duration. "You know the drill" runs the selected
tasks, saves reports, and captures each task's initial UI. Stop all capture servers afterward. Do
not perform manual UI interactions or behavior tests unless separately requested. A report-only
request does not include screenshots.

Use the experiment-report skill. Run models sequentially. Save reports under
`reports/SCENARIO/MODEL-EFFORT/report.html`. Do not start a model until the user requests a run.
Keep failed results in the ledger and report; do not retry without authorization.

Recent transactions: [Opus 4.8 high](../reports/recent-transactions/opus-4-8-high/report.html) and
[Sonnet 5 high](../reports/recent-transactions/sonnet-5-high/report.html). All four attempts passed
automated checks. Initial-state screenshots are saved beside both reports. No manual interaction
tests were performed.

Share document: [Opus 4.8 high](../reports/share-document/opus-4-8-high/report.html) and
[Sonnet 5 high](../reports/share-document/sonnet-5-high/report.html). All four attempts passed
automated checks. Initial-state screenshots are saved beside both reports. No manual interaction
tests were performed.

Weekly activity: [Opus 4.8 high](../reports/weekly-activity/opus-4-8-high/report.html) and
[Sonnet 5 high](../reports/weekly-activity/sonnet-5-high/report.html). All four attempts passed
automated checks. Initial-state screenshots and capture notes are saved beside both reports. The
Opus captures omit the connecting chart lines. The Sonnet context capture clips the left edge of one
axis label. No manual interaction tests were performed.

File attachments: [Opus 4.8 high](../reports/file-attachments/opus-4-8-high/report.html) and
[Sonnet 5 high](../reports/file-attachments/sonnet-5-high/report.html). All four attempts passed
automated checks. Initial-state screenshots and capture notes are saved beside both reports. No
manual interaction tests were performed.

AI chat panel: [Opus 4.8 high](../reports/ai-chat-panel/opus-4-8-high/report.html) and
[Sonnet 5 high](../reports/ai-chat-panel/sonnet-5-high/report.html). All four attempts passed
automated checks. Initial-state screenshots and capture notes are saved beside both reports. No
manual interaction tests were performed.

Payout settings: [Opus 4.8 high](../reports/payout-settings/opus-4-8-high/report.html) and
[Sonnet 5 high](../reports/payout-settings/sonnet-5-high/report.html). All four attempts passed
automated checks. Initial-state screenshots and capture notes are saved beside both reports. No
manual interaction tests were performed.

The `team-members-and-support` suite selects only cases 7 and 8 with both variants.

Team members: [Opus 4.8 high](../reports/team-members/opus-4-8-high/report.html) and
[Sonnet 5 high](../reports/team-members/sonnet-5-high/report.html). All four attempts passed
automated checks. Initial-state screenshots and capture notes are saved beside both reports. No
manual interaction tests were performed.

Support request: [Opus 4.8 high](../reports/support-request/opus-4-8-high/report.html) and
[Sonnet 5 high](../reports/support-request/sonnet-5-high/report.html). All four attempts passed
automated checks. Initial-state screenshots and capture notes are saved beside both reports. No
manual interaction tests were performed.
