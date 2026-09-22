use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::{
    install_rostered_application_program, WorthQueryInstalledApplicationProgram,
    WorthQueryInstalledApplicationSchema, WorthQueryProgramSupportAdmission,
};

use super::super::program_admission::WorthQueryAdmittedProgramSupport;
use super::super::{
    in_memory_with_contributions, WorthQueryInMemoryApplicationDenial,
    WorthQueryInMemoryApplicationLimits,
};
use super::supported_program::WorthQuerySupportedProgramRecord;
use super::{
    WorthQueryApplicationProgramRoots, WorthQueryApplicationProgramRoster,
    WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationContributionTuple, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};

/// Validates and installs program meaning before exposing its application runtime.
pub fn in_memory_program<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
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
        WorthQueryApplicationProgramRoster::new(),
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
        &WorthQueryInstalledApplicationSchema<Schema>,
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
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        configuration,
        limits,
        initial_state,
        Some(Box::new(source)),
    )
}

/// Validates and installs one host's complete program roster before exposing
/// the application runtime its initial program governs.
///
/// Every rostered program is admitted against the same installed schema, so
/// closing the roster proves that no installed rule is left without a
/// declaring owner even when no single program declares them all. The initial
/// program is the one this host activates; the rest become reachable through
/// [`WorthQueryProgramApplicationRuntime::supported_program`].
pub fn in_memory_rostered_program<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    in_memory_program_with_optional_authorization_time_source(
        program,
        roster,
        declaration,
        configuration,
        limits,
        initial_state,
        None,
    )
}

/// Installs a complete program roster with one host-owned trusted-time source
/// fixed before publication.
pub fn in_memory_rostered_program_with_authorization_time_source<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    source: impl crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    in_memory_program_with_optional_authorization_time_source(
        program,
        roster,
        declaration,
        configuration,
        limits,
        initial_state,
        Some(Box::new(source)),
    )
}

fn in_memory_program_with_optional_authorization_time_source<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    authorization_time_source: Option<
        Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>,
    >,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let mut admitted: Option<(
        WorthQueryInstalledApplicationProgram<Schema, Program>,
        Vec<WorthQuerySupportedProgramRecord>,
    )> = None;
    let mut runtime = in_memory_with_contributions::<Schema, Program::Contributions>(
        declaration,
        configuration,
        limits,
        initial_state,
        authorization_time_source,
        Some(Box::new(|installed_schema| {
            let (support, installed, supported) =
                admit_program_roster(program, roster, installed_schema)?;
            admitted = Some((installed, supported));
            Ok(support)
        })),
    )?;
    let (installed, supported) =
        admitted.ok_or(WorthQueryInMemoryApplicationDenial::ProgramAdmissionIncomplete)?;
    runtime
        .mutation_handlers
        .validate_managed_computations(installed.features())
        .map_err(WorthQueryInMemoryApplicationDenial::Contributions)?;
    super::conditional::validate_conditional_actions(
        installed.actions(),
        runtime.installed_conditionals.operation_types(),
    )?;
    let action_bindings = installed
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
            .filter(|binding| action_bindings.contains(&binding.binding_type()))
            .map(|binding| binding.operation_type()),
    );
    runtime
        .program_required_bindings
        .extend(action_bindings.iter().copied());
    let mut output_source_bindings = std::collections::BTreeSet::new();
    Program::Outputs::append_required_bindings(&mut output_source_bindings);
    output_source_bindings.extend(installed.actions().iter().filter_map(|action| {
        action
            .required_output_source()
            .then(|| action.mutation_binding_type())
            .flatten()
    }));
    require_rostered_binding_membership(&runtime, &installed, &output_source_bindings)?;
    runtime
        .program_required_bindings
        .extend(output_source_bindings.iter().copied());
    Ok(WorthQueryProgramApplicationRuntime {
        runtime,
        program: installed,
        connection_types: Program::Outputs::connection_types().into_boxed_slice(),
        root_graph_types: Program::Outputs::root_graph_types().into_boxed_slice(),
        output_source_bindings: output_source_bindings.into_iter().collect(),
        action_bindings: action_bindings.into_iter().collect(),
        supported: supported.into_boxed_slice(),
    })
}

/// Admits the complete roster, then installs every rostered program against it.
///
/// Admission closes before any program is installed, so nothing is ever
/// installed against a roster that is still growing.
fn admit_program_roster<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<
    (
        WorthQueryAdmittedProgramSupport<Schema>,
        WorthQueryInstalledApplicationProgram<Schema, Program>,
        Vec<WorthQuerySupportedProgramRecord>,
    ),
    WorthQueryInMemoryApplicationDenial,
>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    let admission = WorthQueryProgramSupportAdmission::for_installed_schema(installed_schema)
        .support(&program)
        .map_err(program_support_denied)?;
    let closed = roster
        .admit_all(admission)
        .map_err(program_support_denied)?
        .close()
        .map_err(program_support_denied)?;
    let supported = roster
        .install_all(installed_schema, &closed)
        .map_err(program_support_denied)?;
    let initial_revision = program.revision().clone();
    let installed = install_rostered_application_program(program, installed_schema, &closed)
        .map_err(program_support_denied)?;
    Ok((
        WorthQueryAdmittedProgramSupport {
            roster: std::sync::Arc::new(closed),
            initial_revision,
        },
        installed,
        supported,
    ))
}

/// Requires that every installed binding demanding a program is answered by the
/// roster as a whole: an action of some rostered program, or an output source.
fn require_rostered_binding_membership<Schema, Program>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    installed: &WorthQueryInstalledApplicationProgram<Schema, Program>,
    output_source_bindings: &std::collections::BTreeSet<std::any::TypeId>,
) -> Result<(), WorthQueryInMemoryApplicationDenial>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    let mut answered = output_source_bindings.clone();
    if let Some(support) = runtime.installed_program_support() {
        answered.extend(support.rostered_mutation_bindings());
    }
    worth_query_installation::facade::require_complete_program_binding_membership(
        installed,
        runtime.installed_schema(),
        &answered,
    )
    .map_err(WorthQueryInMemoryApplicationDenial::Program)
}

fn program_support_denied(
    denial: worth_query_installation::facade::WorthQueryProgramSupportDenial,
) -> WorthQueryInMemoryApplicationDenial {
    WorthQueryInMemoryApplicationDenial::Program(denial.into())
}
