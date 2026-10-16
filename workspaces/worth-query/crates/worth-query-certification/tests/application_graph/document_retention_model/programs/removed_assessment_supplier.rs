//! A target program retaining workflow commands but removing assessment supply.

use super::*;

pub struct RemovedAssessmentSupplierProgram;

impl ApplicationProgramDefinition<DocumentRetentionSchema> for RemovedAssessmentSupplierProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.no-assessment-supplier.v1",
    );

    fn adoption_authorization<Visitor: worth_query_host::facade::declaration::application_program::ApplicationProgramAdoptionAuthorizationVisitor<DocumentRetentionSchema>>(visitor: Visitor) -> Visitor::Output{
        visitor.capability::<super::super::program_adoption::ProgramAdoptionBinding>()
    }

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
                .action::<SetRetentionBinding, RetentionCompletion>()
                .finish_action()
                .action::<ReviewedSetRetentionBinding, RetentionCompletion>()
                .finish_action()
                .action::<ReviewRequirementBinding, RetentionCompletion>()
                .finish_action()
                .action::<UnlinkReviewRequirementBinding, RetentionCompletion>()
                .finish_action()
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

pub fn validated_removed_assessment_supplier_program()
-> ValidatedApplicationProgram<DocumentRetentionSchema, RemovedAssessmentSupplierProgram> {
    validate::<RemovedAssessmentSupplierProgram>()
}
