pub(crate) use worth_query_worlds::facade::certification::application_graph::canonical_admission::ordinary_attempt::*;

#[test]
fn ordinary_attempt_refusal_cannot_be_swallowed_before_fact_completion() {
    let funded = host::install(super::FUNDED_WORK);
    let positive = observe_twice(&funded);
    assert!(!positive.second_refused);
    positive
        .completed
        .expect("sufficient admitted ordinary observations complete real facts");
    assert_eq!(super::opaque::ignored_refusals(), 0);
    drop(funded);

    let bounded = host::install(super::ONE_VALUE_WORK);
    let refused = observe_twice(&bounded);
    assert!(
        refused.second_refused,
        "the cumulative second observation must refuse under the original request"
    );
    assert!(
        super::opaque::ignored_refusals() > 0,
        "the nested seed must have swallowed an actual native admission refusal"
    );
    let denial = refused
        .completed
        .expect_err("ignored observation failure cannot seal completed facts");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::ValueDecode(
            WorthQueryValueDecodeDenial::Checkpoint(worth_execution::MapKernelStop::WorkCeiling)
        )
    );
    assert_eq!(
        host::read(&bounded, "root").value,
        super::value(),
        "a fresh single-decode request is not poisoned"
    );
    let checkpoint = host::capture(&bounded);
    drop(bounded);
    let restored = host::restore(checkpoint, super::FUNDED_WORK);
    let retried = observe_twice(&restored);
    assert!(!retried.second_refused);
    retried
        .completed
        .expect("fresh sufficient current-format recovery completes ordinary facts");
    assert_eq!(host::read(&restored, "root").counter, 0);
}
