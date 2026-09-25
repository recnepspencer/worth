//! A target program retaining workflow commands but removing assessment supply.

use super::*;

pub struct RemovedAssessmentSupplierProgram;

impl ApplicationProgramDefinition<BoundedDimensionSchema> for RemovedAssessmentSupplierProgram {
    type Contributions = (BoundedDimensionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<BoundedDimensionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.bounded-dimension.no-assessment-supplier.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<BoundedDimensionSchema, BoundedDimensionFeature>()
                .mutation::<SetPartDimensionBinding>()
                .mutation::<ReviewedSetPartDimensionBinding>()
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
) -> ValidatedApplicationProgram<BoundedDimensionSchema, RemovedAssessmentSupplierProgram> {
    validate::<RemovedAssessmentSupplierProgram>()
}
