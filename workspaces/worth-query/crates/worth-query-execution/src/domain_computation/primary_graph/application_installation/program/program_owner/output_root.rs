//! Resolve the requested typed output root through the actual selected owner.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_declaration::facade::application_program::ApplicationOutputGraphShape;
impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn validate_selected_output_root<R: ApplicationOutputGraphShape<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        observation: &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
    ) -> Result<(), WorthQuerySelectedProgramOwnerDenial> {
        use WorthQuerySelectedProgramOwnerDenial as Denial;
        let selected = self
            .runtime()
            .select_application_read_observation(phase, observation)
            .map_err(Denial::ProductSelection)?;
        let owner = self.selected_program_owner(&selected)?;
        if !owner.owns_output_root(TypeId::of::<R>()) {
            return Err(Denial::UndeclaredOutputRoot);
        }
        Ok(())
    }
}
