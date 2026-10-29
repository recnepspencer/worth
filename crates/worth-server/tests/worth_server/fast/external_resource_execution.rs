use serde::Serialize;
use worth_query::facade::foundation::{
    WorthQueryAsyncFailurePosture, WorthQueryAsyncLoadingPosture,
    WorthQueryAsyncRequestIdentityPart, WorthQueryAsyncResourceRequestIdentity,
    WorthQueryAsyncSourceFamily,
};
use worth_server::{
    WorthServerExternalResourceBudget, WorthServerExternalResourceExecutionBoundary,
    WorthServerExternalResourceExecutionDenialCode, WorthServerExternalResourceExecutionOutcome,
    WorthServerExternalResourceIntent, WorthServerExternalResourceTransport,
    WorthServerExternalResourceTransportOutcome, WorthServerExternalResourceTransportResponse,
    WorthServerLoweredExternalResourcePlan,
};

#[test]
fn one_shot_resource_execution_carries_exact_bounded_attempt_evidence() {
    let transport = RespondingTransport {
        body: br#"{"rows":[{"id":"user-1"}]}"#.to_vec(),
        evidence: "transport:host-a:attempt-1",
    };
    let boundary = WorthServerExternalResourceExecutionBoundary::using(&transport);
    let plan = boundary.plan(intent(1024)).expect("intent should lower");
    let WorthServerExternalResourceExecutionOutcome::Completed(completed) = boundary.execute(plan)
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
    let boundary = WorthServerExternalResourceExecutionBoundary::using(&transport);
    let plan = boundary.plan(intent(64)).expect("request should lower");

    let WorthServerExternalResourceExecutionOutcome::Denied(denial) = boundary.execute(plan) else {
        panic!("oversized response must deny");
    };
    assert_eq!(
        denial.code(),
        WorthServerExternalResourceExecutionDenialCode::ResponseBudgetExceeded
    );
    assert_eq!(denial.counters().transport_attempts(), 1);
    assert_eq!(denial.counters().response_bytes(), 65);
}

#[test]
fn transport_stops_remain_typed_and_non_successful() {
    for (outcome, expected) in [
        (
            WorthServerExternalResourceTransportOutcome::RejectedBeforeAttempt {
                reason_key: "credential-unresolved".to_string(),
            },
            WorthServerExternalResourceExecutionDenialCode::ProviderAdmissionDenied,
        ),
        (
            WorthServerExternalResourceTransportOutcome::Denied {
                reason_key: "actor-denied".to_string(),
            },
            WorthServerExternalResourceExecutionDenialCode::ProviderDenied,
        ),
        (
            WorthServerExternalResourceTransportOutcome::TimedOut,
            WorthServerExternalResourceExecutionDenialCode::TimedOut,
        ),
        (
            WorthServerExternalResourceTransportOutcome::Unavailable,
            WorthServerExternalResourceExecutionDenialCode::Unavailable,
        ),
    ] {
        let transport = FixedTransport(outcome);
        let boundary = WorthServerExternalResourceExecutionBoundary::using(&transport);
        let plan = boundary.plan(intent(1024)).expect("request should lower");
        let WorthServerExternalResourceExecutionOutcome::Denied(denial) = boundary.execute(plan)
        else {
            panic!("transport stop must not complete");
        };
        assert_eq!(denial.code(), expected);
        let expected_attempts = usize::from(
            expected != WorthServerExternalResourceExecutionDenialCode::ProviderAdmissionDenied,
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
    let boundary = WorthServerExternalResourceExecutionBoundary::using(&FixedTransport(
        WorthServerExternalResourceTransportOutcome::Unavailable,
    ));
    let left = boundary
        .plan(intent_with_body(1024, vec![0x80]))
        .expect("first binary request should lower");
    let right = boundary
        .plan(intent_with_body(1024, vec![0x81]))
        .expect("second binary request should lower");

    assert_ne!(left.canonical_digest(), right.canonical_digest());
}

fn admitted_result(
    evidence: &'static str,
    id: &'static str,
) -> worth_server::WorthServerAdmittedExternalResourceResult {
    let transport = RespondingTransport {
        body: br#"{"rows":[]}"#.to_vec(),
        evidence,
    };
    let boundary = WorthServerExternalResourceExecutionBoundary::using(&transport);
    let plan = boundary.plan(intent(1024)).unwrap();
    let WorthServerExternalResourceExecutionOutcome::Completed(completed) = boundary.execute(plan)
    else {
        panic!("transport should complete");
    };
    completed
        .admit_json_result(
            "test.external-resource.users.v1",
            1,
            &Rows {
                rows: vec![Row { id }],
            },
        )
        .unwrap()
}

fn intent(max_response_bytes: usize) -> WorthServerExternalResourceIntent {
    intent_with_body(max_response_bytes, br#"{"selector":"owner"}"#.to_vec())
}

fn intent_with_body(
    max_response_bytes: usize,
    request_body: Vec<u8>,
) -> WorthServerExternalResourceIntent {
    let request_identity = WorthQueryAsyncResourceRequestIdentity::declare(
        WorthQueryAsyncSourceFamily::HostResource,
        WorthQueryAsyncLoadingPosture::Blocking,
        WorthQueryAsyncFailurePosture::FailClosed,
        vec![
            WorthQueryAsyncRequestIdentityPart::text("source", "users"),
            WorthQueryAsyncRequestIdentityPart::text("selector", "owner"),
        ],
    )
    .unwrap();
    WorthServerExternalResourceIntent::builder()
        .with_request_identity(request_identity)
        .with_provider_identity("test-host-provider")
        .with_contract_identity("test.users.v1")
        .with_basis_identity("basis:test")
        .with_request_body(request_body)
        .with_budget(
            WorthServerExternalResourceBudget::bounded(1024, max_response_bytes, 5_000).unwrap(),
        )
        .build()
        .unwrap()
}

#[derive(Debug)]
struct RespondingTransport {
    body: Vec<u8>,
    evidence: &'static str,
}

impl WorthServerExternalResourceTransport for RespondingTransport {
    fn execute(
        &self,
        _plan: &WorthServerLoweredExternalResourcePlan,
    ) -> WorthServerExternalResourceTransportOutcome {
        WorthServerExternalResourceTransportOutcome::Responded(
            WorthServerExternalResourceTransportResponse::new(self.body.clone(), self.evidence)
                .unwrap(),
        )
    }
}

#[derive(Debug)]
struct FixedTransport(WorthServerExternalResourceTransportOutcome);

impl WorthServerExternalResourceTransport for FixedTransport {
    fn execute(
        &self,
        _plan: &WorthServerLoweredExternalResourcePlan,
    ) -> WorthServerExternalResourceTransportOutcome {
        self.0.clone()
    }
}

#[derive(Serialize)]
struct Rows<'a> {
    rows: Vec<Row<'a>>,
}

#[derive(Serialize)]
struct Row<'a> {
    id: &'a str,
}
