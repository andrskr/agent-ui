#!/usr/bin/env python3
"""Build a frozen report from one Agent UI ledger batch. No provider is started."""

import argparse
from collections import Counter
from datetime import datetime, timezone
from html import escape
import json
import math
from pathlib import Path
import sqlite3
import sys
import xml.etree.ElementTree as ET


ASSETS = Path(__file__).resolve().parents[1] / "assets"
UNAVAILABLE = "Unavailable"
VARIANTS = {"baseline": "Baseline", "context": "Context"}


def number(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value) and value >= 0


def fmt(value, unit=""):
    if not number(value):
        return UNAVAILABLE
    if unit == "s":
        return f"{value:,.1f} s"
    if unit == "$":
        return f"${value:,.3f}"
    return f"{value:,}"


def decode(value, fallback=None):
    return json.loads(value) if value else fallback


def open_ledger(path):
    db = sqlite3.connect(Path(path).expanduser().resolve().as_uri() + "?mode=ro", uri=True)
    db.row_factory = sqlite3.Row
    db.execute("PRAGMA query_only=ON")
    return db


def check_schema(db):
    if db.execute("PRAGMA user_version").fetchone()[0] != 3:
        raise ValueError("This report generator requires Agent UI ledger schema 3.")


def batches(db):
    check_schema(db)
    return [dict(row) for row in db.execute(
        "SELECT batch_id,suite,state,created_at_ms,settings_json FROM batches ORDER BY created_at_ms DESC"
    )]


def activity(db, run):
    """Count observed activity once per tool or message. Keep absent capture unknown."""
    capture = db.execute("SELECT * FROM run_capture WHERE run_id=?", (run["run_id"],)).fetchone()
    run.update(capture_state=capture["state"] if capture else "not_recorded", tools=None,
               tool_calls=None, tool_errors=None, denials=None, requests=None, events=None,
               log_bytes=capture["log_bytes"] if capture else None, thinking=run.get("reasoning_output_tokens"))
    if not capture or capture["state"] == "not_started":
        return
    events = list(db.execute(
        "SELECT sequence,kind,tool_call_id,details_json FROM run_events WHERE run_id=? ORDER BY sequence",
        (run["run_id"],),
    ))
    run["events"] = len(events)
    if run["provider"] != "claude":
        return
    requested, errors, result = {}, set(), None
    for event in events:
        details = decode(event["details_json"], {})
        key = event["tool_call_id"] or str(event["sequence"])
        if event["kind"] == "tool.requested":
            requested[key] = details.get("name", "Unknown tool")
        elif event["kind"] == "tool.result" and details.get("is_error") is True:
            errors.add(key)
        elif event["kind"] == "provider.result":
            result = details
    # A partial capture gives observed counts, not evidence that no calls happened.
    observed = bool(events) or capture["state"] == "complete"
    if observed:
        run.update(tools=dict(Counter(requested.values())), tool_calls=len(requested), tool_errors=len(errors))
        run["requests"] = db.execute(
            "SELECT count(*) FROM run_request_usage WHERE run_id=?", (run["run_id"],)
        ).fetchone()[0]
    if result:
        denials = result.get("permission_denials")
        run["denials"] = len(denials) if isinstance(denials, list) else None
        if run["thinking"] is None:
            usage = result.get("modelUsage", {})
            matches = [v for k, v in usage.items() if
                       k == run["model_requested"] or v.get("canonicalModel") == run["model_requested"]]
            if len(matches) == 1 and number(matches[0].get("thinkingTokens")):
                run["thinking"] = matches[0]["thinkingTokens"]


