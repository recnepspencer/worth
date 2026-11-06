//! Native contribution construction within already admitted bootstrap custody.
use super::super::application_contribution::WorthQueryApplicationContributionTuple;
use super::super::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};
use super::{
    WorthQueryInMemoryApplicationDenial, WorthQueryInMemoryApplicationLimits,
    WorthQueryProgramAdmissionStep,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;
mod native_preparation;
mod publication;

pub(super) fn in_memory_with_contributions_in_advancement<Schema, Contributions>(
    phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryInMemoryApplicationLimits,
    initial_state: impl FnOnce(
        &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
        &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    authorization_time_source: Option<
        Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>,
    >,
    program_admission: Option<WorthQueryProgramAdmissionStep<'_, Schema>>,
    checkpoint: Option<super::super::WorthQueryApplicationCheckpoint>,
    checkpoint_transition: Option<
        super::super::bootstrap::checkpoint_transition::CheckpointTransition<'_, '_, Schema>,
    >,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryInMemoryApplicationDenial>
where
    Schema: ApplicationSchemaComposition,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let prepared = native_preparation::prepare_native::<Schema, Contributions>(
        phase,
        declaration,
        configuration,
        build_identity,
        limits,
        initial_state,
        authorization_time_source,
        program_admission,
        checkpoint,
        checkpoint_transition,
    )?;
    publication::publish(phase, prepared)
}
