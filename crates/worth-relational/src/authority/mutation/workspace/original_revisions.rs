use super::*;
impl MutationWorkspace<'_> {
    pub(crate) fn write_original_binding_revision_stamps(
        &mut self,
        target: crate::transactions::data::RecordRef,
        binding: &crate::schema::data::LoweredAspectContractBinding,
        evidence: &super::super::canonical_deltas::BorrowedAspectDeltaEvidence<'_>,
        structural: crate::publication::patch::data::RecordStructuralChange,
        changed: bool,
        version: VersionId,
    ) -> Result<(), super::super::MutationApplyDenial> {
        if changed {
            let aspect = self
                .symbols
                .reserve_revision_reference(binding.aspect_key().as_str())?;
            super::super::aspect_versions::write_original_aspect_version(
                &self.execution,
                self.state,
                target.clone(),
                aspect,
                version,
            )
            .map_err(|cause| {
                crate::transactions::data::TransactionCommitError::execution(cause.into())
            })?;
        }
        super::super::field_versions::write_original_field_versions_for_binding(
            &self.execution,
            self.state,
            target,
            binding,
            evidence,
            structural,
            self.symbols,
            version,
        )
    }
}
