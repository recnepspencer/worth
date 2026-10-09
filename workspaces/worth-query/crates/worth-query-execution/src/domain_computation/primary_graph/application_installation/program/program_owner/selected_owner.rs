//! Resolve one occurrence-bound program owner.
use super::*;

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
{
    /// Resolves the installed commit owner carried by one exact branch.
    ///
    /// ```compile_fail,E0451
    /// use worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwner;
    ///
    /// fn a_revision_cannot_forge_an_owner<Schema>() {
    ///     let _owner = WorthQuerySelectedProgramOwner::<Schema> {
    ///         runtime: todo!(),
    ///         revision: todo!(),
    ///         action_bindings: &[],
    ///         output_source_bindings: &[],
    ///         root_graph_types: &[],
    ///     };
    /// }
    /// ```
    pub fn selected_program_owner(
        &self,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Result<WorthQuerySelectedProgramOwner<'_, Schema>, WorthQuerySelectedProgramOwnerDenial>
    {
        if !std::ptr::eq(selected.application(), &self.runtime) {
            return Err(WorthQuerySelectedProgramOwnerDenial::ProductSelection(
                crate::basis::WorthQueryProductBranchAdmissionDenial::ForeignOwner,
            ));
        }
        let inspection = selected
            .inspect_selected_program()
            .map_err(WorthQuerySelectedProgramOwnerDenial::Inspection)?;
        let revision = inspection.revision();
        if revision == self.program.revision() {
            return Ok(WorthQuerySelectedProgramOwner {
                runtime: &self.runtime,
                revision: self.program.revision(),
                action_bindings: &self.action_bindings,
                output_source_bindings: &self.output_source_bindings,
                root_graph_types: &self.root_graph_types,
            });
        }
        let record = self
            .supported
            .iter()
            .find(|record| record.revision() == revision)
            .ok_or(WorthQuerySelectedProgramOwnerDenial::InstalledOwnerUnavailable)?;
        Ok(WorthQuerySelectedProgramOwner {
            runtime: &self.runtime,
            revision: record.revision(),
            action_bindings: &record.action_bindings,
            output_source_bindings: &record.output_source_bindings,
            root_graph_types: &record.root_graph_types,
        })
    }
}
