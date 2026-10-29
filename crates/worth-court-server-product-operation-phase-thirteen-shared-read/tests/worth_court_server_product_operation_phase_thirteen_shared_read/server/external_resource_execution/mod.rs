#[allow(unused_imports)] use crate::external_resource_execution::ordinary_context::*;
use {serde::Serialize};
use {worth_query::facade::foundation::WorthQueryAsyncFailurePosture, worth_query::facade::foundation::WorthQueryAsyncLoadingPosture, worth_query::facade::foundation::WorthQueryAsyncRequestIdentityPart, worth_query::facade::foundation::WorthQueryAsyncResourceRequestIdentity, worth_query::facade::foundation::WorthQueryAsyncSourceFamily};
use {worth_server::WorthServerExternalResourceBudget, worth_server::WorthServerExternalResourceExecutionBoundary, worth_server::WorthServerExternalResourceExecutionDenialCode, worth_server::WorthServerExternalResourceExecutionOutcome, worth_server::WorthServerExternalResourceIntent, worth_server::WorthServerExternalResourceTransport, worth_server::WorthServerExternalResourceTransportOutcome, worth_server::WorthServerExternalResourceTransportResponse, worth_server::WorthServerLoweredExternalResourcePlan};

#[test]
fn one_shot_resource_execution_carries_exact_bounded_attempt_evidence() {
    let transport = RespondingTransport {
        body: br#"{"rows":[{"id":"user-1"}]}"#.to_vec(),
        evidence: "transport:host-a:attempt-1",
    };
    let boundary = worth_server::WorthServerExternalResourceExecutionBoundary::using(&transport);
    let plan = boundary.plan(intent(1024)).expect("intent should lower");
    let worth_server::WorthServerExternalResourceExecutionOutcome::Completed(completed) = boundary.execute(plan)
    else {
        panic!("bounded response should complete");
    };

    assert_eq!(completed.counters().transport_attempts(), 1);
    assert_eq!(completed.counters().response_bytes(), transport.body.len());
    assert_eq!(completed.transport_evidence_identity(), transport.evidence);
    let result = completed
        .admit_json_result(
            "test.external-resource.users.v1",
            1,
            &Rows {
                rows: vec![Row { id: "user-1" }],
            },
        )
        .expect("validated semantic result should admit");
    assert_eq!(result.schema_version(), 1);
    assert_eq!(result.result_digest().len(), 64);
    assert!(!result.result_digest().contains("user-1"));
}

#[test]
fn response_budget_denial_prevents_result_admission() {
    let transport = RespondingTransport {
        body: vec![b'x'; 65],
        evidence: "transport:oversized",
    };
    let boundary = worth_server::WorthServerExternalResourceExecutionBoundary::using(&transport);
    let plan = boundary.plan(intent(64)).expect("request should lower");

    let worth_server::WorthServerExternalResourceExecutionOutcome::Denied(denial) = boundary.execute(plan) else {
        panic!("oversized response must deny");
    };
    assert_eq!(
        denial.code(),
        worth_server::WorthServerExternalResourceExecutionDenialCode::ResponseBudgetExceeded
    );
    assert_eq!(denial.counters().transport_attempts(), 1);
    assert_eq!(denial.counters().response_bytes(), 65);
}

#[test]
fn transport_stops_remain_typed_and_non_successful() {
    for (outcome, expected) in [
        (
            worth_server::WorthServerExternalResourceTransportOutcome::RejectedBeforeAttempt {
                reason_key: "credential-unresolved".to_string(),
            },
            worth_server::WorthServerExternalResourceExecutionDenialCode::ProviderAdmissionDenied,
        ),
        (
            worth_server::WorthServerExternalResourceTransportOutcome::Denied {
                reason_key: "actor-denied".to_string(),
            },
            worth_server::WorthServerExternalResourceExecutionDenialCode::ProviderDenied,
        ),
        (
            worth_server::WorthServerExternalResourceTransportOutcome::TimedOut,
            worth_server::WorthServerExternalResourceExecutionDenialCode::TimedOut,
        ),
        (
            worth_server::WorthServerExternalResourceTransportOutcome::Unavailable,
            worth_server::WorthServerExternalResourceExecutionDenialCode::Unavailable,
        ),
    ] {
        let transport = FixedTransport(outcome);
        let boundary = worth_server::WorthServerExternalResourceExecutionBoundary::using(&transport);
        let plan = boundary.plan(intent(1024)).expect("request should lower");
        let worth_server::WorthServerExternalResourceExecutionOutcome::Denied(denial) = boundary.execute(plan)
        else {
            panic!("transport stop must not complete");
        };
        assert_eq!(denial.code(), expected);
        let expected_attempts = usize::from(
            expected != worth_server::WorthServerExternalResourceExecutionDenialCode::ProviderAdmissionDenied,
        );
        assert_eq!(denial.counters().transport_attempts(), expected_attempts);
        assert_eq!(denial.counters().response_bytes(), 0);
    }
}

#[test]
fn canonical_result_identity_ignores_transport_identity_but_tracks_semantic_drift() {
    let left = admitted_result("transport:a", "user-1");
    let right = admitted_result("transport:b", "user-1");
    let changed = admitted_result("transport:a", "user-2");

    assert_eq!(left.result_digest(), right.result_digest());
    assert_ne!(left.result_digest(), changed.result_digest());
    assert_ne!(
        left.transport_evidence_identity(),
        right.transport_evidence_identity()
    );
}

#[test]
fn canonical_plan_identity_preserves_arbitrary_request_bytes_losslessly() {
    let boundary = worth_server::WorthServerExternalResourceExecutionBoundary::using(&FixedTransport(
        worth_server::WorthServerExternalResourceTransportOutcome::Unavailable,
    ));
    let left = boundary
        .plan(intent_with_body(1024, vec![0x80]))
        .expect("first binary request should lower");
    let right = boundary
        .plan(intent_with_body(1024, vec![0x81]))
        .expect("second binary request should lower");

    assert_ne!(left.canonical_digest(), right.canonical_digest());
}
