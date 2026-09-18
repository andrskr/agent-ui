# Browser checks

Batch: `1789739587045-fc8e2bca-3f0a-4994-bc29-3f2a75d89373`.

All three runs passed automated verification. The screenshots use a 1440 × 1000 CSS viewport and
scale 2. They show the initial Workspace overview with all sidebar groups expanded. The reading
areas scroll independently; a full-page capture does not expand their hidden content. The generated
source was not changed.

Baseline and repair were checked and captured from their dev servers. The context dev page stayed
blank while module loading remained incomplete, including after one server restart. No browser
exception was reported. The saved context production build loaded without source changes. Its
screenshot and interaction checks use that production preview. The cause of the dev loading issue
was not established.

For each rendered page, article selection updated the content and section list. A collapsed Guides
group stayed collapsed when another article was selected. Command reference showed three code
examples. Clicking the first Copy action changed only that action to Copied. Clipboard contents, the
exact reset delay, and the copy-failure path were not verified. Section navigation was exercised.
Next opened Troubleshooting, with no Next control there. Previous returned to Command reference.
Reload restored Workspace overview. The tested pages had no browser console errors.

The report generator reported no data warnings. Local-file browser inspection of the HTML report was
blocked by the browser URL policy. No alternate browser route was used to bypass that block. All
preview servers started for these checks were stopped, and their ports were checked.
