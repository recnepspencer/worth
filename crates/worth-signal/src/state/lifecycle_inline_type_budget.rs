//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalBranchHandle>(
            "crates/worth-signal/src/state/lifecycle.rs",
            "SignalBranchHandle",
            128 * 1024,
        );
        owner.field::<SignalBranchId>("id");
        owner.field::<String>("name");
        owner.field::<Option<SignalBranchId>>("parent_branch_id");
        owner.field::<Option<SignalSnapshotId>>("head_snapshot_id");
    }
}
