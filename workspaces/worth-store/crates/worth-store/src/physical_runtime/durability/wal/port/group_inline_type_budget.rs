//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PhysicalWalGroupAppendOutcome>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/wal/port/group.rs", "PhysicalWalGroupAppendOutcome", 128 * 1024);
        owner.field::<NonEmpty<PreparedPhysicalMutation>>("NotAdmitted.members");
        owner.field::<PhysicalWalGroupAppendFailureCause>("NotAdmitted.cause");
    }
    {
        let _owner = census.owner::<PhysicalWalGroupAppendFailureCause>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/wal/port/group.rs", "PhysicalWalGroupAppendFailureCause", 128 * 1024);
    }
    {
        let mut owner = census.owner::<PhysicalWalGroupAppendContinuation>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/wal/port/group.rs", "PhysicalWalGroupAppendContinuation", 128 * 1024);
        owner.field::<PhysicalDurabilityGroupBasis>("basis");
        owner.field::<Vec<WalBarrierMember<WalAppendedPhysicalMutation>>>("appended");
        owner.field::<PhysicalWalGroupAppendRemainder>("remaining");
        owner.field::<PhysicalWalGroupAppendFailureCause>("cause");
    }
    {
        let _owner = census.owner::<PhysicalWalGroupAppendRemainder>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/wal/port/group.rs", "PhysicalWalGroupAppendRemainder", 128 * 1024);
    }
    {
        let mut owner = census.owner::<IndeterminatePhysicalWalGroupAppend>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/wal/port/group.rs", "IndeterminatePhysicalWalGroupAppend", 128 * 1024);
        owner.field::<PhysicalDurabilityGroupBasis>("basis");
        owner.field::<IndeterminatePhysicalWalGroupAppendState>("state");
    }
    {
        let mut owner = census.owner::<IndeterminatePhysicalWalGroupAppendState>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/wal/port/group.rs", "IndeterminatePhysicalWalGroupAppendState", 128 * 1024);
        owner.field::<Vec<WalBarrierMember<WalAppendedPhysicalMutation>>>("Member.appended");
        owner.field::<WalBarrierMember<WalRangeReservedPhysicalMutation>>("Member.uncertain");
        owner.field::<Vec<WalBarrierMember<WalRangeReservedPhysicalMutation>>>("Member.remaining");
        owner.field::<Vec<WalBarrierMember<WalAppendedPhysicalMutation>>>("Sealing.appended");
        owner.field::<PhysicalDurabilityGroupSealingDenial>("Sealing.cause");
    }
}
