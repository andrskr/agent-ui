# View design

The interface has two navigation levels. Groups contain tasks. The tree stays visible at both
levels. Each main view has one title, one tab row, one action row, and a scrollable content area.

The [design board](views-v1.png) was made with the built-in imagegen tool. The
[exact prompt](image-prompt.md) is saved beside it. Its numbers are examples. The application uses
saved measurements. The board sets the visual direction. The rules below define the implemented
behavior.

## Views

| View           | Main question                   | Content                                                 | Actions                                      |
| -------------- | ------------------------------- | ------------------------------------------------------- | -------------------------------------------- |
| Group Overview | Which tasks have results?       | All members, status, agent time, estimated cost         | Run all; cancel active group                 |
| Group Compare  | How do A and B differ?          | Saved configuration, outcome, metrics, B minus A        | Select A; select B; swap                     |
| Task Overview  | What was the result?            | Cost, duration, tokens, verification, short agent note  | Run; cancel; preview; code                   |
| Task Activity  | What happened during the run?   | Timed events, current step                              | Same task actions                            |
| Task Setup     | What inputs and settings apply? | Saved settings, tools, packages; current prompt         | Same task actions; report and file shortcuts |
| Run settings   | What will the next run use?     | Provider, model, effort, time limit, replacement notice | Start; cancel form                           |

## Navigation

- Up and Down select groups. Enter moves into the selected group. Up and Down then select tasks.
- Esc returns from tasks to the group. From Compare, Esc returns to Group Overview.
- Left and Right switch visible tabs. Tab and Shift+Tab do the same. Navigation wraps at each end.
- Number keys select visible tabs. Mouse clicks use the same tab targets.
- Changing a group keeps Compare open when it was already open. Each group supplies its own pair.
- A task comparison opens Group Compare. Both pair selectors exclude the opposite member and all
  other groups. Missing results remain visible with a reason why they cannot be selected.
- Search accepts text until Enter or Esc. Forms use Up/Down or Tab/Shift+Tab to select a field and
  Left/Right to change its value. These controls are shown inside the form.
- Page Up, Page Down, Home, End, and the mouse wheel scroll content. The title, tabs, and actions
  stay visible. Activity follows new events until the user scrolls.

## Sidebar hierarchy

The [sidebar preview](sidebar-v2.png) uses actual Ratatui cells with sample in-memory results. Menlo
supplies the font in this preview. The user's terminal selects the actual font and size.

- Group headings use bold capitals and a full-width header band. The task count aligns right.
- Task names use regular weight. The selected task uses bold weight and a local highlight.
- Tree guides show membership without depending on colour.
- Task states align right on the name row. Dates use a quieter second row.
- Summary text and dates have less emphasis than names. Missing dates do not reserve an empty row.
- Spacing separates group metadata from tasks and separates each task from the next one.
- Use one character size. Portable terminal cells do not support a font size per row.

## Content rules

- Sidebar group rows show task counts, status counts, and the latest run start date. Task rows show
  the current status and run start date. `Prev` marks an older result while a replacement is pending
  or failed to start. Dates use local time. Missing run dates stay empty.

- Compare has no task tabs, run controls, file inventories, raw reports, previews, or agent actions.
- Compare uses saved results. It never starts an agent. A missing measurement is not zero.
- Configuration differences use the accent colour. Metric signs show B minus A. They do not claim
  that one UI is better. Cached input remains part of total input. Costs are estimates in USD.
- A group with fewer than two results has a Compare empty state. It directs the user to Overview or
  Task View to run tasks.
- Setup labels the current prompt separately from saved settings. Changed inputs have a notice.
- Errors and queued tasks remain visible. Pending runs do not reuse an old result as a new result.
- At narrow widths, metric and group tables use stacked rows. The minimum size is 76 by 24.

## Checks

In-memory tests cover tab key behavior, mouse targets, group membership, signed differences, missing
values, task-only action isolation, changed input labels, and layouts at supported sizes. Manual
checks use an isolated copy of saved results. They do not rerun the experiment tasks.
