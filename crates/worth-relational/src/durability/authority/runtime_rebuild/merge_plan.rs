use crate::history::data::CanonicalCommitEnvelope;
use crate::mvcc::OwnedMergePlan;
use crate::transactions::data::TransactionCommitError;

pub(super) fn merge_commit_mutation_plan_from_envelope(
    envelope: &CanonicalCommitEnvelope,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Option<OwnedMergePlan>, TransactionCommitError> {
    OwnedMergePlan::copy_recovered(envelope, request)
}
