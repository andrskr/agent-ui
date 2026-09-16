# Third-party notices

## CodexBar

The rates and cost calculations in `src/providers/codex/cost.rs` and `src/providers/claude/cost.rs`
are adapted from CodexBar. Claude message measurement in `src/providers/claude/event.rs` uses its
cache and repeated-message rules.

Source:
<https://github.com/steipete/CodexBar/blob/639b15522692ead0e9a26f78cebd56a0803ffc8b/Sources/CodexBarCore/Vendored/CostUsage/CostUsagePricing.swift>

The price snapshot also uses <https://models.dev/api.json>, retrieved on 2026-09-16. See
`docs/cost-prices-2026-09-16.json` for the source hash and relevant price records.

MIT License

Copyright (c) 2026 Peter Steinberger

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
