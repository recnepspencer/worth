"""One focused owner build/run selecting prior failures and changed test bodies.

Fable takeover variant of master2_focused_runtime.py: the changed-source set and the
introduced test names come from the live uncommitted batch (git diff HEAD plus untracked
Rust files) instead of the frozen 41-file source map. Selection logic is unchanged.
"""
from collections import defaultdict
from pathlib import Path
import argparse
import importlib.util
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "target/lane-evidence"
sys.path[:0] = [str(ROOT / "scripts"), str(ROOT / "scripts/test_cost"), str(ROOT / "scripts/ci")]
from application_build import application_build_environment
from executed_coverage import observations
from query_gate_cost import exact_binary
from run_lane import execute, child_environment, LaneExecutionFailure

RENAMED = {
    "chk20_store_written_before_historical_selection_still_opens": [
        "retired_checkpoint_refuses_before_reading_legacy_publication_meaning",
        "current_checkpoint_readmits_original_owned_publication_and_live_members"],
    "custody_checkpoint_revision_reads_legacy_and_refuses_unknown_versions": [
        "current_checkpoint_preserves_meaning_and_retired_or_unknown_versions_refuse"],
    "prepared_cell_second_reservation_refuses_with_original_cause_and_refund": [
        "prepared_cell_admission_refuses_with_original_cause_and_refund"],
    "semantic_result_preserves_custom_enforcement_version_distinct_from_aggregate_request": [
        "semantic_result_preserves_independent_original_execution_and_enforcement_versions"],
}


SCOPES = {"root": lambda p: not p.startswith("workspaces/"),
          "bank": lambda p: p.startswith("workspaces/worth-query-bank-world/"),
          "store": lambda p: p.startswith("workspaces/worth-store/")}


def git(*arguments):
    return subprocess.check_output(["git", *arguments], cwd=ROOT, text=True, encoding="utf-8")


def changed_sources(workspace):
    """Working-tree text of every Rust file the uncommitted repair batch changed or added."""
    names = git("diff", "--name-only", "--diff-filter=AM", "HEAD", "--", "*.rs").splitlines()
    names += git("ls-files", "--others", "--exclude-standard", "--", "*.rs").splitlines()
    texts = {}
    for name in sorted(set(names)):
        if SCOPES[workspace](name):
            texts[name] = Path("\\\\?\\" + str((ROOT / name).resolve())).read_text(encoding="utf-8")
    # Bank has no changed source of its own: its prior failures are reselected by name.
    return texts


def introduced_functions(changed):
    """Function names the batch adds: every fn in an untracked file, plus added fn lines."""
    untracked = set(git("ls-files", "--others", "--exclude-standard", "--", "*.rs").splitlines())
    names = set(re.findall(r"^\+\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+([A-Za-z0-9_]+)",
                           git("diff", "HEAD", "--", "*.rs"), re.M))
    for path, text in changed.items():
        if path in untracked:
            names.update(re.findall(r"\bfn\s+([A-Za-z0-9_]+)", text))
    return names

def policy_module():
    spec = importlib.util.spec_from_file_location("permitted", EVIDENCE / "master2-tier2-permitted.py")
    policy = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(policy)
    return policy


def selection_plan(workspace):
    previous = EVIDENCE / "master2-combined-tier2" / workspace
    inventory = json.loads((previous / "build/inventory.json").read_text())["rust-suites"]
    failures = defaultdict(set)
    logs = sorted(previous.glob("run-*.log"))
    if workspace == "store":
        supplement = EVIDENCE / "master2-store-bounded-supplement/run-three-bounded.log"
        if supplement.is_file():
            logs.append(supplement)
    events, _ = observations(logs, {"rust-suites": inventory})
    for event in events:
        if event["event"] == "failed":
            failures[event["binary"]].add(event["test"])
    assert failures, "do not rerun an unchanged green workspace"
    changed = changed_sources(workspace)
    # Select actual named test bodies in changed files, never production functions.
    functions = {name for path, text in changed.items() if path.endswith(".rs")
                 for name in re.findall(r"#\[test\]\s*(?:#\[[^\n]*\]\s*)*"
                                        r"(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?"
                                        r"fn\s+([A-Za-z0-9_]+)", text)}
    introduced = functions & introduced_functions(changed)
    wanted = defaultdict(set)
    migrations = []
    for binary, names in failures.items():
        assert binary in inventory
        for name in names:
            suffix = name.rsplit("::", 1)[-1]
            replacements = RENAMED.get(suffix)
            if replacements:
                prefix = name.rsplit("::", 1)[0]
                replacements = [prefix + "::" + new for new in replacements]
                wanted[binary].update(replacements)
                migrations.append({"binary": binary, "retired_claim": name,
                                   "required_current_and_refusal_tests": replacements})
            else:
                wanted[binary].add(name)
    for binary, suite in inventory.items():
        for name in suite["testcases"]:
            if name.rsplit("::", 1)[-1] in functions:
                wanted[binary].add(name)
    if workspace == "store":
        owner = "worth-store-physical-backend"
        for name in (
            "filesystem_media::tests::storage_profile::tests::only_uninjected_pinned_registry_profile_is_supported",
            "filesystem_media::artifact_tree::path_storage::tests::qualified_component_open_bounds_cover_unicode_growth_and_missing_file",
        ):
            matches = [binary for binary, suite in inventory.items()
                       if suite["package-name"] == owner and name in suite["testcases"]]
            assert len(matches) == 1, ("actual profile boundary court missing", name)
            wanted[matches[0]].add(name)
    # New modules/tests appear only in the new build; admit them by exact fn name.
    return inventory, wanted, functions, introduced, migrations


