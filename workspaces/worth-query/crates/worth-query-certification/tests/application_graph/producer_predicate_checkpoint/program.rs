//! A real required-output program over the existing retention contribution.
use crate::document_retention_model::{
    assessment_output::{PublishRetentionAssessment, RetentionAssessmentDemand},
    programs::{DocumentRetentionFeature, RetentionProgramP0},
    retention_entry::{ReviewedSetRetentionBinding, SetRetentionBinding},
    schema::{
        DocumentRetentionContribution, DocumentRetentionRowBinding, DocumentRetentionSchema,
        SetRetentionInput,
    },
};
use worth_query_host::facade::declaration::application_program::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

pub struct AssessmentProgram;
pub struct AssessmentFeature;
pub struct RetentionOutput;
pub struct RetentionInput;
pub struct AssessmentConnection;
pub type AssessmentRoot = ApplicationOutputGraph<
    ApplicationConnectionRef<
        DocumentRetentionSchema,
        DocumentRetentionFeature,
        RetentionOutput,
        AssessmentFeature,
        RetentionInput,
        AssessmentConnection,
    >,
    ApplicationOutputLeaf,
>;

impl ApplicationFeature<DocumentRetentionSchema> for AssessmentFeature {
    type Inputs = ApplicationFeatureInputList<RetentionInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.predicate-assessment.feature.v1";
}
impl ApplicationOutputPort<DocumentRetentionSchema, DocumentRetentionFeature> for RetentionOutput {
    type Value = DocumentRetentionRowBinding;
    const IDENTITY: &'static str = "retention";
}
impl ApplicationInputPort<DocumentRetentionSchema, AssessmentFeature> for RetentionInput {
    type Value = DocumentRetentionRowBinding;
    const IDENTITY: &'static str = "document";
    const REQUIRED: bool = true;
}
impl
    ApplicationOccurrenceConnectionBinding<
        DocumentRetentionSchema,
        DocumentRetentionFeature,
        AssessmentFeature,
    > for AssessmentConnection
{
}
impl ApplicationConnectionIdentity for AssessmentConnection {
    const IDENTITY: &'static str = "worth.query.certification.predicate-assessment.connection.v1";
}
impl WorthQueryApplicationRequiredOutputConnection<DocumentRetentionSchema>
    for AssessmentConnection
{
    type Source = ReviewedSetRetentionBinding;
    type Demand = RetentionAssessmentDemand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        source: &SetRetentionInput,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(RetentionAssessmentDemand::new(&source.identity))
    }
}
impl ApplicationProgramDefinition<DocumentRetentionSchema> for AssessmentProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<AssessmentRoot>;
    type Rules =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.predicate-assessment.program.v1",
    );
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
                .provides::<RetentionOutput>()
                .mutation::<SetRetentionBinding>()
                .mutation::<ReviewedSetRetentionBinding>()
                .finish(),
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, AssessmentFeature>()
                .conditional_operation::<PublishRetentionAssessment>()
                .finish(),
        ]
    }
}
pub fn validated_program() -> ValidatedApplicationProgram<DocumentRetentionSchema, AssessmentProgram>
{
    ApplicationProgramAuthoring::<DocumentRetentionSchema, AssessmentProgram>::begin()
        .validated_program()
        .expect("the actual assessment program must validate")
}
