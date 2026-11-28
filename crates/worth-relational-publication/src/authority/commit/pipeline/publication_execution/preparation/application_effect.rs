//! Judge the actual applied body before current-publication bookkeeping.
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatchOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use contracts::transactions::data::primitives::RecordRef;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn unchanged(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    changed: &[RecordRef],
    adjacency: &[contracts::authority::mutation::adjacency_delta::AdjacencyDelta],
    envelope: &branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody,
    symbols: &worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings,
) -> Result<bool, TransactionCommitError> {
    if !adjacency.is_empty()
        || !(*envelope.owner_content()).records().is_empty()
        || !envelope.record_allocations().is_empty()
        || !envelope.cancelled_record_allocations().is_empty()
        || symbols.has_reserved_meanings()
        || envelope.schema_transition.is_some()
        || envelope.schema_continuation_descriptor.is_some()
        || envelope.schema_reconciliation_descriptor.is_some()
    {
        return Ok(false);
    }
    execution
        .run_native_with_checkpoint(|context| {
            let patches = &envelope.patch.authoritative_record_patches;
            for patch in patches {
                contracts::execution::request_stage::checkpoint(context, 1).map_err(stop)?;
                if !patch.preserves_application() {
                    return Ok(false);
                }
            }
            for record in changed {
                contracts::execution::request_stage::checkpoint(context, patches.len() as u64 + 1)
                    .map_err(stop)?;
                if !patches.iter().any(|p| &p.target == record) {
                    return Ok(false);
                }
            }
            Ok(true)
        })
        .map_err(stop)?
        .0
}
fn stop(
    cause: impl Into<contracts::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
