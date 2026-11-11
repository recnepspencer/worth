"""Compare actual stored discovery against the bounded-case policy correction."""
from pathlib import Path
import datetime
import importlib.util
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "target/lane-evidence"
spec = importlib.util.spec_from_file_location("permitted", EVIDENCE / "master2-tier2-permitted.py")
policy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(policy)
old = json.loads((EVIDENCE / "fable-batch1-tier2-root/plan.json").read_text())["selection"]
inventory_path = EVIDENCE / "master2-combined-tier2/store/build/inventory.json"
inventory = json.loads(inventory_path.read_text())
old_holds = {(r["package"], r["test"]) for r in old["ignored_child_parents"]}
assert policy.HELD == old_holds
assert policy.POLICY["excluded_campaign_patterns"] == old["excluded_campaign_patterns"]
expected = policy.REQUIRED - {(r["package"], r["test"]) for r in old["required_normal_cases"]}
assert len(expected) == 3
before, after = set(), set()
for suite in inventory["rust-suites"].values():
    owner = suite["package-name"]
    for name, case in suite["testcases"].items():
        key = (owner, name)
        if case["ignored"]:
            continue
        if key not in old_holds and not any(re.search(p, name) for p in old["excluded_campaign_patterns"]):
            before.add(key)
        if policy.permitted(owner, name):
            after.add(key)
assert after - before == expected
assert not before - after
assert not after & old_holds
assert all(key in after for key in policy.REQUIRED)
print(json.dumps({"inventory": str(inventory_path.relative_to(ROOT)),
    "qualification": "stored discovery policy audit only; no Rust test executed",
    "old_selected": len(before), "new_selected": len(after),
    "added_exactly": sorted(expected), "removed": [],
    "held_parent_count": len(old_holds), "ignored_bodies_selected": 0}, indent=2))
