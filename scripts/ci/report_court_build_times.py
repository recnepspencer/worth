#!/usr/bin/env python3
"""Report Cargo own-unit timings as telemetry, never as topology decisions.

Cargo 1.94 UNIT_DATA duration excludes dependency wait. Cache conditions are
reported by the caller. Deterministic source/target/graph measures are separate.
This command never builds, splits or merges crates.
"""

import argparse
from collections import Counter, defaultdict
from html.parser import HTMLParser
import json
import math
from pathlib import Path
import re
import sys
import tomllib


class ReportError(ValueError):
    """An input cannot establish an unambiguous own-unit measurement."""


class ScriptText(HTMLParser):
    def __init__(self):
        super().__init__()
        self.inside = False
        self.parts = []

    def handle_starttag(self, tag, attrs):
        if tag == "script":
            self.inside = True

    def handle_endtag(self, tag):
        if tag == "script":
            self.inside = False

    def handle_data(self, data):
        if self.inside:
            self.parts.append(data)


def parse_units(html):
    parser = ScriptText()
    parser.feed(html)
    if parser.inside:
        raise ReportError("unterminated script element")
    script = "\n".join(parser.parts)
    matches = list(re.finditer(r"\bconst\s+UNIT_DATA\s*=\s*", script))
    if len(matches) != 1:
        raise ReportError("expected exactly one script const UNIT_DATA assignment")
    payload = script[matches[0].end():]
    try:
        units, end = json.JSONDecoder().raw_decode(payload)
    except json.JSONDecodeError as error:
        raise ReportError(f"invalid UNIT_DATA JSON: {error.msg}") from error
    if not payload[end:].lstrip().startswith(";"):
        raise ReportError("UNIT_DATA assignment lacks terminating semicolon")
    if not isinstance(units, list) or not units:
        raise ReportError("UNIT_DATA must be a nonempty array (fresh build?)")
    indices = set()
    for unit in units:
        if not isinstance(unit, dict):
            raise ReportError("UNIT_DATA entries must be objects")
        index = unit.get("i")
        if type(index) is not int or index < 0 or index in indices:
            raise ReportError(f"invalid or duplicate unit index: {index!r}")
        indices.add(index)
        for field in ("name", "version", "mode", "target"):
            if not isinstance(unit.get(field), str):
                raise ReportError(f"unit {index}: missing/string field {field}")
        for field in ("duration", "start"):
            number = unit.get(field)
            if (type(number) not in (int, float) or not math.isfinite(number)
                    or number < 0):
                raise ReportError(f"unit {index}: invalid {field}")
        features = unit.get("features")
        if not isinstance(features, list) or any(not isinstance(x, str) for x in features):
            raise ReportError(f"unit {index}: invalid features")
    return units


def court_identity(name):
    if "-court-" not in name:
        return None
    workspace, suffix = name.split("-court-", 1)
    if not workspace or not suffix:
        raise ReportError(f"invalid court package name: {name}")
    return workspace, suffix


def testkit_package(name):
    return name.endswith("-testkit") or "-testkit-" in name


def read_metadata(root):
    manifests = set()
    config = root / "tools/boundary-check/config/test-workspaces.txt"
    if config.exists():
        stems = []
        for line in config.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line and not line.startswith("#"):
                workspace, separator, _ = line.partition("|")
                path = Path(workspace.strip())
                if not separator or path.is_absolute() or ".." in path.parts:
                    raise ReportError("governed workspace must be repository-relative")
                stems.append(root / path)
    elif (root / ".git").exists():
        raise ReportError("missing governed-workspace configuration")
    else:
        stems = [root]
    for stem in stems:
        for pattern in ("crates/*court*/Cargo.toml", "crates/*testkit*/Cargo.toml"):
            manifests.update(stem.glob(pattern))
    metadata = {}
    for path in sorted(manifests):
        package = tomllib.loads(path.read_text(encoding="utf-8-sig"))["package"]
        name = package["name"]
        if name in metadata:
            raise ReportError(f"duplicate manifest package name: {name}")
        identity = court_identity(name)
        if identity:
            court = package.get("metadata", {}).get("worth", {}).get("court", {})
            if court.get("surface") != identity[1]:
                raise ReportError(f"missing/mismatched court surface: {path}")
            metadata[name] = {"surface": court["surface"]}
        elif testkit_package(name):
            metadata[name] = {"surface": None}
    return metadata


