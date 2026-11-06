//! The prepared candidate lends only its actual canonical publication origins.
use super::*;
impl PreparedRelationalCommitCandidate {
    pub fn prepare_required_output_publications(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    ) -> Result<
        crate::mvcc::RelationalPreparedOutputPublications,
        crate::transactions::data::TransactionCommitError,
    > {
        if self.runtime_instance_id != installed.runtime_instance_id {
            return Err(crate::mvcc::origin_denial(
                "publication custody belongs to another Native runtime",
            ));
        }
        let payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let payload = payload.as_ref().ok_or_else(|| {
            crate::mvcc::origin_denial("prepared publication candidate is unavailable")
        })?;
        crate::mvcc::publication::application_publication::prepare_publications(
            execution,
            installed,
            payload.execution.canonical_envelope(),
        )
    }
}
