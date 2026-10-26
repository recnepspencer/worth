//! Separate action meaning leaves canonical P0 and its cold artifact unchanged.
use super::super::{
    assessment_output::PublishRetentionAssessment,
    programs::{DocumentRetentionFeature, RetentionCompletion, RetentionProgramP0},
    retention_entry::*,
    schema::*,
    workflow::*,
};
use worth_query_decl::facade::application_program::*;
pub struct WorkflowOwnerProgram;
pub struct WorkflowEffectDelivery;
impl ApplicationActionDelivery<DocumentRetentionSchema, OccurrenceSetRetentionBinding>
    for WorkflowEffectDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "workflow-owner.occurrence-completion.v1";
}
pub struct ReviewedEffectDelivery;
impl ApplicationActionDelivery<DocumentRetentionSchema, ReviewedSetRetentionBinding>
    for ReviewedEffectDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "workflow-owner.reviewed-completion.v1";
}
pub struct GrantStatusDelivery;
impl ApplicationActionDelivery<DocumentRetentionSchema, WorkflowGrantStatusBinding>
    for GrantStatusDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "workflow-owner.grant-change-completion.v1";
}
pub type GrantStatusAction = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    WorkflowGrantStatusBinding,
    GrantStatusDelivery,
>;
pub struct SavedSourceDelivery;
impl ApplicationActionDelivery<DocumentRetentionSchema, SetRetentionBinding>
    for SavedSourceDelivery
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "workflow-owner.save-completion.v1";
}
pub type OccurrenceAction = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    OccurrenceSetRetentionBinding,
    WorkflowEffectDelivery,
>;
pub type ReviewedAction = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    ReviewedSetRetentionBinding,
    ReviewedEffectDelivery,
>;
pub type SavedSourceAction = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    SavedSourceDelivery,
>;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for WorkflowOwnerProgram {
    type Contributions = <RetentionProgramP0 as ApplicationProgramDefinition<
        DocumentRetentionSchema,
    >>::Contributions;
    type Outputs =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.workflow-owner.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
                .action::<super::super::correctable_retention::CorrectableRetentionBinding, RetentionCompletion>()
                .finish_action()
                .action::<OccurrenceSetRetentionBinding, WorkflowEffectDelivery>()
                .finish_action()
                .action::<ReviewedSetRetentionBinding, ReviewedEffectDelivery>()
                .finish_action()
                .action::<SetRetentionBinding, SavedSourceDelivery>()
                .finish_action()
                .action::<ExplicitReviewedSetRetentionBinding, RetentionCompletion>()
                .finish_action()
                .action::<ReviewRequirementBinding, RetentionCompletion>()
                .finish_action()
                .action::<UnlinkReviewRequirementBinding, RetentionCompletion>()
                .finish_action()
                .conditional_operation::<PublishRetentionAssessment>()
                .native_action::<WorkflowDefinitionAuthoringBinding>()
                .finish_action()
                .native_action::<WorkflowInstanceStartBinding>()
                .finish_action()
                .native_action::<WorkflowAdvanceBinding>()
                .finish_action()
                .native_action::<WorkflowApprovalBinding>()
                .finish_action()
                .action::<WorkflowGrantStatusBinding, GrantStatusDelivery>()
                .finish_action()
                .finish(),
        ]
    }
}
