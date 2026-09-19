//! The published host paired with the initial program that governed its
//! installation, and with every other program it rostered.

use std::any::TypeId;

use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationProgram;

use super::supported_program::{
    WorthQuerySupportedProgramHandle, WorthQuerySupportedProgramRecord,
};
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryPresentedProgram;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// Runtime paired with the exact validated program that governed installation.
pub struct WorthQueryProgramApplicationRuntime<Schema, Program> {
    pub(in crate::domain_computation::primary_graph) runtime:
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(in crate::domain_computation::primary_graph) program:
        WorthQueryInstalledApplicationProgram<Schema, Program>,
    pub(in crate::domain_computation::primary_graph) connection_types: Box<[TypeId]>,
    pub(in crate::domain_computation::primary_graph) root_graph_types: Box<[TypeId]>,
    pub(in crate::domain_computation::primary_graph) output_source_bindings: Box<[TypeId]>,
    pub(in crate::domain_computation::primary_graph) action_bindings: Box<[TypeId]>,
    pub(in crate::domain_computation::primary_graph) supported:
        Box<[WorthQuerySupportedProgramRecord]>,
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program> {
    pub const fn runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        &self.runtime
    }

    pub const fn installed_program(
        &self,
    ) -> &WorthQueryInstalledApplicationProgram<Schema, Program> {
        &self.program
    }

    pub(crate) fn contains_connection_type<Connection: 'static>(&self) -> bool {
        self.connection_types.contains(&TypeId::of::<Connection>())
    }

    pub fn contains_output_root<Root>(&self) -> bool
    where
        Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
        Root: ApplicationOutputGraphShape<Schema>,
    {
        self.root_graph_types.contains(&TypeId::of::<Root>())
    }

    pub fn contains_action<Binding: 'static>(&self) -> bool {
        self.program.contains_action_type::<Binding>()
    }

    pub(in crate::domain_computation::primary_graph) fn declares_action_binding(
        &self,
        binding: TypeId,
    ) -> bool {
        self.action_bindings.contains(&binding)
    }

    pub(in crate::domain_computation::primary_graph) fn requires_output_source(
        &self,
        binding: TypeId,
    ) -> bool {
        self.output_source_bindings.contains(&binding)
    }

    /// Resolves this host's initial program against its own support roster.
    ///
    /// A host that rostered no program presents nothing, which is how an
    /// installation that never admitted program meaning refuses program-gated
    /// work instead of inventing an owner for it.
    pub(in crate::domain_computation::primary_graph) fn presented_program(
        &self,
    ) -> Option<WorthQueryPresentedProgram<'_>> {
        self.runtime
            .installed_program_support()?
            .present(self.program.revision())
    }

    /// Names one other program this host rostered by its authoring type.
    ///
    /// The handle exists only when that program passed rostered installation on
    /// this exact host, so no caller can name meaning the roster never carried.
    pub fn supported_program<Supported>(
        &self,
    ) -> Option<WorthQuerySupportedProgramHandle<'_, Schema, Supported>>
    where
        Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
        Supported: ApplicationProgramDefinition<Schema> + 'static,
    {
        let record = self
            .supported
            .iter()
            .find(|record| record.answers_to(TypeId::of::<Supported>()))?;
        Some(WorthQuerySupportedProgramHandle::rostered(
            &self.runtime,
            record,
        ))
    }
}

impl<Schema, Program> std::ops::Deref for WorthQueryProgramApplicationRuntime<Schema, Program> {
    type Target = WorthQueryPrimaryGraphApplicationRuntime<Schema>;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn close_conditional_runtime(
        &mut self,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryConditionalRuntimeInspection,
        crate::domain_computation::primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        self.runtime.close_conditional_runtime()
    }
}
