//! Consumed owner phases release preparation frames before later effects.
use super::super::runtime::WorthQueryApplicationHome;
use super::{plan::*, refusal::*};
use worth_query_declaration::facade::{application_program::*, application_schema::*};
use worth_query_execution::facade::{
    application_contribution::*, application_home_integration::*, application_installation::*,
};
use worth_store_application::{
    ApplicationHomeCheckpointLimits, ApplicationHomeOpeningObservation,
    UnsharedReadmittedApplicationHome,
};
#[inline(never)]
pub(super) fn transition<A, AP, B, BP, Author>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    before: ReadmittedExistingHomeNativeMeaning<A, AP>,
    after: PreparedHomeApplication<B, BP>,
    home: UnsharedReadmittedApplicationHome,
    author: Author,
    resources: worth_relational::facade::indexes::DerivedIndexMaintenanceBudget,
    workflow_dispositions: Option<
        worth_query_execution::facade::primary_graph::WorthQueryWorkflowDispositions,
    >,
    checkpoint_limits: ApplicationHomeCheckpointLimits,
    observation: ApplicationHomeOpeningObservation,
) -> Result<
    WorthQueryApplicationHome<B, BP>,
    WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
>
where
    A: ApplicationSchemaComposition,
    AP: ApplicationProgramDefinition<A> + 'static,
    AP::Outputs: ApplicationProgramOutputsShape<A> + WorthQueryApplicationProgramRoots<A>,
    B: ApplicationSchemaComposition,
    BP: ApplicationProgramDefinition<B> + 'static,
    BP::Outputs: ApplicationProgramOutputsShape<B> + WorthQueryApplicationProgramRoots<B>,
    Author: FnOnce(
        &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
        &WorthQueryHomeProgramTransitionTarget<'_>,
    ) -> Result<
        WorthQueryHomeProgramTransitionPlan,
        worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    >,
{
    let prepared = super::preparation::prepare(phase, before, after, home)?;
    let planned = prepared.plan(phase, author, resources, workflow_dispositions.as_ref())?;
    let bound = planned.perform(phase)?;
    bound
        .attach(phase, checkpoint_limits, observation)
        .map_err(WorthQueryHomeProgramTransitionOpenRefused::successor)
}
