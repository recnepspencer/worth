"""Cargo selections for product unit tests and product-frontier courts."""
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
ROOT_TESTS = ["--workspace", "--lib", "--tests"]
COURT_OWNERS = {'worth-court-execution': ['worth-execution'], 'worth-court-foundational-certification-boundary-artifacts': ['worth-foundational'], 'worth-court-foundational-expressions': ['worth-foundational'], 'worth-court-foundational-certification-boundary-evidence': ['worth-foundational'], 'worth-court-proof': ['worth-proof'], 'worth-court-relational-certification': ['worth-relational'], 'worth-court-relational-branch-reference-compile-time': ['worth-relational'], 'worth-court-runtime-bridge': ['worth-runtime-bridge'], 'worth-court-runtime-world': ['worth-runtime-world'], 'worth-court-server-certification-suite': ['worth-server'], 'worth-court-server-product-operation-phase-eleven': ['worth-server'], 'worth-court-server-product-operation-phase-thirteen-shared-read': ['worth-server'], 'worth-court-signal-owner-services': ['worth-signal'], 'worth-court-signal-branch-retention-contract': ['worth-signal'], 'worth-court-signal-wasm': ['worth-signal-wasm']}

GROUPS = {
    "root": ROOT_TESTS,
    # Rust 1.94 repeatedly ICEs in this external schema host's incremental
    # evaluate_obligation cache. Keep the workaround in its build owner;
    # ordinary Query and Bank builds retain their incremental settings.
    "fixture": ["--manifest-path", FIXTURE, *packages(*FIXTURE_PACKAGES),
                "--lib", "--bins", "--tests", "--config", "build.incremental=false"],
    "ui": ["--manifest-path", UI, "-p", "worth-ui-query-binding", "--lib"],
}

CATEGORIES = ("process", "oracle", "cost", "determinism", "compile_fail")


def ordinary_selection(manifest):
    # Every surface has one harness; category paths select cases after discovery.
    return ["--manifest-path", manifest, "--workspace", "--tests"]


GROUPS["query"] = ordinary_selection(QUERY)
GROUPS["bank"] = ordinary_selection(BANK)


def selected(suite, binary, name):
    if "-court-" not in binary["package"]:
        return suite not in CATEGORY_SUITES
    category = next((c for c in CATEGORIES if name.startswith(c + "::")), None)
    if suite in CATEGORY_SUITES:
        return category == CATEGORY_SUITES[suite]
    return category is None


def lib_of(*names):
    return lambda binary: binary["package"] in names and binary["lib"]


def package_of(*names):
    return lambda binary: bool(set(names) & set(COURT_OWNERS.get(binary["package"], [binary["package"]])))

def everything(_binary):
    return True


def courts_only(binary):
    return "-court-" in binary["package"]


def root_rest(binary):
    return not package_of("worth-relational", "worth-signal", "worth-signal-wasm",
                          "worth-execution")(binary)


# suite -> (group, which binaries, tests per process, processes at once)
TEST_SUITES = {
    "relational": ("root", package_of("worth-relational"), 150, 4),
    "signal": ("root", package_of("worth-signal", "worth-signal-wasm"), 150, 4),
    "execution": ("root", package_of("worth-execution"), 150, 2),
    "root-rest": ("root", root_rest, 150, 4),
    "query-execution": ("query", lib_of("worth-query-execution"), 80, 4),
    "query-rest": (
        "query", lambda binary: not lib_of("worth-query-execution")(binary), 150, 1,
    ),
    # Topology cases get individual processes; other fixture owners use normal chunks.
    "fixture": ("fixture", everything, 150, 16),
    "ui": ("ui", everything, 100, 2),
    "bank": ("bank", everything, 150, 1),
}
# Category suites reuse the same build and select top-level path prefixes.
CATEGORY_SUITES = {}
for prefix in ("query", "bank", "root"):
    for label, category in (("process", "process"), ("exhaustive", "oracle"),
                            ("scale", "cost"), ("determinism", "determinism"),
                            ("compile-fail", "compile_fail")):
        suite = prefix + "-" + label
        CATEGORY_SUITES[suite] = category
        TEST_SUITES[suite] = (prefix, courts_only, 150, 1)

# Build real product executables, not only their test harnesses.
BEFORE = {name: [("bank-product-binaries", ["cargo", "build", "--manifest-path", BANK,
                                          "--workspace", "--bins"])]
          for name in TEST_SUITES if name == "bank" or name.startswith("bank-")}
