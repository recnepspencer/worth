"""Read-only checks for the newly granted consumer paths and their root owners."""
import json
import subprocess
from pathlib import Path

root = Path.cwd()
granted = [
    "crates/worth-relational/src/branch/root_tests.rs",
    "crates/worth-relational/src/branch/root_tests/",
    "crates/worth-relational/src/branch/root_authoritative_payload_tests.rs",
]
commands = [
    ["git", "rev-parse", "HEAD"],
    ["git", "rev-parse", "codex/m3-p3-lifecycle"],
]
for path in granted:
    commands.extend([
        ["git", "diff", "--quiet", "7d0e5189bc", "codex/m3-p3-lifecycle", "--", path],
        ["git", "-C", "../m3-p3-lifecycle", "status", "--porcelain", "--", path],
    ])
commands.extend([
    ["rg", "-n", "pub.*(root|relink|publication_cost|allocation_id)|fn (readmit_for_request|readmit_unchanged_for_request|relink_canonical_envelope)",
     "crates/worth-relational/src/branch/root.rs",
     "crates/worth-relational/src/branch/root_readmission.rs",
     "crates/worth-relational/src/branch/root_readmission/request.rs",
     "crates/worth-relational/src/branch/root_schema.rs"],
    ["rg", "-n", "publication_cost|allocation_id|relink_canonical_envelope|readmit_for_request|readmit_unchanged_for_request|ptr_eq|size_of|is_complete",
     "crates/worth-relational/src/branch/root_tests.rs",
     "crates/worth-relational/src/branch/root_tests",
     "crates/worth-relational/src/branch/root_authoritative_payload_tests.rs"],
    ["rg", "-n", "readmit_for_request|readmit_unchanged_for_request|relink_canonical_envelope|RelationalBranchRootPublicationCost",
     "crates/worth-relational/src/facade"],
    ["rg", "-n", "pub.*(root|schema|visibility)|fn roots",
     "crates/worth-relational/src/branch/basis.rs",
     "crates/worth-relational/src/branch/recorded_basis.rs",
     "crates/worth-relational/src/branch/target.rs"],
    ["git", "status", "--short", "--", "crates", "tools/boundary-check/config/src-court-baseline.json"],
])
results = []
for command in commands:
    result = subprocess.run(command, capture_output=True, text=True, encoding="utf-8")
    print("$ " + subprocess.list2cmdline(command))
    print("exit:", result.returncode)
    print(result.stdout, end="")
    print(result.stderr, end="")
    results.append(dict(command=command, exit_code=result.returncode,
                        stdout=result.stdout, stderr=result.stderr))
for result in results[2:8]:
    assert result["exit_code"] == 0
    assert not result["stdout"] and not result["stderr"]
assert results[-3]["exit_code"] == 1 and not results[-3]["stdout"]
assert results[-1]["exit_code"] == 0 and not results[-1]["stdout"]
(root / "target/lane-evidence/bc-root-owner-preflight.json").write_text(
    json.dumps(results, indent=2) + "\n", encoding="utf-8")
print("No production or test source changed. Tests not run: source-boundary probe only.")
