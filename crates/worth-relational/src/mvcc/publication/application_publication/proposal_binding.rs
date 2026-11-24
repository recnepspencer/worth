//! Bind required output publication to a live validated proposal.
use super::*;

impl crate::mvcc::validation::validated_proposal::ValidatedRelationalProposal {
    /// Bind a running installed producer before immutable publication preparation.
    /// Target references remain untrusted selectors until native allocation and
    /// canonical operation/payload coverage have been checked.
    /// The real scoped assembly has already bound each admitted writer's
    /// publication. An outer authored origin must not bind it a second time.
    pub fn has_coupled_authored_publication(&self) -> bool {
        self.state
            .required_output_publication
            .as_ref()
            .is_some_and(|admission| {
                matches!(
                    admission.publication,
                    RequiredOutputPublication::CoupledAuthored { .. }
                )
            })
    }

    pub fn bind_required_output_publication(
        mut self,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        publication: RequiredOutputPublication,
    ) -> Result<Self, TransactionCommitError> {
        if self.state.required_output_publication.is_some()
            || !publication.admitted_by(installed)
            || self
                .state
                .validation_input
                .basis()
                .descriptor()
                .runtime_instance_id()
                != installed.runtime_instance_id
            || self.state.custom_invariant_generation != installed.custom_invariant_generation
        {
            return Err(origin_denial(
                "required-output publication has a foreign or repeated installed owner",
            ));
        }
        if !installed.source_scope_ownership.is_empty()
            && let Some(source) = publication.ownership_subject(installed)
        {
            installed
                .require_current_ownership_storage(self.state.validation_input.basis(), source)?;
        }
        self.state.required_output_publication = Some(RequiredOutputPublicationAdmission {
            inputs: Default::default(),
            installed: installed.clone(),
            publication,
        });
        Ok(self)
    }
}
