use worth_query_declaration::facade::{
    application_program::{
        ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
    },
    application_schema::{ApplicationSchemaComposition, ApplicationSchemaDeclaration},
};

use crate::domain_computation::primary_graph::WorthQueryApplicationContributionTuple;

use super::{
    construction::in_memory_program_with_optional_authorization_time_source,
    WorthQueryApplicationProgramRoots, WorthQueryApplicationProgramRoster,
    WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::application_installation::{
    WorthQueryInMemoryApplicationDenial, WorthQueryInMemoryApplicationLimits,
};

/// Restores a program runtime from one Query-issued committed-world checkpoint.
///
/// The fresh installation still validates declarations, contributions and the
/// program. Initial-state authoring is intentionally absent: restored Query
/// authority is the only model source for this path.
pub fn in_memory_program_from_checkpoint<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    checkpoint: crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    in_memory_rostered_program_from_checkpoint(
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        configuration,
        build_identity,
        limits,
        checkpoint,
    )
}

/// Restores a program runtime and the programs it supports from one
/// Query-issued committed-world checkpoint. A host restores the same roster it
/// installed, so a branch the checkpoint recorded under a supported program
/// still runs it.
pub fn in_memory_rostered_program_from_checkpoint<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    checkpoint: crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint,
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
        roster,
        declaration,
        configuration,
        build_identity,
        limits,
        |_phase, _graph, _installed| Ok(()),
        None,
        Some(checkpoint),
        None,
    )
}

/// Restore while retaining privileged, read-only invariant projection authority.
///
/// Only the composition root performing restoration receives this authority.
/// The checkpoint remains the sole model source: this grants no bootstrap writes.
/// The returned authority belongs to the restored installation and its current
/// program activation, exactly like authority retained during initial install.
pub fn in_memory_program_from_checkpoint_with_projection<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    checkpoint: crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint,
) -> Result<
    (
        WorthQueryProgramApplicationRuntime<Schema, Program>,
        crate::domain_computation::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
            Schema,
        >,
    ),
    WorthQueryInMemoryApplicationDenial,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let runtime = in_memory_program_from_checkpoint(
        program,
        declaration,
        configuration,
        build_identity,
        limits,
        checkpoint,
    )?;
    let projection = runtime
        .runtime()
        .mutation_projection
        .retain_for_installation();
    Ok((runtime, projection))
}
