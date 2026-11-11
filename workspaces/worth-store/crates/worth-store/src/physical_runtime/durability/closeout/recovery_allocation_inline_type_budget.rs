//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PhysicalRecoveryAllocationAdmission>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/closeout/recovery_allocation.rs", "PhysicalRecoveryAllocationAdmission", 128 * 1024);
        owner.field::<StableStoreIdentity>("store");
        owner.field::<u64>("byte_limit");
    }
}
