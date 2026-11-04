//! Memory-only composition has no physical reopening frames.
use super::*;
#[inline(never)]
pub(super) fn open_memory<'authoring, Schema, Program>(
    request: WorthQueryApplicationHomeRequest<'authoring, Schema, Program>,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
) -> Result<
    WorthQueryApplicationHome<Schema, Program>,
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    use WorthQueryApplicationHomeOpenRefused as Refused;
    in_memory_rostered_program_in_advancement(
        phase,
        request.program,
        request.roster,
        request.declaration,
        request.configuration,
        request.build,
        request.limits,
        request.initial_state,
    )
    .map(|query| WorthQueryApplicationHome {
        query,
        directory: None,
        checkpoint_limits: request.checkpoint_limits,
    })
    .map_err(Refused::Preparation)
}
