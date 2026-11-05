//! Complete construction of one contribution-composed in-memory application.

mod checkpoint_lineage;
mod construction;
use construction::in_memory_with_contributions_in_advancement;
mod denial;
mod limits;
mod profile;
pub(crate) mod program;
pub(in crate::domain_computation::primary_graph) use program::OriginalProgramRootCompletion;
pub(in crate::domain_computation::primary_graph) use program::WorthQuerySupportedProgramRecord;
pub(in crate::domain_computation::primary_graph) mod program_admission;
pub use super::bootstrap::checkpoint_transition::{
    WorthQueryCheckpointMigrationWriter, WorthQueryCheckpointProgramPredecessor,
    WorthQueryCheckpointTransitionRecovery, WorthQueryCheckpointTransitionResources,
};
pub use denial::WorthQueryInMemoryApplicationDenial;
pub use limits::WorthQueryInMemoryApplicationLimits;
pub use profile::WorthQueryInMemoryApplicationProfile;
pub(crate) use program::workflow_approval_authentication_intent;
use program_admission::WorthQueryProgramAdmissionStep;
pub use {
    program::in_memory_program, program::in_memory_program_from_checkpoint,
    program::in_memory_program_from_checkpoint_with_projection,
    program::in_memory_program_with_authorization_time_source, program::in_memory_rostered_program,
    program::in_memory_rostered_program_from_checkpoint,
    program::in_memory_rostered_program_from_checkpoint_with_transition,
    program::in_memory_rostered_program_from_checkpoint_with_transition_in_advancement,
    program::in_memory_rostered_program_in_advancement,
    program::in_memory_rostered_program_with_authorization_time_source,
    program::WorthQueryAdmittedProgramOperation, program::WorthQueryAdmittedProgramOutput,
    program::WorthQueryApplicationPreviewReadmissionDenial,
    program::WorthQueryApplicationPreviewRequest, program::WorthQueryApplicationPreviewSession,
    program::WorthQueryApplicationProgramRoots, program::WorthQueryApplicationProgramRoster,
    program::WorthQueryProgramApplicationRuntime, program::WorthQueryProgramOutputAdvance,
    program::WorthQueryProgramOwner, program::WorthQueryProgramRootDemand,
    program::WorthQueryProgramSupportRetirementFailure,
    program::WorthQueryProgramSupportRetirementReceipt,
    program::WorthQueryReadmittedApplicationPreview,
    program::WorthQueryRecoveredProgramOutputSource, program::WorthQuerySelectedProgramOwner,
    program::WorthQuerySettledProgramOutput, program::WorthQuerySigningOwnerInstallationDenial,
    program::WorthQuerySupportedProgramHandle, program::WorthQueryUnpublishedProgramOutputSource,
    program::WorthQueryWorkflowApplicationRuntime, program::WorthQueryWorkflowRuntimeBindingDenial,
    program::WorthQueryWorkflowRuntimeBindingRefusal, program::WorthQueryWorkflowVocabulary,
};

use super::application_contribution::WorthQueryApplicationContributionTuple;
use super::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

/// Installs contribution-owned schema meaning without an application program.
///
/// The complete application's embedded build identity is mandatory, including
/// for installations whose producers do not opt into input reuse.
///
/// A pipeline-issued identity follows contribution configuration.
///
/// ```
/// use worth_query_execution::facade::{
///     application_contribution::WorthQueryApplicationContributionTuple,
///     application_installation::{in_memory, ApplicationBuildIdentity, WorthQueryInMemoryApplicationLimits},
/// };
/// use worth_query_declaration::facade::application_schema::{
///     ApplicationSchemaComposition, ApplicationSchemaDeclaration,
/// };
/// fn install<Schema>(
///     declaration: ApplicationSchemaDeclaration<Schema>,
///     configuration: <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
///     build: ApplicationBuildIdentity,
///     limits: WorthQueryInMemoryApplicationLimits,
/// ) where
///     Schema: ApplicationSchemaComposition,
///     Schema::Contributions: WorthQueryApplicationContributionTuple<Schema>,
/// {
///     let _ = in_memory(declaration, configuration, build, limits, |_, _, _| Ok(()));
/// }
/// ```
///
/// The otherwise identical call cannot omit that installation input.
///
/// ```compile_fail,E0061
/// use worth_query_execution::facade::{
///     application_contribution::WorthQueryApplicationContributionTuple,
///     application_installation::{in_memory, ApplicationBuildIdentity, WorthQueryInMemoryApplicationLimits},
/// };
/// use worth_query_declaration::facade::application_schema::{
///     ApplicationSchemaComposition, ApplicationSchemaDeclaration,
/// };
/// fn install<Schema>(
///     declaration: ApplicationSchemaDeclaration<Schema>,
///     configuration: <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
///     build: ApplicationBuildIdentity,
///     limits: WorthQueryInMemoryApplicationLimits,
/// ) where
///     Schema: ApplicationSchemaComposition,
///     Schema::Contributions: WorthQueryApplicationContributionTuple<Schema>,
/// {
///     let _ = in_memory(declaration, configuration, limits, |_, _, _| Ok(()));
/// }
/// ```
///
pub fn in_memory<Schema>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<'_>,
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Schema::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    in_memory_with_contributions::<Schema, Schema::Contributions>(
        declaration,
        configuration,
        build_identity,
        limits,
        initial_state,
        None,
        None,
        None,
        None,
    )
}

pub(super) fn in_memory_with_contributions<Schema, Contributions>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<'_>,
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    authorization_time_source: Option<
        Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>,
    >,
    program_admission: Option<WorthQueryProgramAdmissionStep<'_, Schema>>,
    checkpoint: Option<super::WorthQueryApplicationCheckpoint>,
    checkpoint_transition: Option<
        super::bootstrap::checkpoint_transition::CheckpointTransition<'_, '_, Schema>,
    >,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let policy = limits.world.execution_policy();
    crate::domain_computation::request_execution::with_bootstrap_advancement(policy, |phase| {
        in_memory_with_contributions_in_advancement::<Schema, Contributions>(
            &phase,
            declaration,
            configuration,
            build_identity,
            limits,
            initial_state,
            authorization_time_source,
            program_admission,
            checkpoint,
            checkpoint_transition,
        )
    })
    .map_err(|cause| {
        WorthQueryInMemoryApplicationDenial::Graph(super::bootstrap::request_denial(cause))
    })?
}

mod source_scope_ownership;
pub use program::SelectedOwnerOps;
pub(crate) mod host;
