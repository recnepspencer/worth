//! Rostered successors change the declared feature or action contract.
use super::*;

impl ApplicationProgramDefinition<DocumentRetentionSchema> for ChangedFeatureRetentionProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.changed-feature.v1",
    );

    fn adoption_authorization<Visitor: worth_query_host::facade::declaration::application_program::ApplicationProgramAdoptionAuthorizationVisitor<DocumentRetentionSchema>>(visitor: Visitor) -> Visitor::Output{
        visitor.capability::<super::program_adoption::ProgramAdoptionBinding>()
    }

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeatureV2>()
                .action::<SetRetentionBinding, RetentionCompletion>()
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
}

impl ApplicationProgramDefinition<DocumentRetentionSchema> for RemovedOperationRetentionProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.removed-operation.v1",
    );

    fn adoption_authorization<Visitor: worth_query_host::facade::declaration::application_program::ApplicationProgramAdoptionAuthorizationVisitor<DocumentRetentionSchema>>(visitor: Visitor) -> Visitor::Output{
        visitor.capability::<super::program_adoption::ProgramAdoptionBinding>()
    }

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
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
}

impl ApplicationProgramDefinition<DocumentRetentionSchema> for ChangedOperationRetentionProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.changed-operation.v1",
    );

    fn adoption_authorization<Visitor: worth_query_host::facade::declaration::application_program::ApplicationProgramAdoptionAuthorizationVisitor<DocumentRetentionSchema>>(visitor: Visitor) -> Visitor::Output{
        visitor.capability::<super::program_adoption::ProgramAdoptionBinding>()
    }

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
                .action::<SetRetentionBinding, RetentionCompletion>()
                .locality_and_change::<ChangedOperationLocality, ChangedOperationShape>()
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
}
