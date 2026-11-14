//! Real Directory operations refuse unsupported custody before contacting their handlers.
use super::*;
use worth_foundational::facade::DurableStateFamily as Family;
use worth_query_host::facade::product::WorthQueryDurableStateAdmissionDenial as Durable;
#[test]
fn directory_ordinary_output_demand_reports_an_absent_producer_after_its_source_query() {
    use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind as Output;
    let path = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(path.clone(), execution);
    {
        let request = scope();
        let principal = label::authentication::authenticate_external(
            home.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &request,
        );
        let request = home.runtime().request(&principal, &request);
        let rows = request
            .query(label::AccountSummaryParameters)
            .execute()
            .unwrap();
        assert_eq!(rows.rows()[0].label, "primary");
        let denied = request
            .demand(CurrentAccountDemand)
            .start()
            .err()
            .expect("the real source query reaches the absent producer");
        let worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denied) = denied else { panic!("the completed source must reach demand admission"); };
        assert!(
            matches!(denied.kind(), Output::MissingApplicableProducer),
            "{denied:?}"
        );
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("output refusal close: {cause:?}"))
        .reclaim();
    let home = open(path, execution);
    drop(edit(&home));
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("ordinary write after refusal reopen: {cause:?}"))
        .reclaim();
}

#[test]
fn directory_original_operation_owner_refuses_before_retaining_runtime_custody() {
    use worth_query_host::facade::application_entry::WorthQueryOperationConstructionDenial as Denial;
    let path = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(path.clone(), execution);
    {
        let request = scope();
        let principal = label::authentication::authenticate_external(
            home.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &request,
        );
        let description = home
            .runtime()
            .request(&principal, &request)
            .mutate(LabelEdit {
                status: "open".into(),
                label: "retained".into(),
            })
            .without_source()
            .idempotency_owned("original-owner".to_owned())
            .unwrap_or_else(|(_, _, cause)| panic!("capture original: {cause:?}"));
        let (description, denied) = description
            .retain_operation::<program::HomeLabelProgram, program::LabelAction>(home.runtime())
            .err()
            .expect("original owner has no cold custody yet");
        assert!(matches!(
            denied,
            Denial::DurableStateAdmission(Durable::Unsupported {
                family: Family::OriginalOperationDescription,
                supported_from_phase: 9
            })
        ));
        drop(description);
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("returned original description: {cause:?}"))
        .reclaim();
    let home = open(path, execution);
    drop(edit(&home));
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("ordinary continuation: {cause:?}"))
        .reclaim();
}

struct CurrentAccountDemand;
impl
    worth_query_host::facade::application_contribution::WorthQueryApplicationOutputDemand<
        label::IdentityExecutionSchema,
    > for CurrentAccountDemand
{
    type OutputFamily = label::retained_output_capacity::RetainedFamily;
    fn source_intent(&self) -> label::AccountSummaryParameters {
        label::AccountSummaryParameters
    }
}
