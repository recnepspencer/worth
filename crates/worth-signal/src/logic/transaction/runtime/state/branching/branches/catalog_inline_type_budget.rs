//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BranchRuntimeMeta>(
            "crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/catalog.rs",
            "BranchRuntimeMeta",
            128 * 1024,
        );
        owner.field::<BranchAncestryState>("ancestry");
        owner.field::<BranchMutationLedger>("mutation_ledger");
    }
    {
        let mut owner = census.owner::<BranchManager<(), (), ()>>(
            "crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/catalog.rs",
            "BranchManager",
            128 * 1024,
        );
        owner.field::<u64>("owner_runtime_instance_id");
        owner.field::<BTreeMap<SignalBranchId, BranchState<(), (), ()>>>("branches");
        owner.field::<BTreeMap<SignalBranchId, SignalBranchHandle>>("live_branch_catalog");
        owner.field::<BTreeMap<SignalBranchId, BranchRuntimeMeta>>("branch_meta");
        owner
            .field::<BTreeMap<(SignalBranchId, SignalSnapshotId), SnapshotBranchState<(), (), ()>>>(
                "snapshots",
            );
        owner.field::<usize>("maximum_stored_snapshots");
        owner.field::<BTreeMap<SignalBranchId, BTreeSet<SignalBranchId>>>("children_by_parent");
        owner.field::<BTreeMap<SignalBranchId, SignalBranchRetirementReceipt>>(
            "retirement_receipts",
        );
        owner.field::<BTreeSet<SignalBranchId>>("active_merge_participants");
        owner.field::<BTreeMap<SignalBranchId, u64>>("branch_head_generations");
        owner.field::<BTreeMap<SignalBranchId, Option<SignalSnapshotId>>>(
            "branch_restore_snapshot_ids",
        );
        owner.field::<Option<SignalBranchRetentionRegistry>>("retention");
        owner.field::<u32>("next_node_index");
        owner.field::<u64>("next_snapshot_id");
        owner.field::<u64>("next_branch_id");
        owner.field::<u64>("next_lineage_artifact_id");
        owner.field::<u64>("next_lineage_sequence");
    }
}
