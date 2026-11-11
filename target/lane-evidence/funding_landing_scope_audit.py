from pathlib import Path
import subprocess, re

def run(args):
    observed = subprocess.run(args, text=True, capture_output=True)
    print(subprocess.list2cmdline(args))
    print(observed.stdout, end="")
    print(observed.stderr, end="")
    assert observed.returncode == 0, observed.returncode

# Compare Rust tokens with the exact granted patch result; rustfmt may fold
# `self` + `.composition` onto one line without changing the expression.
patch = Path("target/lane-evidence/proposed-outside-area-clippy.patch").read_text().splitlines()
def tokens(text):
    return re.findall(r'"(?:\\.|[^"\\])*"|[A-Za-z_][A-Za-z_0-9]*|[0-9]+|\S', text)
i = 0
checked = 0
while i < len(patch):
    assert patch[i].startswith("--- a/")
    file = patch[i][6:]
    assert patch[i + 1] == "+++ b/" + file
    expected = subprocess.check_output(["git", "show", "HEAD:" + file]).decode().splitlines()
    i += 2
    delta = 0
    while i < len(patch) and patch[i].startswith("@@"):
        start = int(re.match(r"@@ -(\d+)", patch[i]).group(1)) - 1 + delta
        i += 1
        old, new = [], []
        while i < len(patch) and not patch[i].startswith(("@@", "--- a/")):
            line = patch[i]
            if line.startswith((" ", "-")): old.append(line[1:])
            if line.startswith((" ", "+")): new.append(line[1:])
            i += 1
        assert expected[start:start + len(old)] == old, file
        expected[start:start + len(old)] = new
        delta += len(new) - len(old)
    assert tokens("\n".join(expected)) == tokens(Path(file).read_text()), file
    print("Verified exact granted Rust expressions (formatting ignored):", file)
    checked += 1
assert checked == 6
# Exact prefix and all three caller rows are compared below; the old two-row
# EOF patch cannot reverse-apply after the newly granted third append.
excluded = [
"durability/checkpoints/region_directory.rs", "durability/checkpoints/symbol_pages.rs",
"durability/data/checkpoint_directory.rs", "durability/data/checkpoint_directory/admission.rs",
"durability/data/checkpoint_directory/retirement.rs", "durability/data/native_checkpoint_capture_denial.rs",
"durability/data/recovery_stop.rs", "storage/residency/cell.rs",
"storage/residency/cell/selected_readmission.rs", "storage/residency/kind_authority.rs"]
run(["git", "diff", "--quiet", "7d0e5189bc", "--", *["crates/worth-relational/src/" + p for p in excluded]])
config = "tools/boundary-check/config/certification-construction-callers.txt"
prior = subprocess.check_output(["git", "show", "HEAD:" + config]).decode().splitlines()
current = Path(config).read_text().splitlines()
assert current[:len(prior)] == prior
leaf = "crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/contract_work.rs"
assert current[len(prior):] == ["native-fresh-initialization " + leaf, "native-durable-initial-installation " + leaf, "native-home-original-source " + leaf.replace("contract_work.rs", "cold_materialization/selected_image.rs")]
run(["git", "apply", "--reverse", "--check", "target/lane-evidence/proposed-inline-bytes-clone-quote.patch"])
run(["git", "diff", "--quiet", "HEAD", "--", "crates/worth-relational/src/authority/mutation/intents/record_aspect_patch/controlled.rs"])
print("Verified exact granted expressions, append-only callers, ten exclusions, exact Foundation function patch and untouched Native guard.")

import json, hashlib
prior_source = json.loads(Path("target/lane-evidence/funding-final-source-state.json").read_text())["files"]
for path, expected in prior_source.items():
    if path == config: continue
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == expected, path
print("All 59 prior non-config source hashes remain identical to qualified source.")
for row in json.loads(Path("target/lane-evidence/funding-landing-clippy-diagnostics.json").read_text()):
    path = row["path"]
    assert row["master_diff_exit"] == 1, row
    run(["git", "diff", "--quiet", "HEAD", "--", path])
print("Every measured Root lint is in an untouched master-changed file (standing rule 3).")
source = "crates/worth-court-relational-branch-reference-compile-time/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/change_source_pass/selected_commit_mints_receipt.rs"
for ref in ("codex/m3-p3-lifecycle", "origin/master"):
    run(["git", "diff", "--quiet", "7d0e5189bc", ref, "--", source])
old = subprocess.check_output(["git", "show", "HEAD:" + source]).decode()
start = old.index("    Some(\n")
end = old.index("\n}", start)
old = old[:start] + "    runtime.mint_change_receipt(execution, selected, None).ok()" + old[end:]
assert tokens(old) == tokens(Path(source).read_text()), source
print("Verified sole new Rust correction is the eligible, expression-equivalent Option return.")
