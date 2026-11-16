//! Real Directory custody uses installed operations and fresh authenticated readmission.
use super::*;
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
