//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalBranchCellState<(), (), ()>>(
            "crates/worth-signal/src/branch/owner_services/branch_cell_state.rs",
            "SignalBranchCellState",
            128 * 1024,
        );
        owner.field::<SignalBranchHandle>("handle");
        owner.field::<u64>("owner_runtime_instance_id");
        owner.field::<u64>("definition_basis");
        owner.field::<BranchState<(), (), ()>>("state");
        owner.field::<u64>("head_generation");
        owner.field::<Option<SignalSnapshotId>>("restore_snapshot_id");
        owner.field::<Option<SignalPublishedConditionalExecutionBasis>>(
            "conditional_execution_basis",
        );
    }
}
