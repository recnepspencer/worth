//! The prepared candidate lends only its actual canonical publication origins.
use super::*;
use worth_relational_contracts::internal_port as contracts;
impl PreparedRelationalCommitCandidate {
    pub fn prepare_required_output_publications(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority,
    ) -> Result<
        crate::mvcc::publication::application_publication::custody::RelationalPreparedOutputPublications,
        contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    >{
        if self.runtime_instance_id != installed.runtime_instance_id {
            return Err(
                contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "publication custody belongs to another Native runtime",
                ),
            );
        }
        let payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let payload = payload.as_ref().ok_or_else(|| {
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "prepared publication candidate is unavailable",
            )
        })?;
        crate::mvcc::publication::application_publication::prepare_publications(
            execution,
            installed,
            payload.execution.canonical_envelope(),
        )
    }
}
