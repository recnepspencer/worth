//! Bind completed Mutation material and the original publication's live inputs.
use std::sync::Arc;

use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::history::data::{
    canonical_commit_envelope::live_managed_source_history::backing_bytes, CanonicalCommitEnvelope,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::validation::candidate_source::{
    authored_history::material_capture, NativeManagedSourceInterpretation,
};
use worth_execution::ExecutionRequest;

pub(super) fn freeze(
    envelope: CanonicalCommitEnvelope,
    interpretation: &NativeManagedSourceInterpretation,
    execution: ExecutionRequest<'_, '_>,
) -> Result<Arc<CanonicalCommitEnvelope>, TransactionCommitError> {
    let frozen = match envelope
        .select_managed_source_history(interpretation.prepared_source_impact_history())
    {
        Ok(selected) => {
            let material = material_capture::capture(execution, interpretation, selected.packet())
                .map_err(|cause| {
                    let basis = interpretation.physical_basis();
                    cause.into_commit_error(
                        basis.identity().branch_id().clone(),
                        basis.observation().commit_id().map(|id| id.0),
                        basis.observation().version_id(),
                    )
                })?;
            let bytes = backing_bytes().ok_or_else(|| storage(Cause::ChargedBytesOverflow))?;
            let memory = execution
                .reserve_retained_memory(bytes)
                .map_err(|cause| storage(cause.into()))?;
            execution.consult().map_err(|cause| storage(cause.into()))?;
            selected.freeze(material, memory)
        }
        Err(frozen) => frozen,
    };
    Ok(frozen)
}

fn storage(cause: Cause) -> TransactionCommitError {
    use crate::transactions::data::outcomes::transaction_commit_error::{
        CommitExecutionDenial, CommitExecutionDenialKind,
    };
    TransactionCommitError::execution(CommitExecutionDenial {
        kind: CommitExecutionDenialKind::Cause(cause),
        partition_identity: None,
    })
}
