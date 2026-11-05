//! Resolve one occurrence-bound program owner.
use super::*;
use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use crate::domain_computation::primary_graph::vocabulary::application_installation::program_owner::WorthQuerySelectedProgramOwnerDenial;

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
        self.runtime().selected_program_owner(selected)
    }
}

pub trait SelectedOwnerOps<Schema: ApplicationSchema>: crate::domain_computation::primary_graph::application_installation::host::ApplicationInstallationHost<Schema> {
fn selected_program_owner(
        &self,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Result<WorthQuerySelectedProgramOwner<'_, Schema>, WorthQuerySelectedProgramOwnerDenial>
    {
        if !std::ptr::addr_eq(selected.application(), self) {
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
        let record = self.installed_program_support()
            .and_then(|support| support.native_program(revision))
            .ok_or(WorthQuerySelectedProgramOwnerDenial::InstalledOwnerUnavailable)?;
        Ok(WorthQuerySelectedProgramOwner {
            runtime: self.application_installation_host(),
            revision: record.revision(),
            action_bindings: &record.action_bindings,
            output_source_bindings: &record.output_source_bindings,
            root_graph_types: &record.root_graph_types,
            action_inventory: record.installed.action_inventory(),
        })
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_installation::host::ApplicationInstallationHost<Schema> + ?Sized> SelectedOwnerOps<Schema> for HostRuntime {}

impl<Schema: ApplicationSchema> WorthQuerySelectedProgramOwner<'_, Schema> {
    pub fn resolve_action<Binding, Action>(
        &self,
    ) -> Result<
        &worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
        worth_query_installation::facade::WorthQueryActionResolutionDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
        Action: worth_query_declaration::facade::application_program::ApplicationAction<
            Schema,
            Binding,
        >,
    {
        self.action_inventory
            .resolve_action::<Schema, Binding, Action>()
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
