//! The runtime's consumer methods over `domain_computation::primary_graph::application_installation`; each only calls its subsystem.
use worth_query_installation::facade::ApplicationSchema;
impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn selected_program_owner(
        &self,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Result<crate::domain_computation::primary_graph::application_installation::program::WorthQuerySelectedProgramOwner<'_, Schema>, crate::domain_computation::primary_graph::vocabulary::application_installation::program_owner::WorthQuerySelectedProgramOwnerDenial>{
        <Self as crate::domain_computation::primary_graph::application_installation::SelectedOwnerOps<Schema>>::selected_program_owner(self, selected)
    }
}
