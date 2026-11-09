//! Currentness of a retained owned publication uses the current-fact owner.
use super::*;
impl RelationalRequiredOutputPublication {
    /// Compare this origin with the authenticated roster and independent route
    /// presence at the exact basis. No historical route reader is a fallback.
    pub fn matches_current_ownership_at(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &super::super::super::RelationalInstalledApplicationPublicationAuthority,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
    ) -> Result<bool, TransactionCommitError> {
        if !self.origin.source_scope_owned {
            return Ok(false);
        }
        if installed.runtime_instance_id != self.runtime_instance_id
            || basis.descriptor().runtime_instance_id() != self.runtime_instance_id
            || !installed
                .authority
                .owns_required_output_publication(&self.authority)
        {
            return Ok(false);
        }
        installed.require_current_ownership_storage(basis, self.origin.meaning.source_scope)?;
        let roster = crate::publication::ownership::select_current(
            execution,
            basis,
            &[RecordRef::Entity(self.origin.meaning.source_scope)],
        )?;
        execution
            .run_native_with_checkpoint(|context| {
                for origin in roster.origins() {
                    checkpoint(
                        context,
                        (origin.producer.len()
                            + self.origin.producer.len()
                            + origin.meaning.family.len()
                            + self.origin.meaning.family.len()
                            + 96) as u64,
                    )
                    .map_err(execution_stop)?;
                    if origin.commit.commit_id == self.origin.commit.commit_id
                        && origin.producer == self.origin.producer
                        && origin.dependency_identity == self.origin.dependency_identity
                        && origin.meaning == self.origin.meaning
                    {
                        return Ok(true);
                    }
                }
                Ok(false)
            })
            .map_err(execution_stop)?
            .0
    }
}
