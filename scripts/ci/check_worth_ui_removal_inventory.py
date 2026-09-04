from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "workspaces/worth-ui/contracts/milestone-3.16-removal-inventory.json"
CURRENT_STAGE = "gate_4"
INVENTORY_KEYS = {"current_stage", "cutover_target", "entries"}
REQUIRED_FAMILIES = {
    "static-paint authority": (
        "ComponentStaticPaintContract", 51, 50,
        "Gate 0 preserves the live static-paint prerequisite until the Gate 5 cutover.",
        50,
        "Gate 4 retains the live static-paint authority prerequisite until the Gate 5 cutover.",
    ),
    "bootstrap component token dependencies": (
        "with_theme_token_dependency", 10, 10,
        "Gate 0 preserves bootstrap dependency declarations until the Gate 5 cutover.",
        10,
        "Gate 4 retains bootstrap dependency declarations required by the preserved static-paint path until the Gate 5 cutover.",
    ),
    "string-backed ThemeColorValue": (
        "ThemeColorValue", 96, 99,
        "Two Gate 0 explicit-attachment fixtures and the real native pointer-observation fixture use the live ThemeTokenDescriptor input required by preserved static paint; migrate them with the Gate 5 cutover.",
        110,
        "Gate 4 retains string color values in the pre-cutover token definitions and the native pointer-observation and appearance/theme fixtures that exercise preserved static-paint prerequisites; migrate them with the Gate 5 cutover.",
    ),
    "Pulse Unicode icon text": (
        "portal_icon_text", 6, 6,
        "Gate 0 inventories Pulse icon text for its later typed-icon migration.",
        6,
        "Gate 4 retains Pulse's Unicode icon-text substitute and pixel control-point fields for the pre-cutover portal evidence; migrate them with the Gate 5 cutover.",
    ),
    "direct token publication": (
        "UiNativeThemeTokenValueChange", 20, 20,
        "Gate 0 preserves the live publication path until the Gate 5 cutover.",
        33,
        "Gate 4 retains the native token-publication boundary for staged appearance updates, including its runtime signatures and test/support callers; migrate it with the Gate 5 cutover.",
    ),
    "legacy changed-node selection": (
        "changed_graph_nodes", 46, 46,
        "Gate 0 preserves legacy selection until its planned query-owned replacement.",
        25,
        "Gate 4 retains changed-node plumbing in allocation and mounted-delta paths after the G3 legacy theme-selector retirement; remove the remaining migration residue with the Gate 5 cutover.",
    ),
}
RUST_GLOB = "workspaces/worth-ui/**/*.rs"
ENTRY_KEYS = {
    "family", "glob", "literal", "original_baseline", "gate_zero_remaining",
    "gate_zero_retention", "current_remaining", "current_retention",
}


def rust_source_paths(root: Path) -> list[Path]:
    result = subprocess.run(
        [
            "git", "ls-files", "--cached", "--others", "--exclude-standard", "-z",
            "--", "workspaces/worth-ui",
        ],
        cwd=root,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise ValueError(result.stderr.decode("utf-8", errors="replace").strip())
    return [
        root / relative.decode("utf-8")
        for relative in result.stdout.split(b"\0")
        if relative
        and relative.endswith(b".rs")
        and (root / relative.decode("utf-8")).is_file()
    ]


def validate_inventory_shape(inventory: object) -> list[object]:
    if not isinstance(inventory, dict):
        raise ValueError("removal inventory must be a JSON object")
    if set(inventory) != INVENTORY_KEYS:
        raise ValueError("removal inventory keys must remain exact")
    if inventory["current_stage"] != CURRENT_STAGE:
        raise ValueError("removal inventory current stage must remain gate_4")
    if type(inventory["cutover_target"]) is not int or inventory["cutover_target"] != 0:
        raise ValueError("cutover target must be exactly zero")
    if not isinstance(inventory["entries"], list):
        raise ValueError("removal inventory entries must be a JSON array")
    return inventory["entries"]


def validate_entry_values(entry: dict[str, object], family: str) -> None:
    for key in ("glob", "literal", "gate_zero_retention", "current_retention"):
        if not isinstance(entry[key], str) or not entry[key].strip():
            raise ValueError(f"{family}: {key} must be a non-empty string")
    for key in ("original_baseline", "gate_zero_remaining", "current_remaining"):
        if type(entry[key]) is not int or entry[key] < 0:
            raise ValueError(f"{family}: {key} must be a non-negative integer")


def validate(root: Path, manifest: Path) -> None:
    inventory = json.loads(manifest.read_text(encoding="utf-8"))
    entries = validate_inventory_shape(inventory)
    families: set[str] = set()
    files = rust_source_paths(root)
    for entry in entries:
        if not isinstance(entry, dict):
            raise ValueError("removal inventory entries must be JSON objects")
        if set(entry) != ENTRY_KEYS:
            raise ValueError("removal inventory entry keys must remain exact")
        family = entry["family"]
        if not isinstance(family, str) or not family.strip():
            raise ValueError("removal inventory family must be a non-empty string")
        if family in families:
            raise ValueError(f"duplicate inventory family: {family}")
        families.add(family)
        expected = REQUIRED_FAMILIES.get(family)
        if expected is None:
            raise ValueError(f"unexpected removal inventory contract for {family}")
        validate_entry_values(entry, family)
        (
            literal,
            original_baseline,
            gate_zero_remaining,
            gate_zero_retention,
            current_remaining,
            current_retention,
        ) = expected
        if (
            entry["glob"] != RUST_GLOB
            or entry["literal"] != literal
            or entry["original_baseline"] != original_baseline
            or entry["gate_zero_remaining"] != gate_zero_remaining
            or entry["gate_zero_retention"] != gate_zero_retention
            or entry["current_remaining"] != current_remaining
            or entry["current_retention"] != current_retention
        ):
            raise ValueError(f"unexpected removal inventory contract for {family}")
        observed = sum(path.read_text(encoding="utf-8").count(entry["literal"]) for path in files)
        if observed != entry["current_remaining"]:
            raise ValueError(
                f"{family}: observed {observed}, expected exact Gate 4 current "
                f"remaining {entry['current_remaining']}"
            )
    if families != set(REQUIRED_FAMILIES):
        raise ValueError("removal inventory must contain every required legacy family exactly once")


def main() -> int:
    try:
        validate(ROOT, MANIFEST)
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"Worth UI removal inventory gate failed: {error}", file=sys.stderr)
        return 1
    print("Worth UI removal inventory gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
