//! authoritative intent receipt identity keeps typed scope under delimiter pressure.
use super::*;

#[test]
fn authoritative_intent_receipt_identity_keeps_typed_scope_under_delimiter_pressure() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime =
        bridge_backed_runtime_with_support(request_execution, intent_support_profile());

    let left = runtime
        .execute_intent(
            request_execution,
            WorthQueryIntentDeclaration::strategy_commit(
                "intent|receipt",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task|1"), ("title", "alpha:beta")]),
            ),
        )
        .expect("left intent should execute");
    let right = runtime
        .execute_intent(
            request_execution,
            WorthQueryIntentDeclaration::strategy_commit(
                "intent",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "receipt|task"), ("title", "alpha:beta")]),
            ),
        )
        .expect("right intent should execute");

    assert_eq!(
        left.receipt_identity().scope(),
        WorthQueryEvidenceScope::AuthoritativeIntentReceipt
    );
    assert_eq!(
        left.execution_provenance()
            .execution_provenance_chain_identity()
            .scope(),
        WorthQueryEvidenceScope::IntentExecutionProvenanceChain
    );
    assert_ne!(
        left.receipt_identity(),
        right.receipt_identity(),
        "intent receipt identity must not collapse delimiter-shaped field boundaries"
    );
    assert_ne!(
        left.execution_provenance()
            .execution_provenance_chain_identity(),
        right
            .execution_provenance()
            .execution_provenance_chain_identity(),
        "provenance chain identity must not collapse delimiter-shaped field boundaries"
    );
    assert_eq!(
        left.receipt_identity(),
        &compose_authoritative_intent_receipt_identity(&left),
        "runtime authoritative intent receipt identity must match independent typed composition"
    );
    assert_eq!(
        left.execution_provenance()
            .execution_provenance_chain_identity(),
        &compose_intent_execution_provenance_chain_identity(&left),
        "runtime provenance chain identity must match independent typed composition"
    );

    let inspection = runtime
        .inspect_intent_receipt(request_execution, &left)
        .expect("authoritative receipt inspection should inspect");
    let right_inspection = runtime
        .inspect_intent_receipt(request_execution, &right)
        .expect("right authoritative receipt inspection should inspect");
    assert_eq!(
        inspection.delivery_counters().counter_identity().scope(),
        WorthQueryEvidenceScope::IntentInspectionDeliveryCounters
    );
    assert_eq!(
        inspection.inspection_identity().scope(),
        WorthQueryEvidenceScope::IntentReceiptInspection
    );
    assert_eq!(
        inspection.inspection_identity(),
        &compose_authoritative_intent_receipt_inspection_identity(&inspection),
        "authoritative inspection identity must match typed composition"
    );
    assert_eq!(
        inspection.receipt_identity(),
        left.receipt_identity(),
        "authoritative inspection should retain the typed receipt identity"
    );
    assert_ne!(
        inspection.inspection_identity(),
        right_inspection.inspection_identity(),
        "authoritative inspection identity must not collapse delimiter-shaped receipt fields"
    );
}
