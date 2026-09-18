# Browser checks

Batch: `1789740986078-b1d81c43-38b7-4c1f-9bdc-b37d07b8ec2d`.

All three runs passed automated verification. The screenshots use a 1440 × 1000 CSS viewport and
scale 2. They show the initial Workspace overview with all sidebar groups expanded. The reading
areas scroll independently; a full-page capture does not expand their hidden content. All three
screenshots came from the dev servers. The generated source was not changed.

For each page, article selection updated the content and section list. A collapsed Guides group
stayed collapsed when another article was selected. Command reference showed three code examples.
Clicking the first Copy action changed only that action to Copied. Clipboard contents, the exact
reset delay, and the copy-failure path were not verified. Section navigation was exercised. Next
opened Troubleshooting, with no Next control there. Previous returned to Command reference. Reload
restored Workspace overview. No browser console errors were reported during these checks.

The context screenshot shows a narrow reading column with large empty outer margins. Some list items
are cut off with ellipses. The reference table is wider than the visible reading column. These are
visible output issues despite the passed automated checks.

The report generator reported no data warnings. Local-file report inspection was unavailable because
the browser URL policy blocked this report route during the Opus check. Report sections were checked
in the generated HTML instead. No alternate route was used to bypass that block. All preview servers
started for these checks were stopped, and their ports were checked.