def read_plan(path):
    if path is None:
        return {}, {}
    rows = json.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(rows, list):
        raise ReportError("plan must be an array of test occurrences")
    counts = Counter()
    owners = defaultdict(Counter)
    seen = set()
    for row in rows:
        if not isinstance(row, dict) or not isinstance(row.get("court"), str):
            raise ReportError("plan occurrence lacks court")
        court = row["court"]
        occurrence = row.get("court_after")
        if not isinstance(occurrence, str) or (court, occurrence) in seen:
            raise ReportError(f"missing/duplicate plan occurrence for {court}")
        seen.add((court, occurrence))
        counts[court] += 1
        owners[court][row.get("owner", "unspecified")] += 1
    return dict(counts), {name: dict(value) for name, value in owners.items()}


def summarize(units, metadata, counts, owners):
    grouped = defaultdict(list)
    for unit in units:
        name = unit["name"]
        if court_identity(name) or testkit_package(name):
            grouped[name].append(unit)
    if not grouped:
        raise ReportError("no court/testkit compiler units; input may be fully fresh")
    rows = []
    for name, own in sorted(grouped.items()):
        if name not in metadata:
            raise ReportError(f"no court/testkit manifest under --root for {name}")
        targets = set()
        versions = {unit["version"] for unit in own}
        if len(versions) != 1:
            raise ReportError(f"ambiguous versions for {name}")
        for unit in own:
            if unit["target"] in targets:
                raise ReportError(f"ambiguous duplicate target for {name}: {unit['target']!r}")
            targets.add(unit["target"])
        identity = court_identity(name)
        canonical = f' test "{name.replace("-", "_")}" (test)'
        integration = next((u["duration"] for u in own if u["target"] == canonical), None)
        # Testkits' ordinary library target is empty in Cargo's HTML schema.
        library = next((u["duration"] for u in own if u["target"] == ""), None)
        rows.append({
            "package": name, "kind": "court" if identity else "testkit",
            **metadata[name], "tests": counts.get(name),
            "planned_owners": owners.get(name, {}),
            "own_total_seconds": round(sum(u["duration"] for u in own), 6),
            "integration_seconds": integration, "library_seconds": library,
            "targets": [{"target": u["target"] or "lib", "seconds": u["duration"],
                         "unit_index": u["i"]} for u in sorted(own, key=lambda u: u["target"])],
        })
    return rows


def render_table(report):
    print("Telemetry only: own compiler work excludes dependencies; no timing gates.")
    print("Cache conditions: " + report["cache_conditions"])
    for run in report["runs"]:
        print("\n" + run["input"])
        print(f"{'Package':64} {'Own total':>10} {'Tests':>5}")
        for row in run["packages"]:
            tests = "?" if row["tests"] is None else str(row["tests"])
            print(f"{row['package']:64} {row['own_total_seconds']:9.2f}s {tests:>5}")
            for target in row["targets"]:
                print(f"  {target['target'].strip()}: {target['seconds']:.2f}s")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("html", nargs="+", type=Path, help="Cargo timing HTML, one independent run each")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--plan", type=Path, help="final court-plan.json test occurrence array")
    parser.add_argument("--cache-conditions", default="unspecified", help="describe cache state; telemetry only")
    parser.add_argument("--json", action="store_true", help="write machine JSON to stdout")
    args = parser.parse_args(argv)
    try:
        metadata = read_metadata(args.root)
        counts, owners = read_plan(args.plan)
        paths = [path.resolve() for path in args.html]
        if len(paths) != len(set(paths)):
            raise ReportError("duplicate timing input path")
        report = {"cache_conditions": args.cache_conditions, "runs": []}
        for path in paths:
            try:
                units = parse_units(path.read_text(encoding="utf-8-sig"))
                packages = summarize(units, metadata, counts, owners)
            except (OSError, ValueError) as error:
                raise ReportError(f"{path}: {error}") from error
            report["runs"].append({"input": str(path), "packages": packages})
        if args.json:
            print(json.dumps(report, indent=2, allow_nan=False))
        else:
            render_table(report)
    except (OSError, ValueError, KeyError) as error:
        print(f"court build timing error: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
