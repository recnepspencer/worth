use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryApplicationRequiredOutputSource,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationDiscoveredOutputGraph, ApplicationOutputGraph,
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramOutputRootsShape,
    ApplicationProgramOutputs, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::{
    install_application_program, WorthQueryInstalledApplicationProgram,
};

use super::{
    in_memory_with_contributions, WorthQueryInMemoryApplicationDenial,
    WorthQueryInMemoryApplicationLimits,
};

mod conditional;
mod demand;
mod derived_artifact;
mod output_source;
mod specialized_action;
mod speculation;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationContributionTuple, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};
pub use demand::{
    WorthQueryAdmittedProgramOutput, WorthQueryProgramOutputAdvance, WorthQueryProgramRootDemand,
    WorthQuerySettledProgramOutput,
};
pub use specialized_action::WorthQueryAdmittedProgramOperation;
pub use speculation::{
    WorthQueryApplicationPreviewReadmissionDenial, WorthQueryApplicationPreviewRequest,
    WorthQueryApplicationPreviewSession, WorthQueryReadmittedApplicationPreview,
};

type RootConnectionRef<Schema, Root> =
    <Root as ApplicationOutputGraphShape<Schema>>::RootConnection;
type RootConnection<Schema, Root> =
    <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding;
pub trait WorthQueryApplicationProgramRoots<Schema>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    fn append_required_bindings(bindings: &mut std::collections::BTreeSet<std::any::TypeId>);
}

impl<Schema, Root, Dependents> WorthQueryApplicationProgramRoots<Schema>
    for ApplicationOutputGraph<Root, Dependents>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Self: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Self>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    fn append_required_bindings(bindings: &mut std::collections::BTreeSet<std::any::TypeId>) {
        bindings.insert(std::any::TypeId::of::<
            <RootConnection<Schema, Self> as WorthQueryApplicationRequiredOutputConnection<
                Schema,
            >>::Source,
        >());
    }
}

impl<Schema, Root, Dependents> WorthQueryApplicationProgramRoots<Schema>
    for ApplicationDiscoveredOutputGraph<Root, Dependents>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Self: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Self>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    fn append_required_bindings(bindings: &mut std::collections::BTreeSet<std::any::TypeId>) {
        bindings.insert(std::any::TypeId::of::<
            <RootConnection<Schema, Self> as WorthQueryApplicationDiscoveredOutputConnection<
                Schema,
            >>::Source,
        >());
    }
}

impl<Schema, Left, Right> WorthQueryApplicationProgramRoots<Schema> for (Left, Right)
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Left: WorthQueryApplicationProgramRoots<Schema>,
    Right: WorthQueryApplicationProgramRoots<Schema>,
{
    fn append_required_bindings(bindings: &mut std::collections::BTreeSet<std::any::TypeId>) {
        Left::append_required_bindings(bindings);
        Right::append_required_bindings(bindings);
    }
}

impl<Schema, Roots> WorthQueryApplicationProgramRoots<Schema> for ApplicationProgramOutputs<Roots>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Roots: ApplicationProgramOutputRootsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    fn append_required_bindings(bindings: &mut std::collections::BTreeSet<std::any::TypeId>) {
        Roots::append_required_bindings(bindings);
    }
}

/// Runtime paired with the exact validated program that governed installation.
pub struct WorthQueryProgramApplicationRuntime<Schema, Program> {
    runtime: WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    program: WorthQueryInstalledApplicationProgram<Schema, Program>,
    connection_types: Box<[std::any::TypeId]>,
    root_graph_types: Box<[std::any::TypeId]>,
    output_source_bindings: Box<[std::any::TypeId]>,
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
        self.connection_types
            .contains(&std::any::TypeId::of::<Connection>())
    }

    pub fn contains_output_root<Root>(&self) -> bool
    where
        Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
        Root: ApplicationOutputGraphShape<Schema>,
    {
        self.root_graph_types
            .contains(&std::any::TypeId::of::<Root>())
    }

    pub fn contains_action<Binding: 'static>(&self) -> bool {
        self.program.contains_action_type::<Binding>()
    }
}

