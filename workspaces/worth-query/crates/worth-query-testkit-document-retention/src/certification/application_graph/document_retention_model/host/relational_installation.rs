//! One ordinary Relational registration owner, also usable without Signal declarations.
use super::*;
use worth_query_host::facade::declaration::application_schema::{
    ApplicationSchemaContribution, ApplicationSchemaContributionIdentity,
};

pub struct RelationalRetentionContribution;
impl ApplicationSchemaContribution<DocumentRetentionSchema> for RelationalRetentionContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity = <DocumentRetentionContribution as ApplicationSchemaContribution<DocumentRetentionSchema>>::IDENTITY;
    type Members = <DocumentRetentionContribution as ApplicationSchemaContribution<
        DocumentRetentionSchema,
    >>::Members;
}
impl WorthQueryApplicationContribution<DocumentRetentionSchema>
    for RelationalRetentionContribution
{
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, DocumentRetentionSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        install(setup)
    }
}

pub(super) fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, DocumentRetentionSchema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<super::super::actor_generation::ActorLifecycleBinding, _>(
        super::super::actor_generation::ActorLifecycleHandler,
    )?;
    setup.handler::<super::super::model_lifecycle::ModelLifecycleBinding, _>(
        super::super::model_lifecycle::ModelLifecycleHandler,
    )?;
    setup.handler::<super::super::record_history::reaffirm::Binding, _>(
        super::super::record_history::reaffirm::Handler,
    )?;
    setup.invariant(
        DocumentRetentionV1::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        resolve_first_rule,
    )?;
    setup.invariant(
        DocumentRetentionV2::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        resolve_second_rule,
    )?;
    setup
        .handler::<SetRetentionBinding, _>(SetRetentionHandler::default())
        .and_then(|()| {
            setup.handler::<super::super::correctable_retention::CorrectableRetentionBinding, _>(
                ReviewedSetRetentionHandler,
            )
        })
        .and_then(|()| setup.handler::<ReviewedSetRetentionBinding, _>(ReviewedSetRetentionHandler))
        .and_then(|()| {
            setup.handler::<OccurrenceSetRetentionBinding, _>(ReviewedSetRetentionHandler)
        })
        .and_then(|()| {
            setup.handler::<ExplicitReviewedSetRetentionBinding, _>(ReviewedSetRetentionHandler)
        })
        .and_then(|()| setup.handler::<ReviewRequirementBinding, _>(ReviewRequirementHandler))
        .and_then(|()| {
            setup.handler::<UnlinkReviewRequirementBinding, _>(UnlinkReviewRequirementHandler)
        })
        .and_then(|()| setup.handler::<RetentionAssessmentBinding, _>(RetentionAssessmentHandler))
        .and_then(|()| {
            setup.handler::<OrdinaryRetentionAssessmentBinding, _>(
                OrdinaryRetentionAssessmentHandler,
            )
        })
        .and_then(|()| setup.handler::<WorkflowGrantStatusBinding, _>(WorkflowGrantStatusHandler))
}
