//! Reuse the installed assessment owners with an ordinary required-output source.
use super::program::{AssessmentFeature, AssessmentProgram, RetentionInput, RetentionOutput};
use crate::document_retention_model::{
    assessment_output::{AssessmentOutput, RetentionAssessmentDemand, RetentionAssessmentOutputs},
    host::publish,
    operator_identity::{authenticate_operator, request_scope},
    programs::{validated_second_program, DocumentRetentionFeature},
    retention_entry::{
        DocumentRetentionRead, SetRetentionBinding, SetRetentionIntent, DOCUMENT_IDENTITY,
    },
    schema::{DocumentRetentionSchema, SetRetentionInput},
};
use worth_query_host::facade::{
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

struct OrdinaryConnection;
struct OrdinaryAssessmentProgram;
type OrdinaryRoot = ApplicationOutputGraph<
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
impl ApplicationProgramDefinition<DocumentRetentionSchema> for OrdinaryAssessmentProgram {
    type Contributions =
        <AssessmentProgram as ApplicationProgramDefinition<DocumentRetentionSchema>>::Contributions;
    type Outputs = ApplicationProgramOutputs<OrdinaryRoot>;
    type Rules =
        <AssessmentProgram as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.ordinary-assessment.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <AssessmentProgram as ApplicationProgramDefinition<DocumentRetentionSchema>>::feature_specs(
        )
    }
}

#[test]
fn performed_mutation_attempt_report_preserves_source_custody() {
    let program =
        ApplicationProgramAuthoring::<DocumentRetentionSchema, OrdinaryAssessmentProgram>::begin()
            .validated_program()
            .expect("the ordinary source graph must validate");
    let host = publish(
        program,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .unwrap();
    let runtime = host.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&principal, &scope);
    let report = request
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.into(),
                retention_days: 6,
            },
        })
        .without_source()
        .idempotency(&0x86_u64)
        .execute_performed_report::<OrdinaryAssessmentProgram, OrdinaryRoot>(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let WorthQueryMutationHandlerWork::Captured(work) = report.decision_work() else {
        panic!("the performed source must carry its actual execution capture")
    };
    assert!(work.handler_contacted());
    assert!(work.projection_work().field_reads() > 0);
    let outcome = report
        .into_outcome()
        .expect("the actual source reaches publication");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the ordinary source must land with required-output custody")
    };
    assert_eq!(
        request
            .query(DocumentRetentionRead {
                identity: DOCUMENT_IDENTITY.into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        6
    );
    let receipt = performed.receipt().clone();
    drop(performed);
    let controls = WorthQueryOutputDemandControls::new(
        std::num::NonZeroUsize::new(4096).unwrap(),
        std::num::NonZeroUsize::new(8192).unwrap(),
    );
    let mut recovered = request
        .recover_required_outputs::<OrdinaryAssessmentProgram, OrdinaryRoot>(
            &host,
            &receipt,
            RetentionAssessmentDemand::new(DOCUMENT_IDENTITY),
            controls,
        )
        .expect("caller disposal must leave exact source custody recoverable by its owner");
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) =
        recovered.settle(&host, &request).unwrap()
    else {
        panic!("the existing installed assessment producer must settle")
    };
    assert_eq!(
        request
            .at(settled.observation())
            .query(DocumentRetentionRead {
                identity: DOCUMENT_IDENTITY.into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        6
    );
    assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
    assert!(settled
        .root_receipt()
        .unwrap()
        .outputs_of::<RetentionAssessmentOutputs>()
        .unwrap()
        .entity::<AssessmentOutput>()
        .is_ok());
}

#[path = "ordinary_source/unpublished.rs"]
mod unpublished;