impl<Schema> WorthQueryApplicationProgramRoots<Schema>
    for worth_query_declaration::facade::application_program::ApplicationNoOutputGraph
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    fn append_required_bindings(_: &mut std::collections::BTreeSet<std::any::TypeId>) {}
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn compare_and_commit_program_action<Binding>(
        &self,
        program: crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome
    where
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        if !self.program.contains_action_type::<Binding>()
            || self
                .output_source_bindings
                .contains(&std::any::TypeId::of::<Binding>())
        {
            return crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        self.runtime
            .compare_and_commit_application_for_program_action(program, idempotency)
    }

    pub fn compare_and_commit_program_action_retained<Binding>(
        &self,
        program: crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationRetainedCommitOutcome
    where
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        if !self.program.contains_action_type::<Binding>()
            || self
                .output_source_bindings
                .contains(&std::any::TypeId::of::<Binding>())
        {
            return crate::domain_computation::primary_graph::WorthQueryApplicationRetainedCommitOutcome::Other(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Denied(
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::application_program_required(),
                ),
            );
        }
        let outcome = self
            .runtime
            .compare_and_commit_application_for_program_action(
                program.with_client_observation(),
                idempotency,
            );
        self.runtime.retained_commit_outcome(outcome)
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

/// Validates and installs program meaning before exposing its application runtime.
pub fn in_memory_program<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    in_memory_program_with_optional_authorization_time_source(
        program,
        declaration,
        configuration,
        limits,
        initial_state,
        None,
    )
}

/// Validates and installs program meaning with one host-owned trusted-time
/// source fixed before publication.
pub fn in_memory_program_with_authorization_time_source<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    source: impl crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    in_memory_program_with_optional_authorization_time_source(
        program,
        declaration,
        configuration,
        limits,
        initial_state,
        Some(Box::new(source)),
    )
}

fn in_memory_program_with_optional_authorization_time_source<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    authorization_time_source: Option<
        Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>,
    >,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let mut runtime = in_memory_with_contributions::<Schema, Program::Contributions>(
        declaration,
        configuration,
        limits,
        initial_state,
        authorization_time_source,
    )?;
    let installed = install_application_program(program, runtime.installed_schema())
        .map_err(WorthQueryInMemoryApplicationDenial::Program)?;
    runtime
        .mutation_handlers
        .validate_managed_computations(installed.features())
        .map_err(WorthQueryInMemoryApplicationDenial::Contributions)?;
    conditional::validate_conditional_actions(
        installed.actions(),
        runtime.installed_conditionals.operation_types(),
    )?;
    let program_action_bindings = installed
        .actions()
        .iter()
        .filter_map(|action| action.mutation_binding_type())
        .collect::<std::collections::BTreeSet<_>>();
    runtime.program_required_operations.extend(
        installed
            .actions()
            .iter()
            .map(|action| action.operation_type()),
    );
    runtime.program_required_operations.extend(
        runtime
            .installed_schema
            .installed_mutation_binding_inventory()
            .filter(|binding| program_action_bindings.contains(&binding.binding_type()))
            .map(|binding| binding.operation_type()),
    );
    runtime
        .program_required_bindings
        .extend(program_action_bindings);
    let mut output_source_bindings = std::collections::BTreeSet::new();
    Program::Outputs::append_required_bindings(&mut output_source_bindings);
    output_source_bindings.extend(installed.actions().iter().filter_map(|action| {
        action
            .required_output_source()
            .then(|| action.mutation_binding_type())
            .flatten()
    }));
    worth_query_installation::facade::require_complete_program_binding_membership(
        &installed,
        runtime.installed_schema(),
        &output_source_bindings,
    )
    .map_err(WorthQueryInMemoryApplicationDenial::Program)?;
    runtime
        .program_required_bindings
        .extend(output_source_bindings.iter().copied());
    runtime.installed_program_action_operations = Some(
        installed
            .actions()
            .iter()
            .map(|action| action.operation_type())
            .collect(),
    );
    Ok(WorthQueryProgramApplicationRuntime {
        runtime,
        program: installed,
        connection_types: Program::Outputs::connection_types().into_boxed_slice(),
        root_graph_types: Program::Outputs::root_graph_types().into_boxed_slice(),
        output_source_bindings: output_source_bindings.into_iter().collect(),
    })
}
