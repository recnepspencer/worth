//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PhysicalRecoveryPlanningDenial>("workspaces/worth-store/crates/worth-store-recovery-runtime/src/entry/outcome/planning_denial.rs", "PhysicalRecoveryPlanningDenial", 128 * 1024);
        owner.field::<u64>("ResolvedStateAllocation.requested_bytes");
        owner.field::<std::collections::TryReserveError>("ResolvedStateAllocation.cause");
        owner.field::<u64>("PlannedStateAllocation.requested_bytes");
        owner.field::<std::collections::TryReserveError>("PlannedStateAllocation.cause");
        owner.field::<u64>("PublicationCandidateAllocation.requested_bytes");
        owner.field::<std::collections::TryReserveError>("PublicationCandidateAllocation.cause");
        owner.field::<u64>("ExecutionImageAllocation.requested_bytes");
        owner.field::<std::collections::TryReserveError>("ExecutionImageAllocation.cause");
    }
}