def load_report(db, batch_id, scenario=None, attempt="all"):
    check_schema(db)
    if attempt not in ("all", "first", "latest"):
        raise ValueError("Attempt selection must be all, first, or latest.")
    db.row_factory = sqlite3.Row
    batch = db.execute("SELECT * FROM batches WHERE batch_id=?", (batch_id,)).fetchone()
    if not batch:
        raise ValueError(f"Batch not found: {batch_id}")
    batch = dict(batch)
    tasks = [dict(t) for t in db.execute(
        "SELECT * FROM batch_tasks WHERE batch_id=? ORDER BY task_order", (batch_id,)
    )]
    scenarios = sorted({t["scenario"] for t in tasks})
    if scenario is None:
        if len(scenarios) != 1:
            raise ValueError("Select one scenario with --scenario: " + ", ".join(scenarios))
        scenario = scenarios[0]
    if scenario not in scenarios:
        raise ValueError(f"Scenario {scenario} is not in this batch.")
    snapshot = decode(batch.pop("snapshot_json"), {})
    settings = decode(batch.pop("settings_json"), {})
    warnings, runs, excluded = [], [], 0
    for task in tasks:
        if task["scenario"] != scenario:
            continue
        attempts = [dict(row) for row in db.execute(
            "SELECT * FROM runs WHERE batch_id=? AND task_id=? ORDER BY attempt_number",
            (batch_id, task["task_id"]),
        )]
        chosen = attempts if attempt == "all" else attempts[:1] if attempt == "first" else attempts[-1:]
        excluded += len(attempts) - len(chosen)
        if not chosen:
            chosen = [{"task_id": task["task_id"], "state": task["state"], "run_id": None}]
        for run in chosen:
            run["variant"] = task["variant"]
            run["label"] = VARIANTS.get(task["variant"], task["variant"])
            if len(attempts) > 1:
                run["label"] += f" · Attempt {run['attempt_number']}"
            if run["run_id"]:
                activity(db, run)
                report = decode(run.get("report_json"), {})
                run["cost_note"] = report.get("cost_note")
            else:
                run["capture_state"] = "not_started"
            for key in ("setup_seconds", "agent_seconds", "verification_seconds", "elapsed_seconds",
                        "input_tokens", "cached_input_tokens", "cache_write_input_tokens", "output_tokens", "cost_usd"):
                if run.get(key) is not None and not number(run[key]):
                    warnings.append(f"{run['label']}: invalid {key}; shown as unavailable.")
                    run[key] = None
            phases = [run.get(k) for k in ("setup_seconds", "agent_seconds", "verification_seconds")]
            run["other_seconds"] = None
            if number(run.get("elapsed_seconds")) and all(number(v) for v in phases):
                remainder = run["elapsed_seconds"] - sum(phases)
                if remainder >= -0.001:
                    run["other_seconds"] = max(0, remainder)
                else:
                    warnings.append(f"{run['label']}: phase durations exceed elapsed time.")
            run["uncached_input_tokens"] = None
            parts = [run.get(k) for k in ("input_tokens", "cached_input_tokens", "cache_write_input_tokens")]
            if run.get("provider") == "claude" and all(number(v) for v in parts):
                remainder = parts[0] - parts[1] - parts[2]
                if remainder >= 0:
                    run["uncached_input_tokens"] = remainder
                else:
                    warnings.append(f"{run['label']}: cache counts exceed total input.")
            if run.get("recorded_at_ms") is None:
                warnings.append(f"{run['label']}: no final recorded result; values can be incomplete.")
            if run.get("capture_state") != "complete":
                warnings.append(f"{run['label']}: activity capture is {run.get('capture_state')}.")
            runs.append(run)
    example = next((t for t in tasks if t["scenario"] == scenario and t["variant"] == "baseline"),
                   next(t for t in tasks if t["scenario"] == scenario))
    saved = next((t for t in snapshot.get("tasks", []) if t["id"] == example["task_id"]), {})
    content = saved.get("inputs", {}).get("task.md", {}).get("bytes")
    prompt = bytes(content).decode("utf-8") if content is not None else UNAVAILABLE
    if prompt == UNAVAILABLE:
        warnings.append("The saved task prompt is unavailable.")
    return {"batch": batch, "scenario": scenario, "settings": settings, "runs": runs,
            "prompt": prompt, "prompt_task": example["task_id"], "contract": snapshot.get("contract"),
            "selection": attempt, "excluded": excluded, "warnings": warnings}


def node(tag, text=None, **attrs):
    element = ET.Element(tag, {k.rstrip("_").replace("_", "-"): str(v) for k, v in attrs.items()})
    element.text = text
    return element


