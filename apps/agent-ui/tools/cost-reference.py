"""Generate cost references with pinned upstream Swift. Run manually, not from tests."""

import argparse
import json
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("codexbar", type=Path, help="Local CodexBar checkout")
parser.add_argument("--all", action="store_true", help="Print the full comparison matrix")
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
snapshot = json.loads((root / "docs/cost-prices-2026-09-16.json").read_text())
revision = subprocess.check_output(["git", "-C", str(args.codexbar), "rev-parse", "HEAD"], text=True).strip()
if revision != snapshot["codexbar_commit"]:
    raise SystemExit("The CodexBar revision does not match the saved snapshot.")
dirty = subprocess.check_output([
    "git", "-C", str(args.codexbar), "status", "--porcelain", "--",
    "Sources/CodexBarCore/Vendored/CostUsage",
], text=True)
if dirty.strip():
    raise SystemExit("The upstream pricing source must have no local changes.")
source = args.codexbar / "Sources/CodexBarCore/Vendored/CostUsage"

def read(name):
    return (source / name).read_text()

# The explicit catalog is the only external price source in this check.
pricing = read("CostUsagePricing.swift")
start = pricing.index("    static func modelsDevCatalog(")
body = pricing.index("{", start)
end, depth = body + 1, 1
while depth:
    depth += (pricing[end] == "{") - (pricing[end] == "}")
    end += 1
pricing = pricing[:body + 1] + " return nil " + pricing[end - 1:]
parts = [
    read("ModelsDevPricing.swift").split("struct ModelsDevCacheArtifact:")[0],
    pricing,
    read("ModelsDevPricingTargetResolver.swift"),
    read("CostUsagePricing+CodexResolver.swift"),
    read("CostUsagePricing+Overlay.swift"),
]
parts.append('''
// Host caches and user overrides are disabled. Explicit catalog lookups use upstream code.
enum ModelsDevPricingPipeline {
    static func lookup(providerID: String, modelID: String, cacheRoot: URL?) -> ModelsDevPricingLookup? { nil }
}
struct CostUsageCustomPricing {
    static let empty = Self()
    static func load(fileURL: URL?) -> Self { .empty }
    func rates(providerID: String, model: String) -> Int? { nil }
    func estimatedCodexCostUSD(model: String, inputTokens: Int, cachedInputTokens: Int, outputTokens: Int, cacheWriteInputTokens: Int) -> Double? { nil }
}
let catalog = try JSONDecoder().decode(ModelsDevCatalog.self, from: Data(contentsOf: URL(fileURLWithPath: CommandLine.arguments[1])))
var rows = try JSONSerialization.jsonObject(with: Data(contentsOf: URL(fileURLWithPath: CommandLine.arguments[2]))) as! [[String: Any]]
for index in rows.indices {
    let row = rows[index]
    func n(_ key: String) -> Int { (row[key] as! NSNumber).intValue }
    let model = row["model"] as! String
    let date = Date(timeIntervalSince1970: Double(n("timestamp_ms")) / 1000)
    let usd: Double?
    if row["provider"] as! String == "codex" {
        let base = CostUsagePricing.codexCostUSD(model: model, inputTokens: n("input"), cachedInputTokens: n("cached"), outputTokens: n("output"), cacheWriteInputTokens: n("write"), pricingDate: date, modelsDevCatalog: catalog, customPricing: .empty)
        let priority = row["priority"] as! Bool ? CostUsagePricing.codexPriorityCostUSD(model: model, inputTokens: n("input"), cachedInputTokens: n("cached"), cacheWriteInputTokens: n("write"), outputTokens: n("output"), pricingDate: date, modelsDevCatalog: catalog, customPricing: .empty) : nil
        usd = priority.map { max($0, base ?? $0) } ?? base
    } else {
        usd = CostUsagePricing.claudeCostUSD(model: model, inputTokens: n("input") - n("cached") - n("write"), cacheReadInputTokens: n("cached"), cacheCreationInputTokens: n("write"), cacheCreationInputTokens1h: n("write_1h"), outputTokens: n("output"), pricingDate: date, modelsDevCatalog: catalog)
    }
    rows[index]["usd"] = usd ?? NSNull() as Any
}
print(String(data: try JSONSerialization.data(withJSONObject: rows, options: [.sortedKeys]), encoding: .utf8)!)
''')
rows = []
for provider, models in snapshot["prices"].items():
    for model in models:
        for timestamp in [1773359999999, 1773360000000, 1785369599999, 1785369600000, 1789387200000]:
            inputs = [0, 100000, 200000, 200001, 272000, 272001, 400000] if provider == "openai" else [100000, 200000, 200001, 400000]
            for total in inputs:
                for priority in ([False, True] if provider == "openai" else [False]):
                    row = dict(provider="codex" if provider == "openai" else "claude", model=model, timestamp_ms=timestamp, input=total, output=10000)
                    if provider == "openai":
                        row.update(cached=total // 3, write=total // 5, priority=priority)
                        keep = timestamp == 1789387200000 and total in (100000, 272001)
                        keep |= model in ("gpt-5.6-terra", "gpt-5.6-luna") and timestamp in (1785369599999, 1785369600000) and total == 272000
                        keep |= model in ("gpt-5.4-pro", "gpt-5.5-pro") and timestamp == 1789387200000 and total in (200000, 200001)
                    else:
                        row.update(cached=60000, write=30000, write_1h=10000)
                        keep = timestamp == 1789387200000 and total in (100000, 200001)
                        keep |= model in ("claude-sonnet-4-6", "claude-opus-4-6") and timestamp in (1773359999999, 1773360000000) and total == 200001
                    if args.all or keep:
                        rows.append(row)

catalog = {provider: {"id": provider, "models": {model: {"id": model, "cost": cost} for model, cost in models.items() if cost is not None}} for provider, models in snapshot["prices"].items()}
with tempfile.TemporaryDirectory(prefix="agent-ui-cost-reference-") as folder:
    work = Path(folder)
    (work / "main.swift").write_text("\n".join(parts))
    (work / "catalog.json").write_text(json.dumps(catalog))
    (work / "cases.json").write_text(json.dumps(rows))
    result = subprocess.check_output(["swift", str(work / "main.swift"), str(work / "catalog.json"), str(work / "cases.json")], text=True)
for row in json.loads(result):
    print(json.dumps(row, separators=(",", ":"), sort_keys=True))
