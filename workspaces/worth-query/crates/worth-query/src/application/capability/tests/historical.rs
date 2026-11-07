use super::WorthQueryApplicationFacade;
use crate::historical::{
    HistoricalCapabilityDescriptor, HistoricalEvaluationRequest, HistoricalPathReuseDescriptor,
};

#[test]
fn historical_capability_admits_runtime_retained_snapshot_request() {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let historical = facade
        .historical_query_capability()
        .expect("runtime-backed facade should admit historical evaluation");
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "basis:historical-facade",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::retained_snapshot_for_test(
        "basis:historical-facade",
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let admission = historical
        .capability()
        .admit_path(request, capability)
        .expect("historical capability should admit a retained snapshot request");

    assert_eq!(
        admission.requested_path().basis_identity(),
        "basis:historical-facade"
    );
    assert_eq!(historical.counters().capability_lookup_count(), 1);
}
