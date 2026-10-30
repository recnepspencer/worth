//! Fresh program installation within the host's already admitted bootstrap.
use super::{
    WorthQueryApplicationProgramRoots, WorthQueryApplicationProgramRoster,
    WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::{
    application_installation::{
        WorthQueryInMemoryApplicationDenial, WorthQueryInMemoryApplicationLimits,
    },
    WorthQueryApplicationContributionTuple, WorthQueryBootstrapAdvancementPhase,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_declaration::facade::{
    application_program::{
        ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
    },
    application_schema::{ApplicationSchemaComposition, ApplicationSchemaDeclaration},
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

/// Install a fresh program and roster under the supplied bootstrap request.
/// Setup retains that request's work, memory, cancellation and deadline controls.
/// The installed World keeps the runtime request policy declared in `limits`.
/// Ordinary `in_memory_program` and `in_memory_rostered_program` callers open
/// their default bootstrap from that policy instead.
pub fn in_memory_rostered_program_in_advancement<Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &WorthQueryBootstrapAdvancementPhase<'_>,
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
    phase.execution_request().consult().map_err(|cause| {
        WorthQueryInMemoryApplicationDenial::Graph(super::super::super::bootstrap::request_denial(
            super::super::super::application_contribution::advancement_denial(cause),
        ))
    })?;
    super::construction::in_memory_program_in_advancement(
        phase,
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
