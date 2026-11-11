//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<RecoveryHandoffBacking>("workspaces/worth-store/crates/worth-store-recovery-runtime/src/orchestration/handoff/backing.rs", "RecoveryHandoffBacking", 128 * 1024);
        owner.field::<Vec<PendingCompletion>>("pending");
        owner.field::<Vec<RecoveredPhysicalRuntimeCore>>("core");
    }
}
