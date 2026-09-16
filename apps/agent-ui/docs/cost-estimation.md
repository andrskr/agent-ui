# Cost estimation review

Reviewed on 2026-09-16.

The app estimates API-equivalent token cost. It does not estimate the subscription bill. New reports
keep the price source and calculated amount. Saved reports keep their original values.

## Sources and result

The reference is
[CodexBar commit 639b155](https://github.com/steipete/CodexBar/tree/639b15522692ead0e9a26f78cebd56a0803ffc8b).
The review covered these files under `Sources/CodexBarCore/Vendored/CostUsage/`:

- `CostUsagePricing.swift`: model rates, cache rates, historical prices, and API Fast rules.
- `CostUsagePricing+Overlay.swift`: aggregate limits and custom-price precedence.
- `ModelsDevPricing.swift`: catalog lookup and context-price parsing.
- `CostUsageScanner+PricingRows.swift`: request pricing and priority fallback.
- `CostUsageScanner.swift`: usage fields and repeated totals.

CodexBar uses models.dev prices before its built-in table, except for explicit historical rules. Our
old implementation used only its built-in table. That table still matched, but it did not give the
same result as CodexBar with the current catalog.

The app now pins the resolved prices from a [catalog snapshot](cost-prices-2026-09-16.json). The
snapshot records the relevant price entries, retrieval date, and SHA256 of the downloaded
`https://models.dev/api.json` file. A null entry means the exact ID was absent from the catalog.
CodexBar's built-in price then supplies the rate for that supported ID.

The snapshot is deliberate. A price download during a run could change the comparison between two
variants. New price snapshots need review and a code change. There is no runtime network dependency
for pricing.

## Changes

- Sol uses the catalog's USD 4 input, 0.4 cached input, 5 cache write, and 20 output rates per
  million tokens. The old values were 5, 0.5, 6.25, and 30.
- Spark uses the nonzero catalog price. The old research-preview zero rate no longer applies to this
  snapshot.
- Claude fallback prices now include Sonnet 5, Opus 5, and Fable 5.1. Fable 5.1 uses its own
  cache-read rate. Sonnet 4.5 follows the catalog's standard context price.
- Pro context thresholds follow the pinned CodexBar parser. For GPT-5.4 Pro and GPT-5.5 Pro it reads
  `context_over_200k` as 200,000 tokens. It does not read the newer `tiers` field. Other models with
  a bundled 272,000-token threshold keep that threshold. This preserves CodexBar's current behavior.
- Run-total fallback includes reported cache writes. It returns unavailable when total input exceeds
  the model's context-price threshold. Such totals cannot show individual request sizes.
- When both Codex cache-read field names occur, the app takes the larger value. It does not add
  them. Final usage and request evidence must also agree on cache writes when the final count is
  present.

## Validation

A manual comparison used the upstream Swift price parser, resolver, and scalar calculations with an
explicit copy of the saved catalog. Host caches and custom overrides were disabled. The Rust results
matched all 2,040 cases within USD 0.0000000001: 1,680 Codex cases and 360 Claude cases. Cases cover
all supported table entries, both historical cutoffs, standard and priority requests, cache reads
and writes, one-hour Claude writes, and values on both sides of context thresholds. This checks the
calculation. It does not claim a live comparison with the installed CodexBar app.

The test suite keeps 152 selected upstream results in `tests/fixtures/codexbar-pricing.jsonl`. The
compiler embeds them. Tests read no files and start no processes. Separate tests check repeated
records, missing usage, fallback limits, cache aliases, saved source attribution, and comparison
differences.

To regenerate the reference values, use a clean checkout of the pinned CodexBar commit and run this
manual command from the repository root. It requires Python 3 and Swift. It writes temporary files
and invokes Swift. It is not part of automated tests.

```sh
python3 apps/agent-ui/tools/cost-reference.py /path/to/CodexBar > /tmp/codexbar-pricing.jsonl
```

Compare that output with the committed fixture before replacing it. Add `--all` to generate the full
matrix. Never generate expected prices with the Rust calculation under test.

## Limits on exact agreement

- CodexBar can use a different catalog snapshot or custom price overrides. Our app does not read
  those settings. The saved source identifies which prices a run used.
- CodexBar scans account-wide sessions and can use auxiliary logs for Fast mode and child sessions.
  Our estimate uses the saved run's matching fresh thread and its recorded service tier. Incomplete
  request evidence uses the marked fallback. Forked threads are not treated as fresh requests.
- Claude's final CLI-reported dollar cost takes priority over the message fallback. CodexBar's local
  catalog estimate can differ from that provider-reported amount.
- CodexBar can round stored request prices to billionths of a dollar. Our JSON retains the floating
  point sum. Very small numerical differences need not indicate a rate error.
- Unknown model prices stay unavailable. The supported price table covers the experiment model
  choices and retained historical models. It does not copy every provider route in models.dev.
