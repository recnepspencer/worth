use super::source_corpus::{
    BRANCHES_SOURCE, MERGE_RUNTIME_EXECUTION_FINALIZATION_SOURCE, RUNTIME_SNAPSHOTTING_SOURCE,
    RUNTIME_STATE_SOURCE,
};

fn contains_direct_branch_state_store(source: &str, argument_prefix: &str) -> bool {
    source.contains(&format!(".store_branch_state({argument_prefix}"))
}

#[test]
fn branch_state_store_guard_distinguishes_method_calls_from_lookalike_helpers() {
    assert!(contains_direct_branch_state_store(
        "self.branches.store_branch_state(snapshot.meta.branch_id, state);",
        "snapshot.meta.branch_id,"
    ));
    assert!(!contains_direct_branch_state_store(
        "self.snapshot_restore_branch_state(snapshot.meta.branch_id, snapshot.meta.snapshot_id);",
        "snapshot.meta.branch_id,"
    ));
}

#[test]
fn branch_snapshot_restore_packets_are_mediated_through_transition_helpers() {
    assert!(
        BRANCHES_SOURCE.contains("SnapshotBranchState")
            && BRANCHES_SOURCE.contains("into_branch_state("),
        "snapshot branch state should expose an explicit rehydration transition"
    );
    assert!(
        RUNTIME_SNAPSHOTTING_SOURCE.contains("snapshot_state.into_branch_state("),
        "branch snapshot restore should rebuild stored branch state through the snapshot transition helper"
    );
    assert!(
        BRANCHES_SOURCE.contains("prepare_owner_cell_restore")
            && BRANCHES_SOURCE.contains("snapshot_state.into_branch_state("),
        "sealed owner cells should rebuild branch state through the same snapshot transition"
    );
    assert!(
        !RUNTIME_SNAPSHOTTING_SOURCE.contains("let mut state = BranchState {")
            && !RUNTIME_SNAPSHOTTING_SOURCE.contains("let state = BranchState {"),
        "branch snapshot restore should not hand-assemble branch state by struct literal"
    );
    assert!(
        !RUNTIME_SNAPSHOTTING_SOURCE.contains("(snapshot, branch_catalog, state.clone())")
            && !contains_direct_branch_state_store(
                RUNTIME_SNAPSHOTTING_SOURCE,
                "branch.id, branch_state",
            )
            && !contains_direct_branch_state_store(
                RUNTIME_SNAPSHOTTING_SOURCE,
                "snapshot.meta.branch_id,",
            )
            && !RUNTIME_SNAPSHOTTING_SOURCE.contains("insert_snapshot(snapshot.meta.snapshot_id,")
            && !MERGE_RUNTIME_EXECUTION_FINALIZATION_SOURCE
                .contains("store_branch_state(request.target_branch.id,")
            && !MERGE_RUNTIME_EXECUTION_FINALIZATION_SOURCE
                .contains("insert_snapshot(\n            merged_snapshot,")
            && !MERGE_RUNTIME_EXECUTION_FINALIZATION_SOURCE
                .contains("insert_snapshot(\r\n            merged_snapshot,")
            && !RUNTIME_STATE_SOURCE.contains("store_branch_state(current.id,"),
        "inactive branch snapshot capture should not clone and re-store a full BranchState after mutating it in place"
    );
    assert!(
        RUNTIME_STATE_SOURCE.contains("AuthorityTransferPacket")
            && RUNTIME_STATE_SOURCE.contains("RestoreTransferPacket")
            && RUNTIME_STATE_SOURCE.contains("ExplicitBranchForkPacket")
            && RUNTIME_STATE_SOURCE
                .contains("pub fn new(branch_id: SignalBranchId, state: BranchState")
            && (RUNTIME_STATE_SOURCE
                .contains("pub fn new(\n        source_branch: SignalBranchId,")
                || RUNTIME_STATE_SOURCE
                    .contains("pub fn new(\r\n        source_branch: SignalBranchId,")),
        "branch lifecycle transfer packets should be mediated through implementation boundaries"
    );
    assert!(
        !RUNTIME_SNAPSHOTTING_SOURCE.contains("RestoreTransferPacket {")
            && !MERGE_RUNTIME_EXECUTION_FINALIZATION_SOURCE.contains("AuthorityTransferPacket {")
            && !RUNTIME_SNAPSHOTTING_SOURCE.contains("AuthorityTransferPacket {")
            && !RUNTIME_STATE_SOURCE.contains("AuthorityTransferPacket { branch_id")
            && !RUNTIME_STATE_SOURCE.contains("RestoreTransferPacket { branch_id")
            && !BRANCHES_SOURCE.contains("pub authority:")
            && !BRANCHES_SOURCE.contains("pub derived:")
            && !BRANCHES_SOURCE.contains("pub ancestry:")
            && !BRANCHES_SOURCE.contains("pub mutation_ledger:")
            && !BRANCHES_SOURCE.contains("pub branch_id:")
            && !BRANCHES_SOURCE.contains("pub parent_branch_id:")
            && !BRANCHES_SOURCE.contains("pub forked_from_snapshot_id:")
            && !BRANCHES_SOURCE.contains("pub latest_merge_reference:")
            && BRANCHES_SOURCE.contains("pub(crate) struct SnapshotStatePacket")
            && BRANCHES_SOURCE.contains("pub fn packet(self, snapshot_id: SignalSnapshotId) -> SnapshotStatePacket"),
        "branch lifecycle transfer packets should not be assembled by open struct literal on runtime paths"
    );
}