def pretty_model(value):
    if not value:
        return UNAVAILABLE
    parts = value.removeprefix("claude-").split("-")
    if parts[0] in ("opus", "sonnet", "haiku") and all(x.isdigit() for x in parts[1:]):
        return parts[0].capitalize() + " " + ".".join(parts[1:])
    return value


def common(runs, key, fallback=None):
    values = {str(r[key]) for r in runs if r.get(key) is not None}
    if not values:
        return str(fallback) if fallback is not None else UNAVAILABLE
    return next(iter(values)) if len(values) == 1 else "Mixed: " + "; ".join(sorted(values))


def header(model):
    batch, settings, runs = model["batch"], model["settings"], model["runs"]
    date = datetime.fromtimestamp(batch["created_at_ms"] / 1000, timezone.utc).strftime("%d %b %Y")
    h = node("header", class_="ir-header")
    h.append(node("div", "SCENARIO REPORT", class_="text-small text-muted"))
    h.append(node("h1", model["scenario"].replace("-", " ").capitalize()))
    details = node("details", id="ir-prompt")
    details.append(node("summary", "Prompt example", class_="ir-disclosure"))
    details.append(node("div", model["prompt_task"] + " · Saved with this batch", class_="text-small text-muted"))
    details.append(node("pre", model["prompt"], class_="ir-prompt-text"))
    h.append(details)
    details = node("details", id="ir-configuration")
    summary = node("summary", class_="ir-disclosure")
    summary.append(node("span", f"{pretty_model(settings.get('model'))} · {date} UTC · {settings.get('effort', UNAVAILABLE)} effort "))
    summary.append(node("span", "Full configuration", class_="text-small text-muted ir-config-hint"))
    details.append(summary)
    dl = node("dl", class_="ir-config-grid")
    rows = [("Provider", settings.get("provider")), ("Model", settings.get("model")),
            ("Effort (requested)", settings.get("effort")), ("Date", date + " UTC"), ("Suite", batch["suite"]),
            ("Timeout", fmt(settings.get("timeout"), "s") + " per task"),
            ("Concurrency", f"{batch['concurrency']} maximum"), ("Agent CLI", common(runs, "provider_version")),
            ("Runner", common(runs, "runner_version")), ("Runner build", common(runs, "runner_build")),
            ("Node.js", common(runs, "node_version")), ("Provider binary", settings.get("binary") or "Default"),
            ("Batch state", batch["state"]), ("Capture", "Metrics and activity"), ("Contract", model["contract"])]
    for key, value in rows:
        dl.append(node("dt", key))
        dl.append(node("dd", str(value) if value is not None else UNAVAILABLE))
    details.append(dl)
    h.append(details)
    passed = sum(r["state"] == "ready" for r in runs)
    h.append(node("div", f"{len(runs)} task attempts · {passed}/{len(runs)} passed automated checks", class_="text-small text-muted ir-status"))
    return h


def chart(runs, title, key, ident, unit="", parts=(), note=""):
    section = node("section", aria_labelledby=ident)
    section.append(node("h3", title, id=ident))
    available = [r[key] for r in runs if number(r.get(key))]
    maximum = max(available, default=0)
    scale = f"Scale: 0–{fmt(maximum, unit)}" + (" tokens" if not unit else "")
    section.append(node("div", scale if available else "No measured values", class_="text-small text-muted"))
    for run in runs:
        lane = node("div", class_="ir-lane")
        labels = node("div", class_="ir-label")
        labels.append(node("span", run["label"]))
        labels.append(node("span", fmt(run.get(key), unit), class_="ir-value"))
        lane.append(labels)
        value = run.get(key)
        exact = f"${value:,.6f}" if unit == "$" and number(value) else fmt(value, unit)
        track = node("div", class_="ir-track", data_report_tip=f"{run['label']} · {title}: {exact} · State: {run['state']}")
        can_stack = parts and all(number(run.get(k)) for _, k, _ in parts)
        if number(value) and (not parts or can_stack):
            segments = parts or [(title, key, 1)]
            for label, field, color in segments:
                amount = run[field]
                width = 100 * amount / maximum if maximum else 0
                percentage = f" · {100 * amount / value:.3f}% of total" if value else ""
                bar = node("span", class_="ir-bar", style=f"width:{width:.8f}%;background:var(--viz-series-{color})")
                if parts:
                    precise = f"{amount:,.3f} s" if unit == "s" else fmt(amount) + " tokens"
                    bar.set("data-report-tip", f"{run['label']} · {label}: {precise}{percentage}")
                track.append(bar)
        elif number(value):
            # A total can be known while its breakdown is unavailable.
            width = 100 * value / maximum if maximum else 0
            track.append(node("span", class_="ir-bar", style=f"width:{width:.8f}%;background:var(--muted-foreground)"))
        lane.append(track)
        if parts:
            breakdown = node("div", class_="ir-phase-values text-small")
            for label, field, color in parts:
                item = node("span", class_="ir-phase")
                item.append(node("span", class_="ir-dot", aria_hidden="true", style=f"background:var(--viz-series-{color})"))
                item.append(node("span", f"{label} {fmt(run.get(field), unit)}"))
                breakdown.append(item)
            lane.append(breakdown)
        section.append(lane)
    if note:
        section.append(node("div", note, class_="text-small text-muted"))
    return section


