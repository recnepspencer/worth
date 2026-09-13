from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "workspaces/worth-ui/contracts/milestone-3.16-protocol.json"
EXPECTED_LIVE = {
    "protocol_floor": 9, "protocol_current": 9, "mounted_frame": 8,
    "mounted_presentation": 8, "text": 4, "observation": 7,
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
    if set(contract) != {"live"}:
        raise ValueError("protocol manifest must contain only the current live contract")
    live = contract["live"]
    if live != EXPECTED_LIVE:
        raise ValueError(f"live manifest must be exact: {EXPECTED_LIVE}")
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
    live_profile = tomllib.loads(profile.read_text(encoding="utf-8"))
    if live_profile.get("identity") != live["native_profile"]:
        raise ValueError("live native profile identity drifted")
    if live_profile.get("profile_stage") != "current":
        raise ValueError("live native profile must be current")
    if live_profile.get("live_emission") != "enabled":
        raise ValueError("live native profile must enable emission")


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
