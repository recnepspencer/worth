//! Ordinary business actions and kernel-owned workflow controls.
use super::*;

pub(super) fn retention_feature_specs() -> Vec<ApplicationFeatureSpec> {
    vec![
        ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
            .action::<super::super::correctable_retention::CorrectableRetentionBinding, RetentionCompletion>()
            .finish_action()
            .action::<SetRetentionBinding, RetentionCompletion>()
            .finish_action()
            .action::<ReviewedSetRetentionBinding, RetentionCompletion>()
            .finish_action()
            .action::<OccurrenceSetRetentionBinding, RetentionCompletion>()
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
            .action::<WorkflowGrantStatusBinding, RetentionCompletion>()
            .finish_action()
            .finish(),
    ]
}
