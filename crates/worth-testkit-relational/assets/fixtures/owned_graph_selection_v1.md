# Owned-graph selection, encoding 1

`owned_graph_selection_v1.frame` is one canonical commit frame for a Native
owned-graph selection in the encoding public master writes before Tree-Based
Undo Phase 2E: full current and selected sets, no encoding tag.

Recorded from public master `f5e92d0d36` on 2026-11-26. The production library
tree was unchanged. A temporary recorder, never committed, in the
`history_ownership` module of `worth-court-relational-certification` created a
model (kind 1, with fields) owning ten members (kind 3, partition 3) through
ten owning edges, took the branch basis, edited only the model's `note` field,
selected the model's graph back to that basis through the public preparation
and publication ports, and wrote commit 23's artifact bytes unchanged. Its
source and run are in the lane's `logs/tbu-2e-a/s0a-v1-capture.log`. No bytes
were hand-edited.

`a_v1_navigation_selection_readmits_byte_exact` (worth-relational-durability,
`canonical_commit_artifact/shared_readmission_tests.rs`) readmits these bytes
through the public readmission path, which re-encodes the payload and refuses
any difference, and checks that the selection's targets are the whole roster.
