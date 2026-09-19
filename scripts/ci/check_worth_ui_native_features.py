from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "workspaces/worth-ui/Cargo.toml"
LINUX_TARGET = "x86_64-unknown-linux-gnu"
WINDOWS_TARGET = "x86_64-pc-windows-msvc"
MODES = (
    ("linux", LINUX_TARGET, ()),
    ("linux-all-features", LINUX_TARGET, ("--all-features",)),
    ("windows", WINDOWS_TARGET, ()),
    ("windows-all-features", WINDOWS_TARGET, ("--all-features",)),
)
EXPECTED_FEATURES = {
    LINUX_TARGET: {
        ("wgpu", "29.0.4"): {"dx12", "parking_lot", "std", "wgsl"},
        ("winit", "0.30.13"): {
            "bytemuck",
            "percent-encoding",
            "rwh_06",
            "x11",
            "x11-dl",
            "x11rb",
        },
    },
    WINDOWS_TARGET: {
        ("wgpu", "29.0.4"): {"dx12", "parking_lot", "std", "wgsl"},
        ("winit", "0.30.13"): {"rwh_06"},
    },
}
EXPECTED_DEPENDENCIES = {
    ("worth-ui-host-native", "0.1.0"): {
        "pollster",
        "rustybuzz",
        "sha2",
        "swash",
        "toml",
        "wgpu",
        "winit",
        "winsafe",
        "windows",
        "worth_proof",
        "worth_signal",
        "worth_ui_host_contract",
        "worth_ui_retained_order",
    },
    ("worth-ui-native-platform", "0.1.0"): {"worth_ui_runtime"},
    ("worth-ui-host-headless", "0.1.0"): {
        "worth_ui_host_contract",
        "worth_ui_retained_order",
        "worth_ui_test_support",
    },
}


def metadata(target: str, extra: tuple[str, ...]) -> dict[str, object]:
    result = subprocess.run(
        [
            "cargo",
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--manifest-path",
            str(MANIFEST),
            *extra,
            "--filter-platform",
            target,
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr)
    return json.loads(result.stdout)


def resolved_features(target: str, extra: tuple[str, ...]) -> dict[tuple[str, str], set[str]]:
    result = subprocess.run(
        [
            "cargo",
            "tree",
            "--locked",
            "--manifest-path",
            str(MANIFEST),
            "--workspace",
            "--target",
            target,
            "--edges",
            "normal,build,dev",
            "--prefix",
            "none",
            "--format",
            "{p}|{f}",
            "--no-dedupe",
            *extra,
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr)
    return parse_feature_tree(result.stdout)


def parse_feature_tree(output: str) -> dict[tuple[str, str], set[str]]:
    observed: dict[tuple[str, str], set[str]] = {}
    qualified = {
        f"{name} v{version}": (name, version)
        for expectations in EXPECTED_FEATURES.values()
        for name, version in expectations
    }
    for line in output.splitlines():
        package, separator, features = line.partition("|")
        identity = qualified.get(package)
        if not separator or identity is None:
            continue
        observed.setdefault(identity, set()).update(filter(None, features.split(",")))
    return observed


def resolved_node(graph: dict[str, object], name: str, version: str) -> dict[str, object]:
    packages = graph.get("packages")
    resolve = graph.get("resolve")
    if not isinstance(packages, list) or not isinstance(resolve, dict):
        raise ValueError("Cargo metadata omits packages or resolution")
    identity = next(
        (
            package.get("id")
            for package in packages
            if isinstance(package, dict)
            and package.get("name") == name
            and package.get("version") == version
        ),
        None,
    )
    nodes = resolve.get("nodes")
    if identity is None or not isinstance(nodes, list):
        raise ValueError(f"missing package {name} {version}")
    node = next(
        (
            candidate
            for candidate in nodes
            if isinstance(candidate, dict) and candidate.get("id") == identity
        ),
        None,
    )
    if not isinstance(node, dict):
        raise ValueError(f"missing resolved node {name} {version}")
    return node


def expected_dependencies(target: str) -> dict[tuple[str, str], set[str]]:
    expected = {
        identity: set(dependencies)
        for identity, dependencies in EXPECTED_DEPENDENCIES.items()
    }
    if target == LINUX_TARGET:
        expected[("worth-ui-host-native", "0.1.0")].remove("winsafe")
        expected[("worth-ui-host-native", "0.1.0")].remove("windows")
    return expected


def validate_features(
    observed_features: dict[tuple[str, str], set[str]], mode: str, target: str
) -> None:
    for (name, version), expected in EXPECTED_FEATURES[target].items():
        observed = observed_features.get((name, version))
        if observed is None:
            raise ValueError(f"{mode}: missing resolved features for {name} {version}")
        if observed != expected:
            raise ValueError(
                f"{mode}: {name} features drifted: {sorted(observed)} != {sorted(expected)}"
            )


def validate_dependencies(graph: dict[str, object], mode: str, target: str) -> None:
    for (name, version), expected in expected_dependencies(target).items():
        node = resolved_node(graph, name, version)
        dependencies = node.get("deps")
        if not isinstance(dependencies, list):
            raise ValueError(f"{mode}: {name} resolved dependencies are absent")
        observed = {
            dependency.get("name")
            for dependency in dependencies
            if isinstance(dependency, dict) and isinstance(dependency.get("name"), str)
        }
        if observed != expected:
            raise ValueError(
                f"{mode}: {name} dependencies drifted: "
                f"{sorted(observed)} != {sorted(expected)}"
            )


def validate(
    graph: dict[str, object],
    observed_features: dict[tuple[str, str], set[str]],
    mode: str,
    target: str,
) -> None:
    validate_features(observed_features, mode, target)
    validate_dependencies(graph, mode, target)


def main() -> int:
    try:
        for mode, target, extra in MODES:
            validate(metadata(target, extra), resolved_features(target, extra), mode, target)
    except (json.JSONDecodeError, OSError, RuntimeError, ValueError) as error:
        print(f"native resolved-graph audit failed: {error}", file=sys.stderr)
        return 1
    print(
        "Worth UI native resolved graphs are exact for Linux and Windows "
        "in normal and all-feature modes"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
