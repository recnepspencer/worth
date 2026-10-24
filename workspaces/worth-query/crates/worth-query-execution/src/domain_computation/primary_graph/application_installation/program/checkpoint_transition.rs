use super::{
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
/// rostered activation. Ordinary transitions support new records and links
/// between them. A derived-codec cutover instead requires complete performed
/// native create provenance, preserves authored records, and retires whole
/// outputs before their installed target producers reconstruct them. It cannot
/// append authored rows or retain partly classified output correspondence.
/// Workflow records and relation-scoped rules remain unsupported.
pub fn in_memory_rostered_program_from_checkpoint_with_transition<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
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
    let world_policy = limits.world.execution_policy();
    let transition =
        |phase: crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<
            '_,
        >| {
            transition_in_advancement(
                &phase,
                program,
                roster,
                declaration,
                configuration,
                build_identity,
                limits,
                checkpoint,
                predecessor,
                resources,
                capture_policy,
                author,
            )
        };
    // An `Execution(lease)` caller funds the whole transition: the bootstrap
    // runs on that lease, so capture charges inside its own worker lineage.
    match capture_policy {
        WorthQueryCheckpointCapturePolicy::SystemAllocation => {
            crate::domain_computation::primary_graph::with_bootstrap_advancement(
                world_policy,
                transition,
            )
        }
        WorthQueryCheckpointCapturePolicy::Execution(lease) => {
            crate::domain_computation::primary_graph::with_bootstrap_advancement_from_request(
                worth_execution::ExecutionRequest::leased(lease),
                transition,
            )
        }
    }
    .map_err(|cause| {
        WorthQueryInMemoryApplicationDenial::Graph(super::super::super::bootstrap::request_denial(
            cause,
        ))
    })?
}

/// Perform the same native transition inside the caller's original bootstrap request.
/// The request funds the transition; `capture_policy` caps its capture and must
/// name the request's funding lease or a lease descending from it. Serial
/// backing, or any other capture policy, refuses before work or effects.
pub fn in_memory_rostered_program_from_checkpoint_with_transition_in_advancement<Schema, Program>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
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
    let capture_policy = phase
        .admit_capture_policy(capture_policy)
        .map_err(WorthQueryInMemoryApplicationDenial::Graph)?;
    transition_in_advancement(
        phase,
        program,
        roster,
        declaration,
        configuration,
        build_identity,
        limits,
        checkpoint,
        predecessor,
        resources,
        capture_policy,
        author,
    )
}

fn transition_in_advancement<Schema, Program>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
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
    phase.execution_request().consult().map_err(|stop| {
        WorthQueryInMemoryApplicationDenial::Graph(super::super::super::bootstrap::request_denial(
            super::super::super::application_contribution::advancement_denial(stop),
        ))
    })?;
    capture_policy.check_live().map_err(|error| {
        WorthQueryInMemoryApplicationDenial::CheckpointTransitionPolicyStopped(error.into())
    })?;
    let recovery = std::rc::Rc::new(std::cell::RefCell::new(None));
    let result = super::construction::in_memory_program_in_advancement(
        phase,
        program,
        roster,
        declaration,
        configuration,
        build_identity,
        limits,
        |_phase, _, _| Ok(()),
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
    // A post-performance owner is never replaced by a generic stop: it carries
    // exact native custody. Otherwise the original request precedes ordinary errors.
    let result = match result {
        Err(
            cause @ (WorthQueryInMemoryApplicationDenial::CheckpointTransitionDeferred(_)
            | WorthQueryInMemoryApplicationDenial::CheckpointTransitionCaptureStopped(_)
            | WorthQueryInMemoryApplicationDenial::CheckpointTransitionFinalizationStopped(
                _,
            )
            | WorthQueryInMemoryApplicationDenial::CheckpointTransitionAcknowledged {
                ..
            }
            | WorthQueryInMemoryApplicationDenial::CheckpointTransitionSettlementFailed(_)),
        ) => Err(cause),
        ordinary => match phase.execution_request().consult() {
            Ok(()) => ordinary,
            Err(stop) => Err(WorthQueryInMemoryApplicationDenial::Graph(
                super::super::super::bootstrap::request_denial(
                    super::super::super::application_contribution::advancement_denial(stop),
                ),
            )),
        },
    };
    result.map_err(|cause| match recovery.borrow_mut().take() {
        Some(checkpoint) => WorthQueryInMemoryApplicationDenial::CheckpointTransitionAcknowledged {
            checkpoint,
            cause: Box::new(cause),
        },
        None => cause,
    })
}
