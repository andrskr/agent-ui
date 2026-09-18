# Browser checks

Batch: `1789737046184-1834816a-52a9-41f2-88d1-12e8b6450542`.

The screenshots use 1440 × 1000 CSS pixels and scale 2. Baseline and context show the initial This
week state. The generated source was not changed during capture.

Baseline and context loaded. Switching to Last week changed the cards to 84 tickets created, 78
resolved, and 3.1 hours. Dates, bar values, category counts, and percentages also changed. The bar
scale stayed fixed. Each bar tooltip showed the day and count. The initial state was restored before
capture. The baseline capture was repeated after chart rendering was complete.

The repair dev page is blank. Its browser console reported:

```text
Error: Unexpected 'stylex.create' call at runtime. Styles must be compiled by '@stylexjs/babel-plugin'.
    at src/support-overview.tsx:26:23 (served module)
```

`context-plus-repair.png` records this blank page. Its interactions could not be checked. The source
contains a `stylex.create` call in `src/support-overview.tsx`. The browser error alone does not
establish whether the fault is in the generated source or the dev build configuration.

All three runs passed automated checks, including the recorded repair check. A passed build does not
establish that the page renders. These browser observations do not change the saved verification
results and are not a visual quality score. All capture servers were stopped.

## Later diagnosis

The repair configuration template used a shallow object spread. Vite+ adds plugins to the quality
configuration. That plugin list replaced the starter's StyleX and React plugins. The browser then
received an uncompiled `stylex.create` call.

The template now uses `mergeConfig`. A separate copy of this run rendered in both dev and production
with that configuration change alone. The generated source was unchanged. The period selector also
worked, and the production browser console had no errors. The copied app and repository verification
checks passed.

The saved run, metrics, report, and original screenshots remain unchanged. This was a configuration
check, not a new experiment run.
