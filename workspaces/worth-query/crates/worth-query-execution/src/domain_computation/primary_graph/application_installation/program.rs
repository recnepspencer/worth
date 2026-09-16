use crate::domain_computation::primary_graph::WorthQueryApplicationRequiredOutputConnection;
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraph, ApplicationOutputGraphShape,
    ApplicationProgramDefinition, ApplicationProgramOutputRootsShape, ApplicationProgramOutputs,
    ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::{
    install_application_program, WorthQueryInstalledApplicationProgram,
};

use super::{
    in_memory_with_program, WorthQueryInMemoryApplicationDenial,
    WorthQueryInMemoryApplicationLimits,
};

mod demand;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationContributionTuple, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};
pub use demand::{
    WorthQueryAdmittedProgramOutput, WorthQueryProgramOutputAdvance, WorthQueryProgramRootDemand,
    WorthQuerySettledProgramOutput,
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
}

impl<Schema, Program> std::ops::Deref for WorthQueryProgramApplicationRuntime<Schema, Program> {
    type Target = WorthQueryPrimaryGraphApplicationRuntime<Schema>;

    fn deref(&self) -> &Self::Target {
        &self.runtime
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
    Schema: ApplicationSchemaComposition<Contributions = Program::Contributions>,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let mut runtime =
        in_memory_with_program(declaration, configuration, limits, initial_state, true)?;
    let installed = install_application_program(program, runtime.installed_schema())
        .map_err(WorthQueryInMemoryApplicationDenial::Program)?;
    Program::Outputs::append_required_bindings(&mut runtime.program_required_bindings);
    Ok(WorthQueryProgramApplicationRuntime {
        runtime,
        program: installed,
        connection_types: Program::Outputs::connection_types().into_boxed_slice(),
        root_graph_types: Program::Outputs::root_graph_types().into_boxed_slice(),
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
    ) -> Result<
        (
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome,
            Option<(
                crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
                std::sync::Arc<
                    crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
                >,
            )>,
        ),
        crate::domain_computation::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure,
    >
    where
        Source: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
        Root: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Source = Source>,
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
                    .retain_required_output_source(receipt, change, &source_preparation)
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
