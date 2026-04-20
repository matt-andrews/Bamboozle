#!/usr/bin/env python3
"""
Generates the CI summary markdown comment for a PR.

Reads pre-downloaded artifacts from ./artifacts/<name>/ directories and
writes the final comment body to ./comment.md and overall status to
./status.json.

Each artifact directory is searched recursively for the target filename so
that `actions/upload-artifact`'s directory-structure preservation doesn't
matter (e.g. artifacts/k6/scripts/perf-test/k6/k6-output.txt is found the
same as artifacts/k6/k6-output.txt).

Expected artifact directories (with their target files):
  artifacts/unit-tests/   → unit-test-results.json
  artifacts/playwright/   → playwright-results.json
  artifacts/k6/           → k6-output.txt
  artifacts/startup/      → startup-results.json
  artifacts/docker-size/  → docker-size.json
  artifacts/base-unit-tests/ → unit-test-results.json  (optional, for delta)

Environment variables:
  TRIGGERING_WORKFLOW   name of the workflow that triggered this run
  HEAD_SHA              the PR head commit SHA (shown in footer)
  MAX_STARTUP_AVG_MS    (optional) fail if startup avg exceeds this value
  MAX_DOCKER_SIZE_MB    (optional) fail if image size exceeds this value (MiB)
"""

import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path

MARKER = "<!-- bamboozle-ci-report -->"

K6_METRICS = ["checks", "http_req_duration", "http_req_failed", "iteration_duration", "http_reqs"]

# Each section appends one of: "pending" | "passing" | "failing"
_statuses: list[str] = []


def _record(status: str) -> None:
    _statuses.append(status)


def find_file(artifact_dir: str, filename: str) -> Path | None:
    """Return the first match for *filename* anywhere under *artifact_dir*."""
    matches = list(Path(artifact_dir).glob(f"**/{filename}"))
    return matches[0] if matches else None


def load_json(artifact_dir: str, filename: str):
    path = find_file(artifact_dir, filename)
    if path is None:
        return None
    try:
        with open(path) as f:
            return json.load(f)
    except Exception:
        return None


def _env_int(name: str) -> int | None:
    v = os.environ.get(name, "").strip()
    try:
        return int(v) if v else None
    except ValueError:
        return None


def _env_float(name: str) -> float | None:
    v = os.environ.get(name, "").strip()
    try:
        return float(v) if v else None
    except ValueError:
        return None


