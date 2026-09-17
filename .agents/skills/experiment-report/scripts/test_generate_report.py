"""Evidence tests use only in-memory SQLite and HTML."""

from html.parser import HTMLParser
import json
from pathlib import Path
import sqlite3
import unittest

import generate_report as report


ROOT = Path(__file__).resolve().parents[4]


class Elements(HTMLParser):
    def __init__(self, html):
        super().__init__()
        self.tags = []
        self.feed(html)

    def handle_starttag(self, tag, attrs):
        self.tags.append((tag, dict(attrs)))


class ReportTests(unittest.TestCase):
    def setUp(self):
        self.db = sqlite3.connect(":memory:")
        self.db.row_factory = sqlite3.Row
        for filename in ("ledger.sql", "ledger_activity.sql"):
            self.db.executescript((ROOT / "apps/agent-ui/src" / filename).read_text())
        self.db.execute("PRAGMA user_version=2")
        self.snapshot = {"contract": "test", "tasks": [{"id": "form--baseline", "inputs": {
            "task.md": {"bytes": list(b'Build the saved form. <script>alert(1)</script> @@TITLE@@')}
        }}]}
        self.db.execute("INSERT INTO batches VALUES (?,?,?,?,?,?,?,?,?,?,?,?)", (
            "batch", "suite", "manifest", "fingerprint", json.dumps(self.snapshot),
            json.dumps({"provider": "claude", "model": "claude-opus-4-8", "effort": "high", "timeout": 900}),
            4, 1789647937776, None, None, None, "completed",
        ))
        self.db.execute("INSERT INTO batch_tasks VALUES (?,?,?,?,?,?,?)",
                        ("batch", "form--baseline", 0, "form", "baseline", "fingerprint", "ready"))

    def tearDown(self):
        self.db.close()

    def run_row(self, run_id="run", attempt=1, **extra):
        values = dict(run_id=run_id, batch_id="batch", task_id="form--baseline", attempt_number=attempt,
                      state="ready", reserved_at_ms=10, created_at_ms=10, finished_at_ms=2010,
                      recorded_at_ms=2010, provider="claude", model_requested="claude-opus-4-8",
                      effort_requested="high", timeout_seconds=900, setup_seconds=0.25,
                      agent_seconds=1.25, verification_seconds=0.25, elapsed_seconds=2,
                      input_tokens=1000, cached_input_tokens=600, cache_write_input_tokens=300,
                      output_tokens=25, cost_usd=0.01, repair_check_count=0,
                      report_json=json.dumps({"task_config": {"repair": None}}))
        values.update(extra)
        self.db.execute(f"INSERT INTO runs ({','.join(values)}) VALUES ({','.join('?' for _ in values)})", tuple(values.values()))

    def capture(self, state="complete", events=()):
        self.db.execute("INSERT INTO run_capture(run_id,capture_version,state) VALUES ('run',1,'recording')")
        for i, (kind, tool_id, details) in enumerate(events):
            self.db.execute("INSERT INTO run_events(run_id,sequence,observed_at_ms,elapsed_ms,kind,phase,tool_call_id,message_id,details_json) VALUES (?,?,?,?,?,?,?,?,?)",
                            ("run", i, 10, 10, kind, "agent", tool_id, "message" if kind == "message.usage" else None, json.dumps(details)))
        self.db.execute("UPDATE run_capture SET state=? WHERE run_id='run'", (state,))

    def load(self, **kwargs):
        return report.load_report(self.db, "batch", **kwargs)

    def test_cache_and_phase_remainders_do_not_double_count(self):
        self.run_row()
        result = self.load()["runs"][0]
        self.assertEqual(result["uncached_input_tokens"], 100)
        self.assertEqual(result["other_seconds"], 0.25)
        self.assertEqual(result["input_tokens"], 1000)

    def test_missing_capture_is_not_zero_and_known_zero_is_visible(self):
        self.run_row(output_tokens=0, input_tokens=None)
        result = self.load()["runs"][0]
        self.assertIsNone(result["tool_calls"])
        self.assertIsNone(result["uncached_input_tokens"])
        self.assertEqual(report.fmt(result["output_tokens"]), "0")
        self.assertEqual(report.fmt(result["input_tokens"]), "Unavailable")

    def test_duplicate_tool_and_usage_events_are_counted_once(self):
        self.run_row()
        self.capture(events=[
            ("tool.requested", "tool1", {"name": "Read"}),
            ("tool.requested", "tool1", {"name": "Read"}),
            ("tool.result", "tool1", {"is_error": True}),
            ("tool.result", "tool1", {"is_error": True}),
            ("message.usage", None, {"usage": {"output_tokens": 5}}),
            ("message.usage", None, {"usage": {"output_tokens": 10}}),
            ("provider.result", None, {"permission_denials": [], "modelUsage": {
                "claude-opus-4-8": {"thinkingTokens": 12}, "claude-haiku": {"thinkingTokens": 99}}}),
        ])
        row = self.load()["runs"][0]
        self.assertEqual((row["tool_calls"], row["tool_errors"], row["requests"]), (1, 1, 1))
        self.assertEqual(row["tools"], {"Read": 1})
        self.assertEqual(row["thinking"], 12)
        self.assertEqual(row["denials"], 0)
        self.assertEqual(row["output_tokens"], 25)

    def test_failed_attempt_is_kept_and_selection_is_explicit(self):
        self.run_row(state="failed")
        self.run_row("retry", 2)
        self.assertEqual([r["state"] for r in self.load()["runs"]], ["failed", "ready"])
        latest = self.load(attempt="latest")
        self.assertEqual(latest["runs"][0]["run_id"], "retry")
        self.assertEqual(latest["excluded"], 1)
        self.assertEqual(self.load(attempt="first")["runs"][0]["state"], "failed")

    def test_pending_task_remains_visible(self):
        result = self.load()
        self.assertIsNone(result["runs"][0]["run_id"])
        self.assertTrue(result["warnings"])

    def test_partial_capture_keeps_observed_counts(self):
        self.run_row(state="failed")
        self.capture("partial", [("tool.requested", "tool1", {"name": "Bash"})])
        result = self.load()
        self.assertEqual(result["runs"][0]["tool_calls"], 1)
        self.assertTrue(any("partial" in warning for warning in result["warnings"]))
        self.assertIsNone(result["runs"][0]["denials"])

    def test_prompt_is_saved_escaped_and_not_a_template_instruction(self):
        self.run_row()
        model = self.load()
        html = report.render_report(model, (report.ASSETS / "report.html").read_text())
        self.assertIn("Build the saved form.", html)
        self.assertIn("&lt;script&gt;alert(1)&lt;/script&gt; @@TITLE@@", html)
        self.assertNotIn("<script>alert(1)</script>", html)
        parsed = Elements(html)
        self.assertFalse(any(a.get("src") or a.get("href") for tag, a in parsed.tags if tag in ("script", "link", "img")))
        self.assertEqual(len([t for t, a in parsed.tags if t == "details"]), 2)
        self.assertFalse(any("open" in a for t, a in parsed.tags if t == "details"))

    def test_invalid_remainders_are_unavailable(self):
        self.run_row(input_tokens=10, elapsed_seconds=0)
        result = self.load()
        self.assertIsNone(result["runs"][0]["uncached_input_tokens"])
        self.assertIsNone(result["runs"][0]["other_seconds"])
        self.assertEqual(len(result["warnings"]), 3)

    def test_ambiguous_scenario_and_unknown_batch_fail(self):
        self.db.execute("INSERT INTO batch_tasks VALUES (?,?,?,?,?,?,?)",
                        ("batch", "other--context", 1, "other", "context", "fingerprint", "pending"))
        with self.assertRaisesRegex(ValueError, "Select one scenario"):
            self.load()
        with self.assertRaisesRegex(ValueError, "Batch not found"):
            report.load_report(self.db, "missing")
        self.assertEqual(self.load(scenario="form")["scenario"], "form")


if __name__ == "__main__":
    unittest.main()
