//! Judge the actual applied body before current-publication bookkeeping.
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;

pub(super) fn unchanged(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    changed: &[RecordRef],
    adjacency: &[crate::authority::mutation::AdjacencyDelta],
    envelope: &crate::history::data::CanonicalCommitEnvelopeBody,
    symbols: &crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings,
    merge_base: &[crate::history::data::CommitId],
    merge_parents: &[crate::history::data::BranchId],
) -> Result<bool, TransactionCommitError> {
    if !adjacency.is_empty()
        || !envelope.content.records().is_empty()
        || !envelope.record_allocations().is_empty()
        || symbols.has_reserved_meanings()
        || envelope.schema_transition.is_some()
        || envelope.schema_continuation_descriptor.is_some()
        || envelope.schema_reconciliation_descriptor.is_some()
        || !merge_base.is_empty()
        || !merge_parents.is_empty()
    {
        return Ok(false);
    }
    execution
        .run_native_with_checkpoint(|context| {
            let patches = &envelope.patch.authoritative_record_patches;
            for patch in patches {
                crate::execution::request_stage::checkpoint(context, 1).map_err(stop)?;
                if !patch.preserves_application() {
                    return Ok(false);
                }
            }
            for record in changed {
                crate::execution::request_stage::checkpoint(context, patches.len() as u64 + 1)
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
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