def format_duration(ms: float) -> str:
    s = ms / 1000
    if s < 60:
        return f"{s:.1f}s"
    m = int(s // 60)
    return f"{m}m {s % 60:.0f}s"


def section_unit_tests() -> str:
    data = load_json("artifacts/unit-tests", "unit-test-results.json")
    base = load_json("artifacts/base-unit-tests", "unit-test-results.json")

    if data is None:
        _record("pending")
        return "### 🧪 Unit Tests\n⏳ _Pending_\n"

    passed = data.get("passed", 0)
    failed = data.get("failed", 0)
    ignored = data.get("ignored", 0)

    _record("failing" if failed > 0 else "passing")
    icon = "❌" if failed > 0 else "✅"

    delta = ""
    if base is not None:
        diff = (passed + failed + ignored) - (base.get("passed", 0) + base.get("failed", 0) + base.get("ignored", 0))
        if diff > 0:
            delta = f" · (+{diff} ▲)"
        elif diff < 0:
            delta = f" · ({diff} ▼)"
        else:
            delta = " · (no change)"

    return "\n".join([
        "### 🧪 Unit Tests",
        f"{icon} **{passed} passed** · {failed} failed · {ignored} ignored{delta}",
        "",
    ])


def section_playwright() -> str:
    data = load_json("artifacts/playwright", "playwright-results.json")

    if data is None:
        _record("pending")
        return "### 🎭 Playwright E2E\n⏳ _Pending_\n"

    stats = data.get("stats", {})
    passed = stats.get("expected", 0)
    failed = stats.get("unexpected", 0)
    skipped = stats.get("skipped", 0)
    flaky = stats.get("flaky", 0)
    duration_ms = stats.get("duration", 0)

    _record("failing" if failed > 0 else "passing")
    icon = "❌" if failed > 0 else "✅"
    flaky_str = f" · {flaky} flaky" if flaky > 0 else ""

    return "\n".join([
        "### 🎭 Playwright E2E",
        f"{icon} **{passed} passed** · {failed} failed · {skipped} skipped{flaky_str} · {format_duration(duration_ms)}",
        "",
    ])


def section_k6() -> str:
    txt_path = find_file("artifacts/k6", "k6-output.txt")
    if txt_path is None:
        _record("pending")
        return "### ⚡ K6 Load Tests\n⏳ _Pending_\n"

    try:
        content = txt_path.read_text(encoding="utf-8", errors="replace")
    except Exception as e:
        _record("failing")
        return f"### ⚡ K6 Load Tests\n❌ _Could not parse output: {e}_\n"

    # k6 prints "FAILED" on its summary line when any threshold is exceeded
    k6_failed = bool(re.search(r"\bFAILED\b", content))
    _record("failing" if k6_failed else "passing")
    icon = "❌" if k6_failed else "✅"

    selected = []
    for line in content.splitlines():
        for metric in K6_METRICS:
            if re.search(rf"\b{re.escape(metric)}\b", line):
                clean = re.sub(r"\x1b\[[0-9;]*m", "", line).rstrip()
                selected.append(clean)
                break

    block = "\n".join(selected) if selected else "(no metrics found)"
    return "\n".join([
        "### ⚡ K6 Load Tests",
        f"{icon}",
        "```",
        block,
        "```",
        "",
    ])


def section_startup() -> str:
    data = load_json("artifacts/startup", "startup-results.json")

    if data is None:
        _record("pending")
        return "### 🚀 Startup Performance\n⏳ _Pending_\n"

    avg_ms_val: int | None = data.get("avg_ms")
    min_ms_val: int | None = data.get("min_ms")
    max_ms_val: int | None = data.get("max_ms")
    successful = data.get("successful", 0)
    iterations = data.get("iterations", "?")

    avg_display = f"{avg_ms_val}ms" if avg_ms_val is not None else "—"
    min_display = f"{min_ms_val}ms" if min_ms_val is not None else "—"
    max_display = f"{max_ms_val}ms" if max_ms_val is not None else "—"

    threshold = _env_int("MAX_STARTUP_AVG_MS")
    threshold_exceeded = threshold is not None and avg_ms_val is not None and avg_ms_val > threshold

    run_failed = successful != iterations
    is_failing = run_failed or threshold_exceeded
    _record("failing" if is_failing else "passing")
    icon = "❌" if is_failing else "✅"

    notes = []
    if threshold_exceeded:
        notes.append(f"avg {avg_display} exceeds threshold ({threshold}ms)")
    note_line = f"\n> ⚠️ {'; '.join(notes)}" if notes else ""

    return "\n".join([
        f"### 🚀 Startup Performance ({successful}/{iterations} runs)",
        f"{icon}{note_line}",
        "",
        "| Min | Avg | Max |",
        "|-----|-----|-----|",
        f"| {min_display} | {avg_display} | {max_display} |",
        "",
    ])


def section_docker_size() -> str:
    data = load_json("artifacts/docker-size", "docker-size.json")

    if data is None:
        _record("pending")
        return "### 🐳 Docker Image Size\n⏳ _Pending_\n"

    human = data.get("human", "unknown")
    bytes_val = data.get("bytes", 0)

    threshold_mb = _env_float("MAX_DOCKER_SIZE_MB")
    threshold_bytes = threshold_mb * 1024 * 1024 if threshold_mb is not None else None
    threshold_exceeded = threshold_bytes is not None and bytes_val > threshold_bytes

    _record("failing" if threshold_exceeded else "passing")
    icon = "❌" if threshold_exceeded else "✅"

    notes = []
    if threshold_exceeded:
        notes.append(f"`{human}` exceeds threshold ({threshold_mb:.0f} MiB)")
    note_line = f"\n> ⚠️ {'; '.join(notes)}" if notes else ""

    return "\n".join([
        "### 🐳 Docker Image Size",
        f"{icon} `{human}` (uncompressed){note_line}",
        "",
    ])


def build_comment() -> str:
    triggering = os.environ.get("TRIGGERING_WORKFLOW", "unknown workflow")
    head_sha = os.environ.get("HEAD_SHA", "")
    sha_str = f"`{head_sha[:7]}`" if head_sha else ""
    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    meta = f"> Last updated: {timestamp} · triggered by _{triggering}_ {sha_str}"

    sections = "\n".join([
        section_unit_tests(),
        section_playwright(),
        section_k6(),
        section_startup(),
        section_docker_size(),
    ])

    # Derive overall status from recorded section statuses
    pending_count = _statuses.count("pending")
    failing_count = _statuses.count("failing")
    total = len(_statuses)

    if failing_count > 0:
        overall_state = "failure"
        state_desc = f"{failing_count} check(s) failed"
    elif pending_count > 0:
        overall_state = "pending"
        state_desc = f"{total - pending_count}/{total} checks complete"
    else:
        overall_state = "success"
        state_desc = "All checks passed"

    with open("status.json", "w") as f:
        json.dump({"state": overall_state, "description": state_desc}, f)

    return "\n".join([
        MARKER,
        "## CI Summary",
        "",
        meta,
        "",
        sections,
    ])


if __name__ == "__main__":
    comment = build_comment()
    with open("comment.md", "w") as f:
        f.write(comment)
    print("comment.md and status.json written")
    for line in comment.splitlines():
        if line.startswith("###") or line.startswith(">"):
            print(line)
