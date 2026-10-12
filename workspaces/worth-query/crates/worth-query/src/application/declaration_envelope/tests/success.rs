use crate::application::{WorthQueryDeclarationEnvelopeInput, WorthQueryDeclarationReceiptKind};

use super::support::{
    admitted_handle, envelope_checked_from_receipt, progressed, route_checked_with_intent,
    EnvelopeInput, MixedEnvelopeFamily, RelationalEnvelopeFamily,
};

#[test]
fn envelope_common_lane_reads_like_public_crossing_story() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let envelope = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_and_envelope(EnvelopeInput::<
            RelationalEnvelopeFamily,
        >::new("edge:42"))
        .unwrap_or_else(|_| panic!("envelope common lane should issue"));

    assert_eq!(
        envelope.declaration_family_key(),
        "RelationalEnvelopeFamily"
    );
    assert!(envelope.explain().crossing_posture().contains("successful"));
}

#[test]
fn explicit_and_common_envelope_paths_converge_on_one_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");
    let explicit = handle
        .envelope_routes_from_progressed(progressed(
            &handle,
            EnvelopeInput::<MixedEnvelopeFamily>::new("edge:42"),
        ))
        .unwrap_or_else(|_| panic!("explicit envelope path should issue"));
    let common = handle
        .declare_review_progress_describe_plan_receipt_and_envelope(EnvelopeInput::<
            MixedEnvelopeFamily,
        >::new("edge:42"))
        .unwrap_or_else(|_| panic!("common envelope path should issue"));

    assert_eq!(explicit.envelope_digest(), common.envelope_digest());
}

#[test]
fn mixed_crossing_envelopes_retain_mixed_route_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let envelope = admitted_handle(request_execution, "primary")
        .declare_review_progress_describe_plan_receipt_and_envelope(EnvelopeInput::<
            MixedEnvelopeFamily,
        >::new("edge:42"))
        .unwrap_or_else(|_| panic!("mixed envelope should issue"));

    assert_eq!(
        envelope.receipt().kind(),
        WorthQueryDeclarationReceiptKind::Mixed
    );
    assert_eq!(
        envelope
            .route_plan()
            .expect("route plan should be retained")
            .route_count(),
        2
    );
}

#[test]
fn advanced_lane_envelopes_issue_without_checked_wrapper_loss() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    match route_checked_with_intent(
        &handle,
        EnvelopeInput::<RelationalEnvelopeFamily>::new("edge:42"),
        crate::application::WorthQueryDeclarationRouteIntent::Auto,
    ) {
        crate::application::WorthQueryDeclarationRoutePlanChecked::Planned(plan) => {
            let receipt = handle
                .receipt_routes(
                    crate::application::WorthQueryDeclarationReceiptInput::planned(plan),
                )
                .unwrap_or_else(|_| panic!("planned route should issue receipt"));
            let envelope = handle
                .envelope_routes(WorthQueryDeclarationEnvelopeInput::issued(receipt))
                .unwrap_or_else(|_| panic!("advanced envelope lane should issue"));
            assert_eq!(
                envelope.receipt_digest(),
                envelope.receipt().receipt_digest()
            );
        }
        _ => panic!("relational route plan should be planned"),
    }
}

#[test]
fn issued_receipt_checked_input_preserves_public_envelope_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "primary");

    let checked_receipt = handle.receipt_routes_checked(
        crate::application::WorthQueryDeclarationReceiptInput::planned(
            handle
                .plan_routes_from_progressed(progressed(
                    &handle,
                    EnvelopeInput::<RelationalEnvelopeFamily>::new("edge:42"),
                ))
                .unwrap_or_else(|_| panic!("planned route should exist")),
        ),
    );

    match handle.envelope_routes_checked(WorthQueryDeclarationEnvelopeInput::receipt_checked(
        checked_receipt,
    )) {
        crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope) => {
            assert_eq!(
                envelope.explain().evidence_origin(),
                crate::application::WorthQueryDeclarationEnvelopeEvidenceOrigin::AdmittedProgression
            );
        }
        _ => panic!("issued checked receipts should envelope without loss"),
    }
}

#[test]
fn envelope_digest_changes_when_admitted_world_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let primary = match envelope_checked_from_receipt(
        &admitted_handle(request_execution, "primary"),
        EnvelopeInput::<RelationalEnvelopeFamily>::new("edge:42"),
    ) {
        crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope) => envelope,
        _ => panic!("primary world should envelope"),
    };
    let alternate = match envelope_checked_from_receipt(
        &admitted_handle(request_execution, "alternate"),
        EnvelopeInput::<RelationalEnvelopeFamily>::new("edge:42"),
    ) {
        crate::application::WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope) => envelope,
        _ => panic!("alternate world should envelope"),
    };

    assert_ne!(primary.envelope_digest(), alternate.envelope_digest());
}