def main():
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("workspace", choices=("root", "bank", "store"))
    args.add_argument("--out", type=Path, required=True)
    options = args.parse_args()
    assert not options.out.exists(), "reuse evidence rather than overwrite/rerun it"
    policy = policy_module()
    policy.disk_guard()
    previous, wanted, functions, introduced, migrations = selection_plan(options.workspace)
    packages = {previous[binary]["package-name"] for binary in wanted}
    # New source test modules live in these existing, actual owner packages.
    if options.workspace == "root":
        packages.update(("worth-foundational", "worth-relational"))

    class FocusedQueries:
        workspace_inventory = False
        build_selection = {"packages": sorted(packages), "targets": {}}

        def __call__(self, discovered):
            jobs, selected, found = [], [], set()
            for binary, suite in discovered["rust-suites"].items():
                if not suite.get("harness", True):
                    if "<main>" in wanted.get(binary, set()):
                        case = suite["testcases"]["<main>"]
                        assert not case["ignored"], (binary, "<main>", "ignored body")
                        assert policy.permitted(suite["package-name"], "<main>"), (binary, "excluded main")
                        jobs.append((binary, None, None, "ci", [binary]))
                        found.add((binary, "<main>"))
                        selected.append({"binary": binary, "test": "<main>",
                                         "package": suite["package-name"], "execution": "standalone"})
                    continue
                names = {name for name in suite["testcases"]
                         if name in wanted.get(binary, set()) or name.rsplit("::", 1)[-1] in functions}
                for name in names:
                    case = suite["testcases"][name]
                    assert not case["ignored"], (binary, name, "ignored body")
                    assert policy.permitted(suite["package-name"], name), (binary, name, "excluded body")
                    found.add((binary, name))
                    selected.append({"binary": binary, "test": name, "package": suite["package-name"]})
                if names:
                    prefix = exact_binary(binary) + " & ("
                    terms, chunks = [], []
                    for name in sorted(names):
                        term = "test(=" + name + ")"
                        candidate = prefix + " + ".join([*terms, term]) + ")"
                        if terms and len(subprocess.list2cmdline(["-E", candidate])) > 18000:
                            chunks.append(prefix + " + ".join(terms) + ")")
                            terms = []
                        terms.append(term)
                    if terms:
                        chunks.append(prefix + " + ".join(terms) + ")")
                    for number, query in enumerate(chunks):
                        assert len(subprocess.list2cmdline(["-E", query])) <= 18000
                        jobs.append((binary + ":focused-" + str(number), query, None, "ci"))
            missing = {(binary, name) for binary, names in wanted.items() for name in names} - found
            assert not missing, ("a prior failure/replacement vanished", sorted(missing))
            if options.workspace == "root":
                missing_new = introduced - {name.rsplit("::", 1)[-1] for _, name in found}
                assert not missing_new, ("an introduced regression was not selected", sorted(missing_new))
            (options.out / "focused-selection.json").write_text(json.dumps({
                "selected": selected, "migrations": migrations,
                "required_introduced_tests": sorted(introduced) if options.workspace == "root" else [],
                "native_query_chunks": [{"name": job[0], "expression_chars": len(job[1])}
                                        for job in jobs if job[1] is not None],
                "scope": "prior failed names plus named test bodies in reviewed changed sources"}, indent=2) + "\n")
            return jobs

    environment = application_build_environment(child_environment())
    runner = policy.runner_type()(ROOT, environment)
    try:
        result = execute(ROOT, options.workspace, [], options.out,
                         profile="ci-test" if options.workspace == "store" else "dev",
                         cargo_runner=runner, queries=FocusedQueries(), include_docs=False)
    except LaneExecutionFailure as failure:
        print(str(failure), flush=True)
        return 1
    return int(bool(result.get("failures")))


if __name__ == "__main__":
    raise SystemExit(main())
