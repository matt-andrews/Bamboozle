#!/usr/bin/env python3
"""
Generates the CI summary markdown comment for a PR.

Reads pre-downloaded artifacts from ./artifacts/<name>/ directories and
writes the final comment body to ./comment.md.

Expected artifact directories:
  artifacts/unit-tests/unit-test-results.json
  artifacts/playwright/playwright-results.json
  artifacts/k6/k6-output.txt
  artifacts/startup/startup-results.json
  artifacts/docker-size/docker-size.json
  artifacts/base-unit-tests/unit-test-results.json  (optional, for delta)

Environment variables:
  TRIGGERING_WORKFLOW   name of the workflow that triggered this run
  HEAD_SHA              the PR head commit SHA (shown in footer)
"""

import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

MARKER = "<!-- bamboozle-ci-report -->"

K6_METRICS = ["checks", "http_req_duration", "http_req_failed", "iteration_duration", "http_reqs"]


def load_json(path: str):
    try:
        with open(path) as f:
            return json.load(f)
    except Exception:
        return None


def status_icon(data, failed_key="failed", unexpected_key=None):
    if data is None:
        return "⏳"
    f = data.get(failed_key, 0) or data.get(unexpected_key or "", 0)
    return "❌" if f > 0 else "✅"


def format_duration(ms: float) -> str:
    s = ms / 1000
    if s < 60:
        return f"{s:.1f}s"
    m = int(s // 60)
    s = s % 60
    return f"{m}m {s:.0f}s"


def section_unit_tests() -> str:
    data = load_json("artifacts/unit-tests/unit-test-results.json")
    base = load_json("artifacts/base-unit-tests/unit-test-results.json")

    if data is None:
        return "### 🧪 Unit Tests\n⏳ _Pending_\n"

    icon = "❌" if data.get("failed", 0) > 0 else "✅"
    passed = data.get("passed", 0)
    failed = data.get("failed", 0)
    ignored = data.get("ignored", 0)

    delta = ""
    if base is not None:
        diff = (passed + failed + ignored) - (base.get("passed", 0) + base.get("failed", 0) + base.get("ignored", 0))
        if diff > 0:
            delta = f" · (+{diff} ▲)"
        elif diff < 0:
            delta = f" · ({diff} ▼)"
        else:
            delta = " · (no change)"

    lines = [
        "### 🧪 Unit Tests",
        f"{icon} **{passed} passed** · {failed} failed · {ignored} ignored{delta}",
        "",
    ]
    return "\n".join(lines)


def section_playwright() -> str:
    data = load_json("artifacts/playwright/playwright-results.json")

    if data is None:
        return "### 🎭 Playwright E2E\n⏳ _Pending_\n"

    stats = data.get("stats", {})
    passed = stats.get("expected", 0)
    failed = stats.get("unexpected", 0)
    skipped = stats.get("skipped", 0)
    flaky = stats.get("flaky", 0)
    duration_ms = stats.get("duration", 0)

    icon = "❌" if failed > 0 else "✅"
    dur_str = format_duration(duration_ms)

    flaky_str = f" · {flaky} flaky" if flaky > 0 else ""
    lines = [
        "### 🎭 Playwright E2E",
        f"{icon} **{passed} passed** · {failed} failed · {skipped} skipped{flaky_str} · {dur_str}",
        "",
    ]
    return "\n".join(lines)


def section_k6() -> str:
    txt_path = "artifacts/k6/k6-output.txt"
    if not os.path.exists(txt_path):
        return "### ⚡ K6 Load Tests\n⏳ _Pending_\n"

    selected = []
    try:
        with open(txt_path, encoding="utf-8", errors="replace") as f:
            for line in f:
                for metric in K6_METRICS:
                    if re.search(rf"\b{re.escape(metric)}\b", line):
                        # Strip ANSI colour codes that k6 emits
                        clean = re.sub(r"\x1b\[[0-9;]*m", "", line).rstrip()
                        selected.append(clean)
                        break
    except Exception as e:
        return f"### ⚡ K6 Load Tests\n❌ _Could not parse output: {e}_\n"

    icon = "✅" if selected else "⚠️"
    block = "\n".join(selected) if selected else "(no metrics found)"
    lines = [
        "### ⚡ K6 Load Tests",
        f"{icon}",
        "```",
        block,
        "```",
        "",
    ]
    return "\n".join(lines)


def section_startup() -> str:
    data = load_json("artifacts/startup/startup-results.json")

    if data is None:
        return "### 🚀 Startup Performance\n⏳ _Pending_\n"

    min_ms = data.get("min_ms", "—")
    avg_ms = data.get("avg_ms", "—")
    max_ms = data.get("max_ms", "—")
    successful = data.get("successful", "?")
    iterations = data.get("iterations", "?")

    icon = "❌" if successful != iterations else "✅"
    lines = [
        f"### 🚀 Startup Performance ({successful}/{iterations} runs)",
        f"{icon}",
        "",
        "| Min | Avg | Max |",
        "|-----|-----|-----|",
        f"| {min_ms}ms | {avg_ms}ms | {max_ms}ms |",
        "",
    ]
    return "\n".join(lines)


def section_docker_size() -> str:
    data = load_json("artifacts/docker-size/docker-size.json")

    if data is None:
        return "### 🐳 Docker Image Size\n⏳ _Pending_\n"

    human = data.get("human", "unknown")
    lines = [
        "### 🐳 Docker Image Size",
        f"`{human}` (uncompressed)",
        "",
    ]
    return "\n".join(lines)


def build_comment() -> str:
    triggering = os.environ.get("TRIGGERING_WORKFLOW", "unknown workflow")
    head_sha = os.environ.get("HEAD_SHA", "")
    sha_str = f"`{head_sha[:7]}`" if head_sha else ""
    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    meta = f"> Last updated: {timestamp} · triggered by _{triggering}_ {sha_str}"

    sections = [
        MARKER,
        "## CI Summary",
        "",
        meta,
        "",
        section_unit_tests(),
        section_playwright(),
        section_k6(),
        section_startup(),
        section_docker_size(),
    ]
    return "\n".join(sections)


if __name__ == "__main__":
    comment = build_comment()
    with open("comment.md", "w") as f:
        f.write(comment)
    print("comment.md written successfully")
    # Print a preview of detected sections
    for line in comment.splitlines():
        if line.startswith("###") or line.startswith(">"):
            print(line)
