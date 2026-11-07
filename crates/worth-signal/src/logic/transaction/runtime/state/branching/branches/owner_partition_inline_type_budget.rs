//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalOwnerPartitionDenial>("crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/owner_partition.rs", "SignalOwnerPartitionDenial", 128 * 1024);
        owner.field::<usize>("LiveBranchCapacityExhausted.maximum_live_branches");
        owner.field::<usize>("RetirementReceiptCapacityExhausted.maximum_retained_receipts");
    }
    {
        let mut owner = census.owner::<SignalOwnerSnapshotReservationDenial>("crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/owner_partition.rs", "SignalOwnerSnapshotReservationDenial", 128 * 1024);
        owner.field::<usize>("CapacityExhausted.maximum_stored_snapshots");
        owner.field::<SignalSnapshotId>("IdentityExhausted.next_snapshot_id");
    }
    {
        let mut owner = census.owner::<SignalOwnerMetadataState<(), (), ()>>("crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/owner_partition.rs", "SignalOwnerMetadataState", 128 * 1024);
        owner
            .field::<BTreeMap<(SignalBranchId, SignalSnapshotId), SnapshotBranchState<(), (), ()>>>(
                "snapshots",
            );
        owner.field::<usize>("maximum_stored_snapshots");
        owner.field::<BTreeMap<SignalBranchId, BTreeSet<SignalBranchId>>>("children_by_parent");
        owner.field::<BTreeSet<SignalBranchId>>("retirement_reservations");
        owner.field::<BTreeMap<SignalBranchId, SignalBranchRetirementReceipt>>(
            "retirement_receipts",
        );
        owner.field::<BTreeSet<SignalBranchId>>("active_merge_participants");
        owner.field::<usize>("maximum_retirement_receipts");
        owner.field::<usize>("reserved_retirement_receipt_count");
        owner.field::<u64>("next_snapshot_id");
    }
    {
        let mut owner = census.owner::<SignalOwnerPartition<(), (), ()>>("crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/owner_partition.rs", "SignalOwnerPartition", 128 * 1024);
        owner.field::<SignalOwnerMetadataState<(), (), ()>>("metadata");
        owner.field::<u64>("next_branch_id");
        owner.field::<SignalBranchRetentionRegistry>("retention");
        owner.field::<SignalBranchId>("selected_branch_id");
        owner.field::<Vec<SignalOwnerCellSeed<(), (), ()>>>("cells");
        owner.field::<BTreeMap<SignalBranchId, crate::durability::SignalDurableBranchDescriptor>>(
            "readmitted_coordinates",
        );
    }
}
