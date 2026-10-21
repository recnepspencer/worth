use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationProgram, WorthQueryInstalledApplicationSchema,
};

use super::super::{WorthQueryInMemoryApplicationDenial, WorthQueryInMemoryApplicationLimits};
use super::supported_program::WorthQuerySupportedProgramRecord;
use super::{
    WorthQueryApplicationProgramRoots, WorthQueryApplicationProgramRoster,
    WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationContributionTuple, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};

/// Validates and installs program meaning before exposing its application runtime.
pub fn in_memory_program<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
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
        build_identity,
        limits,
        initial_state,
        None,
        None,
        None,
    )
}

/// Validates and installs program meaning with one host-owned trusted-time
/// source fixed before publication.
pub fn in_memory_program_with_authorization_time_source<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
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
        build_identity,
        limits,
        initial_state,
        Some(Box::new(source)),
        None,
        None,
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
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
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
        build_identity,
        limits,
        initial_state,
        None,
        None,
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
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
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
        build_identity,
        limits,
        initial_state,
        Some(Box::new(source)),
        None,
        None,
    )
}

pub(super) fn in_memory_program_with_optional_authorization_time_source<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    authorization_time_source: Option<
        Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>,
    >,
    checkpoint: Option<crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint>,
    checkpoint_transition: Option<crate::domain_computation::primary_graph::bootstrap::checkpoint_transition::CheckpointTransition<'_, '_, Schema>>,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    crate::domain_computation::primary_graph::with_bootstrap_advancement(
        limits.world.execution_policy(),
        |phase| {
            in_memory_program_in_advancement(
                &phase,
                program,
                roster,
                declaration,
                configuration,
                build_identity,
                limits,
                initial_state,
                authorization_time_source,
                checkpoint,
                checkpoint_transition,
            )
        },
    )
    .map_err(|cause| {
        WorthQueryInMemoryApplicationDenial::Graph(super::super::super::bootstrap::request_denial(
            cause,
        ))
    })?
}

pub(super) fn in_memory_program_in_advancement<Schema, Program>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    authorization_time_source: Option<
        Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>,
    >,
    checkpoint: Option<crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint>,
    checkpoint_transition: Option<crate::domain_computation::primary_graph::bootstrap::checkpoint_transition::CheckpointTransition<'_, '_, Schema>>,
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
    let runtime = super::super::in_memory_with_contributions_in_advancement::<
        Schema,
        Program::Contributions,
    >(
        phase,
        declaration,
        configuration,
        build_identity,
        limits,
        initial_state,
        authorization_time_source,
        Some(Box::new(|installed_schema| {
            let (support, installed, supported) =
                super::assembly::admit_program_roster(program, roster, installed_schema)?;
            admitted = Some((installed, supported));
            Ok(support)
        })),
        checkpoint,
        checkpoint_transition,
    )?;
    super::assembly::assemble(runtime, admitted)
}
