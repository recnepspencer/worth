use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationRequiredOutputConnection,
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
mod specialized_action;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationContributionTuple, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};
pub use demand::{
    WorthQueryAdmittedProgramOutput, WorthQueryProgramOutputAdvance, WorthQueryProgramRootDemand,
    WorthQuerySettledProgramOutput,
};
pub use specialized_action::WorthQueryAdmittedProgramOperation;

type RootConnectionRef<Schema, Root> =
    <Root as ApplicationOutputGraphShape<Schema>>::RootConnection;
type RootConnection<Schema, Root> =
    <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding;
type PreparedProgramSource = (
    crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
    std::sync::Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>,
);
type ProgramSourceCommit = Result<
    (
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome,
        Option<PreparedProgramSource>,
    ),
    crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure,
>;

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

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs: ApplicationProgramOutputsShape<Schema>,
{
    pub fn compare_and_commit_required_output_source<Root, Source>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        program: crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            Source::Operation,
            Source::Input,
            <Source::ScopeBinding as worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> ProgramSourceCommit
    where
        Source: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
        Root: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Source = Source>,
        Source::Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_output_source::<Source>(
            program,
            idempotency,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Required(std::any::TypeId::of::<Root>()),
            None,
        )
    }

    pub fn compare_and_commit_discovered_output_source<Root, Source>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        program: crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            Source::Operation,
            Source::Input,
            <Source::ScopeBinding as worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
        discovery: <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Discovery,
    ) -> ProgramSourceCommit
    where
        Source: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
        Root: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Root>:
            WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Source>,
        Source::Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_output_source::<Source>(
            program,
            idempotency,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Discovered(std::any::TypeId::of::<Root>()),
            Some(std::sync::Arc::new(discovery)),
        )
    }

    pub fn complete_program_output_source(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) {
        self.runtime.complete_prepared_output_source(receipt);
    }

    pub fn recover_discovered_program_source<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<
        (
            <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<
                Schema,
            >>::Discovery,
            std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >,
            crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
        ),
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial,
    >
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "discovered output root is not installed for this program",
            ));
        }
        self.runtime.recover_discovered_output_source::<
            <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Discovery,
        >(receipt, std::any::TypeId::of::<Root>())
    }

    fn compare_and_commit_output_source<Source>(
        &self,
        program: crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            Source::Operation,
            Source::Input,
            <Source::ScopeBinding as worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
        root_kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind,
        discovery: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
    ) -> ProgramSourceCommit
    where
        Source: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
        Source::Input: Clone + Send + Sync + 'static,
    {
        let source_preparation = self
            .runtime
            .output_demands
            .begin_source_preparation(program.product_branch().occurrence());
        match self
            .runtime
            .compare_and_commit_application_for_required_output_source(program, idempotency)
        {
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(
                mut receipt,
            ) => {
                let descriptive = receipt.clone();
                let Some(change) = receipt.take_performed_relational_product_change() else {
                    return Err(crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure {
                        receipt: descriptive,
                        denial: crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                            "committed required-output source has no performed-change carrier",
                        ),
                    });
                };
                let prepared = match self
                    .runtime
                    .retain_required_output_source(receipt, change, &source_preparation, root_kind, discovery)
                {
                    Ok(prepared) => prepared,
                    Err(denial) => {
                        return Err(crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure {
                            receipt: descriptive,
                            denial,
                        })
                    }
                };
                Ok((
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(
                        descriptive,
                    ),
                    Some(prepared),
                ))
            }
            outcome => Ok((outcome, None)),
        }
    }
}
