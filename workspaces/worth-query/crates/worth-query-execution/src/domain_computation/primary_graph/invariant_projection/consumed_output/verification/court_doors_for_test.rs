//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerification;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::verification::ConsumedOutputVerificationStop;

impl ConsumedOutputEvidence {
    #[doc(hidden)]
    pub fn verify_many_with_admission_for_test(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        roots: &[Self],
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        Self::verify_many_with_admission(execution, roots, owner, runtime, snapshot, selected)
    }
}
