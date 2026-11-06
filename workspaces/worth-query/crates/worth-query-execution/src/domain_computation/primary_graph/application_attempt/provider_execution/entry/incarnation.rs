//! A carried effect program is checked before current-owner policy or I/O.
use super::*;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn require_program_incarnation<Operation, Input, Scope>(
        &self,
        program: &WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryApplicationCommitOutcome> {
        program.refuse_stale_home_authority(self.runtime.product_runtime.owner.owner_identity())
            .map_err(|denial| WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::request_authority_denied(
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::BasisAdmission,
                    &denial)))
    }
}
