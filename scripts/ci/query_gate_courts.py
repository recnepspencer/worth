"""Cargo selections for product unit tests and boundary/class courts."""
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[2]
QUERY = "workspaces/worth-query/Cargo.toml"
FIXTURE = (
    "workspaces/worth-query/crates/worth-query-certification/fixtures/"
    "consumer_entry/Cargo.toml"
)
BANK = "workspaces/worth-query-bank-world/Cargo.toml"
UI = "workspaces/worth-ui/Cargo.toml"
BOUNDARY = "tools/boundary-check/Cargo.toml"

def packages(*names):
    return [part for name in names for part in ("-p", name)]


LOWER = ("worth-relational", "worth-execution", "worth-signal",
         "worth-runtime-bridge", "worth-runtime-world")
QUERY_PUBLIC = ("worth-query-execution", "worth-query-publication", "worth-query",
                "worth-query-host")
FIXTURE_PACKAGES = ("worth-query-topology-entry", "worth-query-parameter-entry",
                    "worth-query-consumer-root")

# One test build per group. Every suite of a group takes its binaries from
# the same cargo selection, so the first-party chain is compiled once.
ROOT_TESTS = [
    *packages("worth-relational", "worth-signal", "worth-execution",
              "worth-signal-wasm", "worth-server"),
    "--lib", "--tests",
]
GROUPS = {
    "root": ROOT_TESTS,
    # Rust 1.94 repeatedly ICEs in this external schema host's incremental
    # evaluate_obligation cache. Keep the workaround in its build owner;
    # ordinary Query and Bank builds retain their incremental settings.
    "fixture": ["--manifest-path", FIXTURE, *packages(*FIXTURE_PACKAGES),
                "--lib", "--bins", "--config", "build.incremental=false"],
    "ui": ["--manifest-path", UI, "-p", "worth-ui-query-binding", "--lib"],
}

NON_ORDINARY = ("process", "exhaustive", "scale", "compile_fail")


def court_packages(manifest, cost_class):
    crates = ROOT / Path(manifest).parent / "crates"
    found = []
    for path in crates.glob("*-court-*/Cargo.toml"):
        value = tomllib.loads(path.read_text(encoding="utf-8"))
        court = value["package"].get("metadata", {}).get("worth", {}).get("court", {})
        if court.get("class") not in ("fast", *NON_ORDINARY):
            raise ValueError(f"court must declare exactly one class: {path}")
        if court.get("class") == cost_class:
            found.append(value["package"]["name"])
    return sorted(found)


def ordinary_selection(manifest):
    excluded = [name for cost in NON_ORDINARY
                for name in court_packages(manifest, cost)]
    return ["--manifest-path", manifest, "--workspace",
            *[part for name in excluded for part in ("--exclude", name)], "--tests"]


GROUPS["query"] = ordinary_selection(QUERY)
GROUPS["bank"] = ordinary_selection(BANK)


def lib_of(*names):
    return lambda binary: binary["package"] in names and binary["lib"]


def package_of(*names):
    return lambda binary: binary["package"] in names

def everything(_binary):
    return True


# suite -> (group, which binaries, tests per process, processes at once)
TEST_SUITES = {
    "relational": ("root", package_of("worth-relational"), 150, 4),
    "signal": ("root", package_of("worth-signal", "worth-signal-wasm"), 150, 4),
    "execution": ("root", package_of("worth-execution"), 150, 2),
    "query-execution": ("query", lib_of("worth-query-execution"), 80, 4),
    "query-rest": (
        "query", lambda binary: not lib_of("worth-query-execution")(binary), 150, 1,
    ),
    # Topology cases get individual processes; other fixture owners use normal chunks.
    "fixture": ("fixture", everything, 150, 16),
    "ui": ("ui", everything, 100, 2),
    "bank": ("bank", everything, 150, 1),
}
# Process and oracle courts are separate selections; ignored scale courts are explicit only.
for prefix, manifest in (("query", QUERY), ("bank", BANK)):
    for cost in NON_ORDINARY:
        names = court_packages(manifest, cost)
        if names:
            label = prefix + "-" + cost.replace("_", "-")
            GROUPS[label] = ["--manifest-path", manifest, *packages(*names), "--tests"]
            TEST_SUITES[label] = (label, everything, 150, 1)

# Build real product executables, not only their test harnesses.
BEFORE = {name: [("bank-product-binaries", ["cargo", "build", "--manifest-path", BANK,
                                          "--workspace", "--bins"])]
          for name in TEST_SUITES if name == "bank" or name.startswith("bank-")}
