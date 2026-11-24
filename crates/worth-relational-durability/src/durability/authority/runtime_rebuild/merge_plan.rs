//! Validate the recorded merge plan source before its request funds a copy.
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::mvcc::transaction::reconstructed_merge_plan::{
    OwnedMergePlan, RecordedMergePlanSource,
};

pub fn prepare_recovered_merge_plan(
    envelope: &CanonicalCommitEnvelope,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<OwnedMergePlan, RelationalRecoveryStop> {
    request
        .consult()
        .map_err(|cause| copy_stop(TransactionCommitError::execution(cause.into())))?;
    let source = require_merge_source(envelope)?;
    OwnedMergePlan::copy_recovered(source, request).map_err(copy_stop)
}

fn copy_stop(error: TransactionCommitError) -> RelationalRecoveryStop {
    super::envelope_replay::commit_stop::replay_commit_stop(error, "merge plan copy denied".into())
}

fn require_merge_source(
    envelope: &CanonicalCommitEnvelope,
) -> Result<RecordedMergePlanSource<'_>, RelationalRecoveryStop> {
    RecordedMergePlanSource::from_envelope(envelope)
        .ok_or_else(|| missing_merge_authority(envelope).into())
}

pub fn missing_merge_authority(envelope: &CanonicalCommitEnvelope) -> DurabilityError {
    DurabilityError::new(
        RecoveryFailureClass::ReplayFailure,
        format!(
            "failed to reconstruct merge execution summary for durable merge commit {}",
            envelope.commit.commit_id.0
        ),
    )
}

#[cfg(test)]
pub mod tests;
