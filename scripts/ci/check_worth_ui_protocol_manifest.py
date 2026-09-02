from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "workspaces/worth-ui/contracts/milestone-3.16-protocol.json"
EXPECTED_LIVE = {
    "protocol_floor": 6, "protocol_current": 6, "mounted_frame": 5,
    "mounted_presentation": 5, "text": 3, "observation": 7,
    "measurement": 5, "solicited_effect": 1,
    "native_profile": "worth-ui-windows-dx12-v1",
}
EXPECTED_NEXT = {
    "protocol_floor": 7, "protocol_current": 7, "mounted_frame": 6,
    "mounted_presentation": 6, "text": 4, "observation": 7,
    "measurement": 5, "solicited_effect": 1,
    "native_profile": "worth-ui-windows-dx12-v2",
}


def capture(source: str, pattern: str) -> int:
    match = re.search(pattern, source)
    if match is None:
        raise ValueError(f"missing protocol source pattern: {pattern}")
    return int(match.group(1))


def validate(root: Path, manifest: Path) -> None:
    contract = json.loads(manifest.read_text(encoding="utf-8"))
    live = contract["live"]
    intended = contract["intended_next"]
    if live != EXPECTED_LIVE:
        raise ValueError(f"live manifest must remain exact: {EXPECTED_LIVE}")
    if intended != EXPECTED_NEXT:
        raise ValueError(f"intended-next manifest must remain exact: {EXPECTED_NEXT}")
    protocol = (root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_frame/protocol.rs").read_text(encoding="utf-8")
    text = (root / "workspaces/worth-ui/crates/worth-ui-host-contract/src/mounted_projection/semantic_text.rs").read_text(encoding="utf-8")
    observed = {
        "protocol_floor": capture(protocol, r"COMPATIBLE_FLOOR: u16 = (\d+)"),
        "protocol_current": capture(protocol, r"CURRENT: u16 = (\d+)"),
        "mounted_frame": capture(protocol, r"CURRENT_FRAME_SCHEMA: u16 = (\d+)"),
        "mounted_presentation": capture(protocol, r"CURRENT_PRESENTATION_SCHEMA: u16 = (\d+)"),
        "observation": capture(protocol, r"CURRENT_OBSERVATION_SCHEMA: u16 = (\d+)"),
        "measurement": capture(protocol, r"CURRENT_MEASUREMENT_SCHEMA: u16 = (\d+)"),
        "solicited_effect": capture(protocol, r"CURRENT_SOLICITED_EFFECT_SCHEMA: u16 = (\d+)"),
        "text": capture(text, r"pub const fn current\(\) -> Self \{\s*Self\((\d+)\)"),
    }
    for key, value in observed.items():
        if live[key] != value:
            raise ValueError(f"live {key} drifted: source={value}, manifest={live[key]}")
    profile = root / f"workspaces/worth-ui/crates/worth-ui-host-native/profiles/{live['native_profile']}.toml"
    if not profile.is_file():
        raise ValueError(f"missing live native profile {profile.name}")
    if intended["protocol_current"] != intended["protocol_floor"]:
        raise ValueError("intended cutover floor must equal intended current")
    if intended["protocol_current"] <= live["protocol_current"]:
        raise ValueError("intended protocol must succeed the live protocol")
    intended_profile = root / f"workspaces/worth-ui/crates/worth-ui-host-native/profiles/{intended['native_profile']}.toml"
    if not intended_profile.is_file():
        raise ValueError("intended-next native profile must be staged")
    staged_profile = tomllib.loads(intended_profile.read_text(encoding="utf-8"))
    if staged_profile.get("identity") != intended["native_profile"]:
        raise ValueError("intended-next native profile identity drifted")
    if staged_profile.get("profile_stage") != "qualification-only-non-current":
        raise ValueError("intended-next native profile must remain qualification-only")
    if staged_profile.get("live_emission") != "disabled":
        raise ValueError("intended-next native profile must keep live emission disabled")


def main() -> int:
    try:
        validate(ROOT, MANIFEST)
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"Worth UI protocol manifest gate failed: {error}", file=sys.stderr)
        return 1
    print("Worth UI protocol manifest gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
