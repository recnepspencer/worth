use super::{
    WorthQueryApplicationProgramRoots, WorthQueryApplicationProgramRoster,
    WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::{
    application_installation::{
        WorthQueryApplicationLimits, WorthQueryApplicationOpenRefusal, WorthQueryOpenAdoption,
        WorthQueryOpenAdoptionPredecessor, WorthQueryOpenAdoptionResources,
        WorthQueryOpenAdoptionWriter,
    },
    ApplicationHome, WorthQueryApplicationCheckpoint, WorthQueryApplicationContributionTuple,
    WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_declaration::facade::{
    application_program::{
        ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
    },
    application_schema::{ApplicationSchemaComposition, ApplicationSchemaDeclaration},
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

/// Migrates one exact recovered predecessor to the admitted initial program,
/// committing typed new records and target revalidation before World installation.
///
/// The predecessor is descriptive only. Ordinary restore still requires a
/// rostered activation. This first transition surface supports new records and
/// links between them; it refuses retained outputs, workflow records and
/// relation-scoped rules until their migration owners supply complete support.
///
/// Retained until every caller declares the adoption on [`program`](super::program).
/// A refusal retains the unchanged home, successor home, or repair capsule.
pub fn in_memory_rostered_program_from_checkpoint_with_transition<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryApplicationLimits,
    checkpoint: WorthQueryApplicationCheckpoint,
    predecessor: WorthQueryOpenAdoptionPredecessor,
    resources: WorthQueryOpenAdoptionResources,
    author: impl FnOnce(
        &mut WorthQueryOpenAdoptionWriter<'_, Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryApplicationOpenRefusal>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    super::program(program, declaration, configuration, limits)
        .roster(roster)
        .adopt_on_open(WorthQueryOpenAdoption::new(predecessor, resources, author))
        .open(ApplicationHome::holding(checkpoint))
}
