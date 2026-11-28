//! Bind completed Mutation material and the original publication's live inputs.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenialOwnerPort as _;
use std::sync::Arc;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

use branch::history::data::canonical_commit_envelope::live_managed_source_history::backing_bytes;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use contracts::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;
use worth_relational_mutation::internal_port::validation::candidate_source::authored_history::material_capture;
use worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation;

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
    use contracts::transactions::data::outcomes::transaction_commit_error::{
        CommitExecutionDenial, CommitExecutionDenialKind,
    };
    TransactionCommitError::execution(CommitExecutionDenial {
        kind: CommitExecutionDenialKind::Cause(cause),
        partition_identity: None,
    })
}
