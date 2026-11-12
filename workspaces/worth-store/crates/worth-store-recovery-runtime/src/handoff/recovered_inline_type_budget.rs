//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<RecoveredPhysicalRuntimeHandoff>(
            "workspaces/worth-store/crates/worth-store-recovery-runtime/src/handoff/recovered.rs",
            "RecoveredPhysicalRuntimeHandoff",
            128 * 1024,
        );
        owner.field::<RecoveredPhysicalRuntimeCore>("core");
        owner.field::<Box<RecoveredPhysicalRuntimeHandoffEvidence>>("evidence");
    }
}
