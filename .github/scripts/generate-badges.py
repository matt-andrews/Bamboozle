#!/usr/bin/env python3
"""
Generates shields.io endpoint badge JSON files from CI artifacts.

Writes badge files to BADGE_OUTPUT_DIR (default: current directory):
  badge-startup.json      — average startup time

Environment variables:
  BADGE_OUTPUT_DIR      where to write output files (default: .)
  MAX_STARTUP_AVG_MS    (optional) threshold; values above this are red
"""

import json
import os
from pathlib import Path


def find_file(artifact_dir: str, filename: str) -> Path | None:
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


def startup_color(avg_ms: int, threshold_ms: int | None) -> str:
    if threshold_ms is not None:
        if avg_ms > threshold_ms:
            return "red"
        if avg_ms > threshold_ms * 0.8:
            return "yellow"
        return "brightgreen"
    # Sensible defaults when no threshold is configured
    if avg_ms > 500:
        return "red"
    if avg_ms > 200:
        return "yellow"
    return "brightgreen"


def write_badge(path: Path, label: str, message: str, color: str) -> None:
    badge = {"schemaVersion": 1, "label": label, "message": message, "color": color}
    path.write_text(json.dumps(badge))
    print(f"  wrote {path.name}: {label} | {message} | {color}")


output_dir = Path(os.environ.get("BADGE_OUTPUT_DIR", "."))
output_dir.mkdir(parents=True, exist_ok=True)

threshold_startup = _env_int("MAX_STARTUP_AVG_MS")

# --- Startup badge ---
startup_data = load_json("artifacts/startup", "startup-results.json")
if startup_data and startup_data.get("avg_ms") is not None:
    avg_ms = startup_data["avg_ms"]
    write_badge(
        output_dir / "badge-startup.json",
        label="startup",
        message=f"{avg_ms}ms avg",
        color=startup_color(avg_ms, threshold_startup),
    )
else:
    write_badge(output_dir / "badge-startup.json", "startup", "unknown", "lightgrey")
