//! masked result field denies before graph planning or runtime.
use super::*;

#[test]
fn masked_result_field_denies_before_graph_planning_or_runtime() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration =
        declare(local_policy_result_read).expect("policy result declaration should canonicalize");
    let policy_tenant = narrowing_policy_tenant_inputs(
        1,
        PolicyAspectMask::allow_all().with_masked(display_name_field().source_field_key().clone()),
    );
    let context = narrowed_context(policy_tenant);
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-policy-result-denied")
        .expect("ordinary workspace should open");

    let stop = declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .into_result()
        .expect_err("masked result fields must deny before planning");
    let denial = stop
        .context_denial()
        .expect("policy narrowing failure must remain a context denial");
    let narrowing_error = match denial.source() {
        WorthQueryReadContextDenialSource::PolicyNarrowing(error) => error,
        source => panic!("expected policy narrowing denial, got {source:?}"),
    };

    assert_eq!(
        narrowing_error.failure_class(),
        PolicyNarrowingFailureClass::AuthorizedProjectionDenied(
            AuthorizedProjectionFailureClass::MaskedProjectionRequested,
        )
    );
    assert_eq!(
        stop.next_action(),
        WorthQueryReadNextAction::SupplyPolicyAuthority
    );
    assert_eq!(denial.counters().policy_tenant_admitted_count(), 1);
    assert_eq!(denial.counters().policy_narrowing_attempt_count(), 1);
    assert_eq!(denial.counters().policy_narrowing_admitted_count(), 0);
    assert_eq!(
        denial
            .counters()
            .relationship_proof_admission_attempt_count(),
        0
    );
    assert_eq!(
        denial.counters().graph_authority_admission_attempt_count(),
        0
    );
    assert_eq!(stop.journey_counters().planning_attempt_count(), 0);
    assert_eq!(stop.journey_counters().planning_completed_count(), 0);
    assert_eq!(
        stop.journey_counters()
            .lower_runtime_execution_attempt_count(),
        0
    );
    assert!(stop.context_receipt().is_none());
}
