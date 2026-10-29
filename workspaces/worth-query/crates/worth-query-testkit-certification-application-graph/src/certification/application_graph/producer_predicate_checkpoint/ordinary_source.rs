//! Reuse the installed assessment owners with an ordinary required-output source.
pub use super::program::{AssessmentFeature, AssessmentProgram, RetentionInput, RetentionOutput};
pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    assessment_output::{
        AssessmentOutput, PublishRetentionAssessment, RetentionAssessmentDemand,
        RetentionAssessmentOutputs,
    },
    host::publish,
    operator_identity::{authenticate_operator, request_scope},
    programs::{validated_second_program, DocumentRetentionFeature, RetentionCompletion},
    retention_entry::{
        DocumentRetentionRead, ReviewedSetRetentionBinding, SetRetentionBinding,
        SetRetentionIntent, DOCUMENT_IDENTITY,
    },
    schema::{DocumentRetentionSchema, SetRetentionInput},
};
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
        WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
    },
    application_installation::WorthQueryApplicationProgramRoster,
    declaration::application_program::*,
    primary_graph::{
        WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
        WorthQueryMutationHandlerWork, WorthQueryRequiredOutputConnectionDenial,
    },
};

pub struct OrdinaryConnection;
pub struct OrdinaryAssessmentProgram;
pub type OrdinaryRoot = ApplicationOutputGraph<
    ApplicationConnectionRef<
        DocumentRetentionSchema,
        DocumentRetentionFeature,
        RetentionOutput,
        AssessmentFeature,
        RetentionInput,
        OrdinaryConnection,
    >,
    ApplicationOutputLeaf,
>;
impl
    ApplicationOccurrenceConnectionBinding<
        DocumentRetentionSchema,
        DocumentRetentionFeature,
        AssessmentFeature,
    > for OrdinaryConnection
{
}
impl ApplicationConnectionIdentity for OrdinaryConnection {
    const IDENTITY: &'static str = "worth.query.certification.ordinary-assessment.connection.v1";
}
impl WorthQueryApplicationRequiredOutputConnection<DocumentRetentionSchema> for OrdinaryConnection {
    type Source = SetRetentionBinding;
    type Demand = RetentionAssessmentDemand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        source: &SetRetentionInput,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(RetentionAssessmentDemand::new(&source.identity))
    }
}
impl WorthQueryApplicationRequiredOutputSource<DocumentRetentionSchema, OrdinaryConnection>
    for SetRetentionBinding
{
    fn demand_from_source(
        source: &SetRetentionInput,
    ) -> Result<RetentionAssessmentDemand, WorthQueryRequiredOutputConnectionDenial> {
        <OrdinaryConnection as WorthQueryApplicationRequiredOutputConnection<
            DocumentRetentionSchema,
        >>::demand_from_source(source)
    }
}
pub struct OrdinaryAssessmentDelivery;
impl ApplicationActionDelivery<DocumentRetentionSchema, SetRetentionBinding>
    for OrdinaryAssessmentDelivery
{
    type Mode = ApplicationFixedDeliveryMode<OrdinaryRoot, ApplicationNoProjection>;
    const IDENTITY: &'static str = "ordinary-assessment.required-completion.v1";
}
impl ApplicationFixedActionDelivery<DocumentRetentionSchema, SetRetentionBinding>
    for OrdinaryAssessmentDelivery
{
    type Root = OrdinaryRoot;
    type Projection = ApplicationNoProjection;
}

impl ApplicationProgramDefinition<DocumentRetentionSchema> for OrdinaryAssessmentProgram {
    type Contributions =
        <AssessmentProgram as ApplicationProgramDefinition<DocumentRetentionSchema>>::Contributions;
    type Outputs = ApplicationProgramOutputs<OrdinaryRoot>;
    type Rules =
        <AssessmentProgram as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.ordinary-assessment.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
                .provides::<RetentionOutput>()
                .action::<SetRetentionBinding, OrdinaryAssessmentDelivery>()
                .finish_action()
                .action::<ReviewedSetRetentionBinding, RetentionCompletion>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, AssessmentFeature>()
                .conditional_operation::<PublishRetentionAssessment>()
                .finish(),
        ]
    }
}

#[path = "ordinary_source/terminal_unavailable.rs"]
pub mod terminal_unavailable;
