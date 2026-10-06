use worth_query_declaration::facade::{
    application_program::{
        ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
    },
    application_schema::{ApplicationSchemaComposition, ApplicationSchemaDeclaration},
};

use crate::domain_computation::primary_graph::{
    ApplicationHome, WorthQueryApplicationContributionTuple,
};

use super::{
    WorthQueryApplicationProgramRoots, WorthQueryApplicationProgramRoster,
    WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::application_installation::{
    WorthQueryApplicationLimits, WorthQueryApplicationOpenDenial,
};

/// Restores a program runtime from one Query-issued committed-world checkpoint.
///
/// The fresh installation still validates declarations, contributions and the
/// program. Initial-state authoring is intentionally absent: restored Query
/// authority is the only model source for this path.
///
/// Retained, with the rostered form below, until every caller reopens a home
/// through [`program`](super::program).
pub fn in_memory_program_from_checkpoint<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryApplicationLimits,
    checkpoint: crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryApplicationOpenDenial>
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
    limits: WorthQueryApplicationLimits,
    checkpoint: crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryApplicationOpenDenial>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    super::program(program, declaration, configuration, limits)
        .roster(roster)
        .open(ApplicationHome::holding(checkpoint))
        .map_err(|refusal| refusal.denial)
}
