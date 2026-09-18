# Scenario index

The previous scenario catalog was removed on 18 September 2026. No executable tasks or suites are
currently prepared. The local ledger and generated reports were cleared.

The next plan will contain 12 small, isolated UI cases. Each case must test a distinct UI piece. Use
a mix of simple and moderately complex cases. Avoid whole pages and broad application flows. Agree
on the cases with the user before writing prompts, task configurations, or suites.

Keep baseline, context, and repair prompts equivalent. Use the shared starter and the instruction
rules in [AGENTS.md](AGENTS.md). Do not add accessibility or responsive behavior requirements.

The normal workflow only runs the selected tasks and generates reports. Do not start preview
servers, capture screenshots, or perform manual UI interactions unless the user explicitly asks for
that work for the current run. Do not start a model until the user requests a run.