def table(runs, title, rows, note=None):
    section = node("section")
    section.append(node("h3", title))
    wrap = node("div", class_="table-responsive ir-table-wrap")
    table_node = node("table", class_="table table-sm")
    head, body, row = node("thead"), node("tbody"), node("tr")
    row.append(node("th", "Metric", scope="col"))
    for run in runs:
        row.append(node("th", run["label"], scope="col", class_="text-end"))
    head.append(row)
    for label, values in rows:
        row = node("tr")
        row.append(node("th", label, scope="row"))
        for value in values:
            row.append(node("td", str(value), class_="text-end"))
        body.append(row)
    table_node.extend([head, body])
    wrap.append(table_node)
    section.append(wrap)
    if note:
        section.append(node("div", note, class_="text-small text-muted"))
    return section


def render_report(model, template):
    runs = model["runs"]
    article = node("article", id="invite-report")
    article.append(header(model))
    article.append(chart(runs, "Time by phase", "elapsed_seconds", "ir-time-title", "s", (
        ("Setup", "setup_seconds", 2), ("Agent", "agent_seconds", 1),
        ("Verification", "verification_seconds", 3), ("Other", "other_seconds", 4))))
    article.append(chart(runs, "Estimated API cost", "cost_usd", "ir-cost-title", "$",
                         note="USD · Saved API price estimates · Includes reported auxiliary model use"))
    panels = node("div", class_="ir-panels ir-token-panels")
    panels.append(chart(runs, "Total input tokens", "input_tokens", "ir-input-title", parts=(
        ("Cache reads", "cached_input_tokens", 1), ("Cache writes", "cache_write_input_tokens", 2),
        ("Uncached", "uncached_input_tokens", 3)), note="Normalized input · Cache breakdown is available for Claude"))
    panels.append(chart(runs, "Output tokens", "output_tokens", "ir-output-title", note="Output tokens · Separate scale from input"))
    article.append(panels)
    values = lambda key: [fmt(r.get(key)) for r in runs]
    known_tools = ["Bash", "Read", "Grep", "Edit", "Glob", "Write"]
    observed_tools = {tool for r in runs for tool in (r.get("tools") or {})}
    tools = [tool for tool in known_tools if tool in observed_tools] + sorted(observed_tools - set(known_tools))
    rows = [(tool, [fmt(r["tools"].get(tool, 0)) if r.get("tools") is not None else UNAVAILABLE for r in runs]) for tool in tools]
    rows.append(("Total tool calls", values("tool_calls")))
    article.append(table(runs, "Recorded tool activity", rows,
                         "Calls by tool · Zero means no recorded calls · Partial captures show observed counts only"))
    rows = [("Run state", [r["state"] for r in runs]), ("Observed model requests", values("requests")),
            ("Tool results with errors", values("tool_errors")), ("Permission denials", values("denials")),
            ("Saved activity events", values("events")), ("Raw log size (bytes)", values("log_bytes")),
            ("Activity capture", [r.get("capture_state", "not_recorded") for r in runs]),
            ("Invalid event lines", values("invalid_event_lines")),
            ("Automated verification", ["Passed" if r["state"] == "ready" else
                                        "Failed" if r.get("verification_exit_code") not in (None, 0) else "Not confirmed" for r in runs]),
            ("Changed source files", values("changed_file_count"))]
    article.append(table(runs, "Activity and checks", rows,
                         "Permission denials can also appear as tool errors. Do not add these counts."))
    article.append(table(runs, "Token use", [(label, values(key)) for label, key in (
        ("Total input tokens", "input_tokens"), ("Input: cache reads", "cached_input_tokens"),
        ("Input: cache writes", "cache_write_input_tokens"), ("Input: uncached", "uncached_input_tokens"),
        ("Output tokens", "output_tokens"), ("Reported thinking tokens", "thinking"))],
        "Input totals include cache reads and writes for Claude. Thinking tokens use normalized usage or the saved provider result; thinking time was not measured."))
    section, notes = node("section"), node("ul")
    section.append(node("h3", "Measurement notes"))
    texts = [f"Attempt selection: {model['selection']}. {model['excluded']} other attempts are excluded from this report.",
             "These are individual observations. Concurrent tasks can affect setup and verification time.",
             "Agent time includes tool work.",
             "Verification time covers command execution. Other time is the remaining elapsed time.",
             "Automated checks do not measure visual quality. No visual quality score was recorded.",
             "Cost is a saved API price estimate, not a subscription charge.",
             "Unavailable is not zero. Partial activity counts cover only the observed events."]
    for key, label in (("cost_models_json", "Cost models"), ("cost_note", "Cost note")):
        groups = {}
        for r in runs:
            value = ", ".join(decode(r.get(key), [])) if key == "cost_models_json" else r.get(key)
            if value:
                groups.setdefault(value, []).append(r["label"])
        for value, labels in groups.items():
            scope = "" if len(labels) == len(runs) else ", ".join(labels) + " · "
            texts.append(f"{scope}{label}: {value}")
    for r in runs:
        if r.get("error"):
            texts.append(f"{r['label']} run error: {r['error']}")
    for text in texts + model["warnings"]:
        notes.append(node("li", text))
    section.append(notes)
    article.append(section)
    footer = node("footer", class_="text-small text-muted")
    footer.append(node("div", f"Source: Agent UI SQLite ledger · Batch {model['batch']['batch_id']}"))
    for run in runs:
        footer.append(node("div", f"{run['label']}: {run['run_id'] or 'No run recorded'}"))
    article.append(footer)
    if template.count("@@CONTENT@@") != 1 or template.count("@@TITLE@@") != 1:
        raise ValueError("Invalid report template placeholders.")
    # Replace the data last. User text must never become another template instruction.
    return template.replace("@@TITLE@@", escape(model["scenario"].replace("-", " ").capitalize() + " report")).replace(
        "@@CONTENT@@", ET.tostring(article, encoding="unicode", method="html"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", required=True, type=Path)
    parser.add_argument("--list", action="store_true", help="List recorded batches without generating a report")
    parser.add_argument("--batch")
    parser.add_argument("--scenario")
    parser.add_argument("--attempt", choices=("all", "first", "latest"), default="all")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.list and any((args.batch, args.scenario, args.output)):
        parser.error("Use --list without batch, scenario, or output.")
    if not args.list and (not args.batch or not args.output):
        parser.error("Report generation requires --batch and --output.")
    try:
        with open_ledger(args.db) as db:
            db.execute("BEGIN")
            if args.list:
                print(json.dumps(batches(db), indent=2))
                return
            model = load_report(db, args.batch, args.scenario, args.attempt)
        html = render_report(model, (ASSETS / "report.html").read_text())
        output = args.output.expanduser().absolute()
        if output.suffix.lower() != ".html":
            raise ValueError("Output must be an .html file.")
        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open("x", encoding="utf-8") as file:
            file.write(html)
        print(json.dumps({"output": str(output), "batch": args.batch, "scenario": model["scenario"],
                          "attempt_selection": args.attempt, "attempts": len(model["runs"]),
                          "excluded_attempts": model["excluded"], "warnings": model["warnings"]}, indent=2))
    except (ValueError, OSError, sqlite3.Error, KeyError, TypeError) as error:
        parser.exit(1, f"Report failed: {error}\n")


if __name__ == "__main__":
    main()
