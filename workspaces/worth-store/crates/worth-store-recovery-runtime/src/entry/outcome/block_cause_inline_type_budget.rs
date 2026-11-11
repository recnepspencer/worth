//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PhysicalRecoveryBlockCause>("workspaces/worth-store/crates/worth-store-recovery-runtime/src/entry/outcome/block_cause.rs", "PhysicalRecoveryBlockCause", 128 * 1024);
        owner.field::<super::super::PhysicalRecoveryBlockKind>("Limit.phase");
        owner.field::<PhysicalRecoveryLimitFailure>("Limit.limit");
    }
}
