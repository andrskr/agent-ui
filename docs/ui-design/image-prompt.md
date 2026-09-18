# Design board prompt

Tool: built-in image generation. Output: `views-v1.png`.

This board sets the layout direction. Its example values and small text are illustrative. The
implementation must use saved measurements and exact application labels.

```text
Use case: ui-mockup
Asset type: one high-fidelity UI design board for a real Rust Ratatui terminal app named AGENT UI.
Primary request: design one coherent terminal UI system with SIX screens on a single board, arranged 2 columns by 3 rows. Screen captions outside each frame: "01 GROUP OVERVIEW", "02 COMPARE", "03 TASK OVERVIEW", "04 TASK ACTIVITY", "05 TASK SETUP", "06 RUN SETTINGS".
Style: precise monospace terminal UI, dark charcoal background, mint accent, restrained gray dividers, green ready status, generous but practical spacing, crisp typography. Match an existing terminal app, not a web dashboard: no rounded cards, gradients, charts, icons, device mockup, or decorative graphics. High-resolution landscape design board, readable text.
Shared shell: top app label AGENT UI without global action buttons. Left sidebar Groups with Smoke parent and indented Baseline, Context; current group or task highlighted. Right content has context title, one tab strip, one compact action row, content. Consistent footer text "↑↓ Select   Enter Tasks   Esc Back   ←→ / Tab Views   ? Help".
Group tabs: "Overview" and "Compare". Task tabs: "Overview", "Activity", "Setup". Same tab row position on all screens. Never nest task tabs inside Compare.
01: Smoke group, Overview active, action "n Run all tasks", compact table of Baseline/Context, status, agent time, estimated cost. Summary "2 tasks · 2 ready". No repeated prose.
02: Smoke group, Compare active. Inline pair selectors "a A: Baseline" and "b B: Context", action "s Swap". Exactly two sections: "Configuration" with columns Setting, A, B (Provider codex/codex, Model luna/luna, Effort low/low, Time limit 300s/300s), and "Metrics" with columns Metric,A,B,B−A (Status Ready/Ready, Verification Passed/Passed, Agent time 26.0s/26.1s/+0.1s, Input tokens 40,657/41,298/+641, Output tokens 485/505/+20, Estimated cost <$0.01/<$0.01). No source files, no diff files, no agent assessment, no narrative reports, no Run or Compare buttons, no Activity/Setup subtabs. Tiny note "USD estimates · cached tokens are included in input".
03: Smoke / Baseline, Overview active; action row "n Run  b Preview  e Code". READY status; concise provider/model/effort line; two columns Duration and Tokens; estimated cost; short result note. No changed-file inventory or long disclaimer.
04: same Task shell and tabs, Activity active; same action row; elapsed timestamps and a short live operation log with one error example. No metric duplication.
05: same Task shell and tabs, Setup active; same action row; saved configuration (Provider, Model, Effort, Time limit), Task prompt text, optional package versions, tool versions. No raw JSON dump.
06: same shell behind a compact centered RUN ALL TASKS dialog with Provider, Model, Effort fields and keyboard hints. Button "Enter Start" and warning "Replaces saved output for these tasks."
Constraints: all information must fit a monospace grid, consistent alignment, tab position and navigation. No code artifacts or generated source file lists in comparison. Use concise plain English labels. Sample numbers are illustrative. This is a design reference board, not a claim of measured results.
```
