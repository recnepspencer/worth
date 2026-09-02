from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MATRIX = ROOT / "workspaces/worth-ui/contracts/milestone-3.16-native-appearance.json"
EXPECTED_MECHANICS = [
    "surface-fill", "surface-border", "corner-radii", "outline",
    "text-range-foreground", "portal-surface", "backdrop", "overlay-order",
    "pointer-affordance", "damage", "clip",
]
EXPECTED_SYMBOLS = [
    "UiMountedSurfaceAppearanceMechanic", "UiMountedPortalSurfaceAppearanceMechanic",
    "UiMountedOutlineAppearanceMechanic", "UiMountedTextForegroundAppearanceMechanic",
    "UiMountedBackdropMechanic", "UiMountedOverlayOrderMechanic",
    "UiMountedPointerAffordanceMechanic", "UiAppearanceDamageRegion", "UiAppearanceClip",
    "UiMountedSurfaceAppearanceCompletionInput", "UiMountedOutlineAppearanceCompletionInput",
    "UiMountedBackdropCompletionInput", "UiMountedTextForegroundAppearanceCompletionInput",
    "UiHostAppearanceMechanicFamily",
]
EXPECTED_OWNERS = {
    "consumed_fact_index": "workspaces/worth-ui/crates/worth-ui-runtime/src/graph/indexes/fact/index.rs",
    "mounted_preview": "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_projection/preview.rs",
    "live_static_paint": "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_projection/static_paint.rs",
}
PUBLIC_TYPE_DECLARATION = re.compile(r"\bpub\s+(?:struct|enum)\s+([A-Za-z_][A-Za-z0-9_]*)\b")
RUST_COMMENT = re.compile(r"//[^\n]*|/\*.*?\*/", re.DOTALL)
STAGED_NATIVE_RELATIVE_PATHS = (
    Path("workspaces/worth-ui/crates/worth-ui-host-native/src/native/presentation/appearance"),
    Path("workspaces/worth-ui/crates/worth-ui-host-native/src/native/event_loop/pointer_cursor.rs"),
)


def missing_declared_contract_symbols(source: str, symbols: list[str]) -> list[str]:
    declarations = set(PUBLIC_TYPE_DECLARATION.findall(RUST_COMMENT.sub("", source)))
    return [symbol for symbol in symbols if symbol not in declarations]


def leaked_contract_symbols(source: str, symbols: list[str]) -> list[str]:
    identifiers = set(re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\b", RUST_COMMENT.sub("", source)))
    return [symbol for symbol in symbols if symbol in identifiers]


def validate(root: Path, matrix_path: Path) -> None:
    matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
    mechanics = matrix["mechanics"]
    if mechanics != EXPECTED_MECHANICS:
        raise ValueError("appearance mechanic family matrix must contain the exact canonical families")
    if matrix["required_host_contract_symbols"] != EXPECTED_SYMBOLS:
        raise ValueError("host contract symbol matrix must remain exact")
    if matrix["preserved_owners"] != EXPECTED_OWNERS:
        raise ValueError("preserved appearance owners must remain exact")
    live = root / f"workspaces/worth-ui/crates/worth-ui-host-native/profiles/{matrix['live_profile']}.toml"
    intended = root / f"workspaces/worth-ui/crates/worth-ui-host-native/profiles/{matrix['intended_profile']}.toml"
    if not live.is_file() or not intended.is_file():
        raise ValueError("Gate 1 requires live v1 and staged intended v2 profiles")
    staged_profile = tomllib.loads(intended.read_text(encoding="utf-8"))
    if staged_profile.get("identity") != matrix["intended_profile"]:
        raise ValueError("staged intended profile identity drifted")
    if staged_profile.get("profile_stage") != "qualification-only-non-current":
        raise ValueError("intended profile must remain qualification-only")
    if staged_profile.get("live_emission") != "disabled":
        raise ValueError("intended profile must not enable live emission")
    host_root = root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_projection/appearance"
    source = "\n".join(path.read_text(encoding="utf-8") for path in host_root.glob("*.rs"))
    missing = missing_declared_contract_symbols(source, matrix["required_host_contract_symbols"])
    if missing:
        raise ValueError(f"host contract symbols missing for matrix: {missing}")
    for owner, relative_path in matrix["preserved_owners"].items():
        if not (root / relative_path).is_file():
            raise ValueError(f"preserved {owner} owner is missing: {relative_path}")
    command = (root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_frame/presentation_work/command_change.rs").read_text(encoding="utf-8")
    if "Appearance" in command:
        raise ValueError("Gate 0 appearance mechanics must not enter the live command family")
    live_roots = [
        root / "workspaces/worth-ui/crates/worth-ui-runtime/src/mounting/projection",
        root / "workspaces/worth-ui/crates/worth-ui-host-native/src",
        root / "workspaces/worth-ui/crates/worth-ui-host-headless/src",
        root / "workspaces/worth-ui/crates/worth-ui-native-platform/src",
    ]
    for live_root in live_roots:
        live_source = "\n".join(
            path.read_text(encoding="utf-8")
            for path in live_root.rglob("*.rs")
            if not is_staged_native_path(root, path)
        )
        leaked = leaked_contract_symbols(live_source, EXPECTED_SYMBOLS)
        if leaked:
            raise ValueError(f"Gate 0 appearance contract reached live publisher {live_root}: {leaked}")


def is_staged_native_path(root: Path, path: Path) -> bool:
    relative = path.relative_to(root)
    return any(
        relative == staged_path or staged_path in relative.parents
        for staged_path in STAGED_NATIVE_RELATIVE_PATHS
    )


def main() -> int:
    try:
        validate(ROOT, MATRIX)
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"Worth UI appearance/native matrix gate failed: {error}", file=sys.stderr)
        return 1
    print("Worth UI appearance/native matrix gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
