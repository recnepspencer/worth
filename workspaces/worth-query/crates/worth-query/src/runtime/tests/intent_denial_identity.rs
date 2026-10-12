use super::support::*;
use crate::WorthQueryEvidenceScope;

#[test]
fn invariant_denial_retains_typed_snapshot_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = bridge_backed_runtime_with_support_and_intent_authority(
        request_execution,
        intent_support_profile(),
        InvariantViolationIntentAuthority,
    );

    let error = runtime
        .execute_intent(
            request_execution,
            WorthQueryIntentDeclaration::strategy_commit(
                "typed-denial-snapshot",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1"), ("dependency", "cycle")]),
            ),
        )
        .expect_err("invariant violation should deny before publication");

    let WorthQueryRuntimeError::IntentCommitDenied { evidence, .. } = error else {
        panic!("expected intent denial error");
    };
    let snapshot_identity = evidence
        .snapshot_identity()
        .expect("denial should retain attempted execution snapshot identity");
    assert_eq!(
        snapshot_identity.evidence_identity().scope(),
        WorthQueryEvidenceScope::WriteReceiptSnapshotIdentity
    );

    let inspection = runtime
        .inspect_intent_denial(request_execution, &evidence)
        .expect("denial evidence should inspect");
    assert_eq!(
        inspection.snapshot_identity(),
        Some(snapshot_identity),
        "inspection must retain the same typed snapshot handle"
    );
    assert_eq!(inspection.denial_identity(), evidence.denial_digest());
}
