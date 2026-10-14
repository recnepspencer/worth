//! Resolve one occurrence-bound program owner.
use super::*;
use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;

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
    ///         action_inventory: todo!(),
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
        if !std::ptr::eq(selected.application(), &self.backing.runtime) {
            return Err(WorthQuerySelectedProgramOwnerDenial::ProductSelection(
                crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(
                    AdmissionRefusal::ForeignOwner,
                ),
            ));
        }
        let inspection = selected
            .inspect_selected_program()
            .map_err(WorthQuerySelectedProgramOwnerDenial::Inspection)?;
        let revision = inspection.revision();
        if revision == self.backing.program.revision() {
            return Ok(WorthQuerySelectedProgramOwner {
                runtime: &self.backing.runtime,
                revision: self.backing.program.revision(),
                action_bindings: &self.backing.action_bindings,
                output_source_bindings: &self.backing.output_source_bindings,
                root_graph_types: &self.backing.root_graph_types,
                action_inventory: self.backing.program.action_inventory(),
            });
        }
        let record = self
            .backing
            .supported
            .iter()
            .find(|record| record.revision() == revision)
            .ok_or(WorthQuerySelectedProgramOwnerDenial::InstalledOwnerUnavailable)?;
        Ok(WorthQuerySelectedProgramOwner {
            runtime: &self.backing.runtime,
            revision: record.revision(),
            action_bindings: &record.action_bindings,
            output_source_bindings: &record.output_source_bindings,
            root_graph_types: &record.root_graph_types,
            action_inventory: record.installed.action_inventory(),
        })
    }
}

impl<Schema: ApplicationSchema> WorthQuerySelectedProgramOwner<'_, Schema> {
    pub fn resolve_fixed_action<Binding, Action>(
        &self,
    ) -> Result<
        &worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
        worth_query_installation::facade::WorthQueryActionResolutionDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
        Action: worth_query_declaration::facade::application_program::ApplicationFixedAction<
            Schema,
            Binding,
        >,
    {
        self.action_inventory
            .resolve_fixed_action::<Schema, Binding, Action>()
    }
}

impl<Schema: ApplicationSchema> WorthQuerySelectedProgramOwner<'_, Schema> {
    pub fn resolve_declared_action<Binding: ApplicationMutationBinding<Schema>>(
        &self,
        declared: &worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
    ) -> Result<
        &worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
        worth_query_installation::facade::WorthQueryActionResolutionDenial,
    > {
        self.action_inventory
            .resolve_declared_action::<Schema, Binding>(declared)
    }
}
