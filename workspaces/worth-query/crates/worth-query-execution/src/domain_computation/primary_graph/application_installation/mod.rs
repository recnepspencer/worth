//! Complete construction of one contribution-composed in-memory application.

mod checkpoint_lineage;
mod construction;
mod contribution_preparation;
pub(crate) mod home_preparation;
mod prepared_meaning;
use construction::construct_with_contributions_in_advancement;
mod denial;
mod limits;
mod profile;
pub(crate) mod program;
pub(in crate::domain_computation::primary_graph) use program::HistoryAdmittedChildSource;
pub(in crate::domain_computation::primary_graph) use program::OriginalProgramRootCompletion;
pub(in crate::domain_computation::primary_graph) use program::WorthQuerySupportedProgramRecord;
pub(in crate::domain_computation::primary_graph) mod program_admission;
pub use super::bootstrap::checkpoint_transition::{
    WorthQueryCheckpointMigrationWriter, WorthQueryCheckpointProgramPredecessor,
    WorthQueryCheckpointTransitionRecovery, WorthQueryCheckpointTransitionResources,
};
pub use denial::WorthQueryApplicationPreparationDenial;
pub use limits::WorthQueryApplicationLimits;
pub use profile::WorthQueryApplicationProfile;
pub(crate) use program::workflow_approval_authentication_intent;
pub use program::{
    in_memory_program, in_memory_program_with_authorization_time_source,
    in_memory_rostered_program, in_memory_rostered_program_in_advancement,
    in_memory_rostered_program_with_authorization_time_source,
    in_memory_rostered_program_with_authorization_time_source_in_advancement,
    WorthQueryAdmittedProgramOperation, WorthQueryAdmittedProgramOutput,
    WorthQueryApplicationPreviewReadmissionDenial, WorthQueryApplicationPreviewRequest,
    WorthQueryApplicationPreviewSession, WorthQueryApplicationProgramRoots,
    WorthQueryApplicationProgramRoster, WorthQueryHomeRetirementAdvance,
    WorthQueryInterimCheckpointRestore, WorthQueryPreparedHomeQuery,
    WorthQueryProgramApplicationRuntime, WorthQueryProgramOutputAdvance, WorthQueryProgramOwner,
    WorthQueryProgramRootDemand, WorthQueryProgramSupportRetirementFailure,
    WorthQueryProgramSupportRetirementReceipt, WorthQueryReadmittedApplicationPreview,
    WorthQueryRecoveredProgramOutputSource, WorthQueryRetiredHomeQuery,
    WorthQueryRetiringHomeQuery, WorthQuerySelectedProgramOwner,
    WorthQuerySelectedProgramOwnerDenial, WorthQuerySettledProgramOutput,
    WorthQuerySigningOwnerInstallationDenial, WorthQuerySupportedProgramHandle,
    WorthQueryUnpublishedProgramOutputSource, WorthQueryWorkflowApplicationRuntime,
    WorthQueryWorkflowRuntimeBindingDenial, WorthQueryWorkflowRuntimeBindingRefusal,
    WorthQueryWorkflowRuntimeConfiguration, WorthQueryWorkflowVocabulary,
};
use program_admission::WorthQueryProgramAdmissionStep;

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
///     application_installation::{in_memory, ApplicationBuildIdentity, WorthQueryApplicationLimits},
/// };
/// use worth_query_declaration::facade::application_schema::{
///     ApplicationSchemaComposition, ApplicationSchemaDeclaration,
/// };
/// fn install<Schema>(
///     declaration: ApplicationSchemaDeclaration<Schema>,
///     configuration: <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
///     build: ApplicationBuildIdentity,
///     limits: WorthQueryApplicationLimits,
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
///     application_installation::{in_memory, ApplicationBuildIdentity, WorthQueryApplicationLimits},
/// };
/// use worth_query_declaration::facade::application_schema::{
///     ApplicationSchemaComposition, ApplicationSchemaDeclaration,
/// };
/// fn install<Schema>(
///     declaration: ApplicationSchemaDeclaration<Schema>,
///     configuration: <Schema::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
///     build: ApplicationBuildIdentity,
///     limits: WorthQueryApplicationLimits,
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
    limits: WorthQueryApplicationLimits,
    initial_state: impl FnOnce(
        &super::WorthQueryBootstrapAdvancementPhase<'_>,
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryApplicationPreparationDenial>
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
    limits: WorthQueryApplicationLimits,
    initial_state: impl FnOnce(
        &super::WorthQueryBootstrapAdvancementPhase<'_>,
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
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryApplicationPreparationDenial>
where
    Schema: ApplicationSchemaComposition,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let policy = limits.world.execution_policy();
    super::application_contribution::with_bootstrap_advancement(policy, |phase| {
        construct_with_contributions_in_advancement::<Schema, Contributions>(
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
        WorthQueryApplicationPreparationDenial::Graph(super::bootstrap::request_denial(cause))
    })?
}

#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
mod source_scope_ownership;
