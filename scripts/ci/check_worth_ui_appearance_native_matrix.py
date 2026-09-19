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
    "appearance_presentation_work": "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_frame/appearance_work.rs",
    "runtime_appearance_lowering": "workspaces/worth-ui/crates/worth-ui-runtime/src/mounting/projection/appearance/lowering.rs",
    "native_appearance_translation": "workspaces/worth-ui/crates/worth-ui-host-native/src/native/presentation/appearance/command.rs",
    "headless_appearance_translation": "workspaces/worth-ui/crates/worth-ui-host-headless/src/headless_translation/appearance/mod.rs",
}
EXPECTED_OWNER_DECLARATIONS = {
    "consumed_fact_index": re.compile(r"\bpub\s+struct\s+UiGraphConsumedFactIndex\b"),
    "mounted_preview": re.compile(r"\bpub\s+enum\s+UiMountedPreviewProjection\b"),
    "appearance_presentation_work": re.compile(
        r"\bpub\s+struct\s+UiMountedAppearancePresentationWork\b"
    ),
    "runtime_appearance_lowering": re.compile(r"\bpub\(super\)\s+fn\s+lower\b"),
    "native_appearance_translation": re.compile(
        r"\bpub\(crate\)\s+enum\s+UiNativeAppearanceCommand\b"
    ),
    "headless_appearance_translation": re.compile(r"\bpub\(crate\)\s+fn\s+translate\b"),
}
PUBLIC_TYPE = re.compile(r"\bpub\s+(?:struct|enum)\s+([A-Za-z_][A-Za-z0-9_]*)\b")
RUST_COMMENT = re.compile(r"//[^\n]*|/\*.*?\*/", re.DOTALL)


def validate(root: Path, matrix_path: Path) -> None:
    matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
    if set(matrix) != {"live_profile", "mechanics", "required_host_contract_symbols", "live_owners"}:
        raise ValueError("native appearance matrix must describe only the live contract")
    if matrix["live_profile"] != "worth-ui-windows-dx12-v2":
        raise ValueError("native appearance matrix must select the v2 live profile")
    if matrix["mechanics"] != EXPECTED_MECHANICS:
        raise ValueError("appearance mechanic family matrix must remain exact")
    if matrix["required_host_contract_symbols"] != EXPECTED_SYMBOLS:
        raise ValueError("host contract symbol matrix must remain exact")
    if matrix["live_owners"] != EXPECTED_OWNERS:
        raise ValueError("live appearance owner matrix must remain exact")

    profile_path = root / f"workspaces/worth-ui/crates/worth-ui-host-native/profiles/{matrix['live_profile']}.toml"
    profile = tomllib.loads(profile_path.read_text(encoding="utf-8"))
    if profile.get("identity") != matrix["live_profile"]:
        raise ValueError("live native profile identity drifted")
    if profile.get("profile_stage") != "current" or profile.get("live_emission") != "enabled":
        raise ValueError("v2 native profile must be current with live emission enabled")

    contract_root = root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_projection/appearance"
    contract_source = "\n".join(path.read_text(encoding="utf-8") for path in contract_root.glob("*.rs"))
    declarations = set(PUBLIC_TYPE.findall(RUST_COMMENT.sub("", contract_source)))
    missing = [symbol for symbol in EXPECTED_SYMBOLS if symbol not in declarations]
    if missing:
        raise ValueError(f"host contract symbols missing for matrix: {missing}")

    for owner, relative in EXPECTED_OWNERS.items():
        path = root / relative
        if not path.is_file():
            raise ValueError(f"live {owner} owner is missing: {relative}")
        source = RUST_COMMENT.sub("", path.read_text(encoding="utf-8"))
        if not EXPECTED_OWNER_DECLARATIONS[owner].search(source):
            raise ValueError(f"live {owner} owner declaration is missing: {relative}")


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
