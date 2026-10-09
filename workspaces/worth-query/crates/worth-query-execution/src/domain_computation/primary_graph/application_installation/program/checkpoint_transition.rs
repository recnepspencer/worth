use super::{
    construction::in_memory_program_with_optional_authorization_time_source,
    WorthQueryApplicationProgramRoots, WorthQueryApplicationProgramRoster,
    WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::{
    application_installation::{
        WorthQueryCheckpointMigrationWriter, WorthQueryCheckpointProgramPredecessor,
        WorthQueryCheckpointTransitionResources, WorthQueryInMemoryApplicationDenial,
        WorthQueryInMemoryApplicationLimits,
    },
    bootstrap::checkpoint_transition::CheckpointTransition,
    WorthQueryApplicationCheckpoint, WorthQueryApplicationContributionTuple,
    WorthQueryCheckpointCapturePolicy, WorthQueryPrimaryGraphInstallationDenial,
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
pub fn in_memory_rostered_program_from_checkpoint_with_transition<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
    checkpoint: WorthQueryApplicationCheckpoint,
    predecessor: WorthQueryCheckpointProgramPredecessor,
    resources: WorthQueryCheckpointTransitionResources,
    capture_policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    author: impl FnOnce(
        &mut WorthQueryCheckpointMigrationWriter<'_, Schema>,
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
    capture_policy.check_live().map_err(|error| {
        WorthQueryInMemoryApplicationDenial::CheckpointTransitionPolicyStopped(error.into())
    })?;
    let recovery = std::rc::Rc::new(std::cell::RefCell::new(None));
    let result = in_memory_program_with_optional_authorization_time_source(
        program,
        roster,
        declaration,
        configuration,
        limits,
        |_, _| Ok(()),
        None,
        Some(checkpoint),
        Some(CheckpointTransition {
            recovery: std::rc::Rc::clone(&recovery),
            predecessor,
            resources,
            capture_policy,
            author: Box::new(author),
        }),
    );
    result.map_err(|cause| match recovery.borrow_mut().take() {
        Some(checkpoint) => WorthQueryInMemoryApplicationDenial::CheckpointTransitionAcknowledged {
            checkpoint,
            cause: Box::new(cause),
        },
        None => cause,
    })
}
