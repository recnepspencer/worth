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

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
                .mutation::<SetRetentionBinding>()
                .mutation::<ReviewedSetRetentionBinding>()
                .mutation::<ReviewRequirementBinding>()
                .mutation::<UnlinkReviewRequirementBinding>()
                .mutation::<WorkflowDefinitionAuthoringBinding>()
                .mutation::<WorkflowInstanceStartBinding>()
                .mutation::<WorkflowAdvanceBinding>()
                .mutation::<WorkflowApprovalBinding>()
                .mutation::<WorkflowGrantStatusBinding>()
                .finish(),
        ]
    }
}

pub fn validated_removed_assessment_supplier_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, RemovedAssessmentSupplierProgram> {
    validate::<RemovedAssessmentSupplierProgram>()
}
