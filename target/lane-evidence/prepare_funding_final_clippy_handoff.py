"""Prepare exact remaining Root lint corrections without touching their owners."""
from pathlib import Path
import difflib, json
court = "crates/worth-court-relational-certification/tests/worth_court_relational_certification/"
native = "crates/worth-relational/src/"
changes = {}
def update(path, old, new, count=1):
    source = changes.get(path, Path(path).read_text())
    assert source.count(old) == count, (path, source.count(old), old)
    changes[path] = source.replace(old, new)
for path in (court + "process/serial/relational/managed_source_work_account/public_only.rs", court + "process/serial/relational/managed_source_work_account.rs"):
    update(path, "stopped.work <= EXTRA - 1", "stopped.work < EXTRA")
update(court + "process/serial/relational/managed_source_work_account.rs", "scenario.rule_limit.then(|| std::num::NonZeroU64::MIN)", "scenario.rule_limit.then_some(std::num::NonZeroU64::MIN)")
for path, count in ((court + "relational/durable_publication/index_catalog.rs", 2), (court + "relational/durable_publication/index_catalog/recovery.rs", 2), (native + "tests/durability/contracts/caller_association.rs", 1), (native + "tests/durability/medium/cold_envelope_matrix.rs", 1)):
    update(path, ".canonical_commit_artifacts(&execution)", ".canonical_commit_artifacts(execution)", count)
update(court + "relational/managed_source/nested.rs", 'stopped.err().expect("original nested request is cancelled")', 'stopped.expect_err("original nested request is cancelled")')
update(court + "relational/managed_source/row_read.rs", '.err()\n                    .expect("original ticket cannot outlive cancellation")', '.expect_err("original ticket cannot outlive cancellation")')
path = court + "relational/managed_source/owner.rs"
source = Path(path).read_text()
a = source.index("        } else if previous\n")
b = source.index("        Ok(CustomInvariantVerdict::Pass)", a)
old = source[a:b]
new = old.replace("        {\n            if let Some(old) = previous.as_ref() {", "            && let Some(old) = previous.as_ref()\n        {")
new = new.replace("                ", "            ")
# Only the nested body loses four spaces, not continuation indentation outside it.
lines = old.splitlines()
start = lines.index("            if let Some(old) = previous.as_ref() {")
new = "\n".join(lines[:start-1] + ["            && let Some(old) = previous.as_ref()", "        {"] + [line[4:] for line in lines[start+1:-2]] + ["        }"]) + "\n"
update(path, old, new)
path = "crates/worth-court-relational-branch-reference-compile-time/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/change_source_pass/selected_commit_mints_receipt.rs"
update(path, "    Some(\n        runtime\n            .mint_change_receipt(execution, selected, None)\n            .ok()?,\n    )", "    runtime\n        .mint_change_receipt(execution, selected, None)\n        .ok()")
path = native + "derived_relation/evaluation/memo/tests.rs"
source = Path(path).read_text()
a = source.index("            .filter_map(|(ordinal, (kind, a, b))| {")
b = source.index("            .collect();", a)
new = """            .filter(|(_, (kind, a, b))| {
                probe.relation == KindId(*kind)
                    && probe.anchor == if probe.outgoing { id(*a) } else { id(*b) }
            })
            .map(|(ordinal, (kind, a, b))| StructuralRelationRecord {
                relation_id: RelationId::new(PartitionId(0), ordinal as u64, 1),
                kind_id: KindId(kind),
                source: id(a),
                target: id(b),
            })
"""
update(path, source[a:b], new)
for name in ("created_entity_bindings.rs", "created_relation_bindings.rs"):
    path = native + "transactions/data/outcomes/" + name
    source = Path(path).read_text()
    a = source.index("#[cfg(test)]\nmod tests {")
    b = source.index("\nfn resolve_client_key", a)
    tests = source[a:b].rstrip()
    changes[path] = source[:a] + source[b:].lstrip().rstrip() + "\n\n" + tests + "\n"
patch = ""
for path, changed in changes.items():
    source = Path(path).read_bytes().decode()
    if "\r\n" in source:
        assert source.count("\n") == source.count("\r\n"), path
        changed = changed.replace("\n", "\r\n")
    patch += "".join(difflib.unified_diff(source.splitlines(True), changed.splitlines(True), fromfile="a/" + path, tofile="b/" + path))
Path("target/lane-evidence/proposed-funding-final-outside-clippy.patch").write_bytes(patch.encode())
Path("target/lane-evidence/funding-final-outside-clippy-files.json").write_text(json.dumps({p: len(Path(p).read_text().splitlines()) for p in changes}, indent=2)+"\n")
print("Prepared", len(changes), "owner files /16 lint fixes, without writing source.")
