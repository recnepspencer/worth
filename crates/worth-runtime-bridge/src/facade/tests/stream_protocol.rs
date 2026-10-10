use super::*;
use crate::input::envelope::BridgeCommittedPatchEnvelope;
use crate::truth_identity_fixtures::{truth_branch, truth_commit, truth_patch, truth_snapshot};

#[test]
fn runtime_rejects_replay_record_validation_when_window_basis_changes() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let declaration = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::Prefer(
            crate::stream::StreamCoalescingFamily::RoutingWindowCoalescing,
        ),
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::RouteInvalidations,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let protocol = runtime
        .validate_change_stream_declaration(declaration)
        .expect("stream declarations should validate");
    let contract = runtime
        .resolve_change_stream_consumer_contract(&protocol)
        .expect("stream contract should resolve");
    let original_window = runtime
        .plan_change_stream_window(
            &contract,
            vec![stream_envelope(1, 1), stream_envelope(2, 2)],
        )
        .expect("original window should plan");
    let checkpoint = runtime.publish_consumer_checkpoint(
        &contract,
        &original_window,
        crate::stream::StreamCheckpointFrontierKind::ContiguousFrontier,
    );
    let replay_record = runtime
        .canonicalize_stream_replay_record(&contract, &original_window, &checkpoint)
        .expect("replay record should canonicalize");
    let changed_window = runtime
        .plan_change_stream_window(&contract, vec![stream_envelope(1, 1)])
        .expect("changed window should plan");
    let changed_checkpoint = runtime.publish_consumer_checkpoint(
        &contract,
        &changed_window,
        crate::stream::StreamCheckpointFrontierKind::ContiguousFrontier,
    );

    let error = runtime
        .validate_stream_replay_record(
            &contract,
            &changed_window,
            &changed_checkpoint,
            &replay_record,
        )
        .expect_err("changed windows must fail replay validation");

    assert_eq!(
        error.kind(),
        crate::error::BridgeStreamErrorKind::StreamReplayMismatch
    );
}

#[test]
fn runtime_classifies_width_sensitive_backpressure_without_changing_window_truth() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let declaration = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::Prefer(
            crate::stream::StreamCoalescingFamily::RoutingWindowCoalescing,
        ),
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::RouteInvalidations,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let protocol = runtime
        .validate_change_stream_declaration(declaration)
        .expect("stream declarations should validate");
    let contract = runtime
        .resolve_change_stream_consumer_contract(&protocol)
        .expect("stream contract should resolve");
    let narrow_window = runtime
        .plan_change_stream_window(&contract, vec![stream_envelope(1, 1)])
        .expect("narrow window should plan");
    let burst_window = runtime
        .plan_change_stream_window(
            &contract,
            vec![stream_envelope(1, 1), stream_envelope(2, 2)],
        )
        .expect("burst window should plan");

    let narrow_pressure = runtime.classify_stream_backpressure(&narrow_window);
    let burst_pressure = runtime.classify_stream_backpressure(&burst_window);

    assert_eq!(narrow_pressure.pressure_class(), "no-pressure");
    assert_eq!(burst_pressure.pressure_class(), "elevated-pressure");
    assert_eq!(
        burst_pressure.pressure_reason_family(),
        "coalesced-window-width"
    );
    assert_eq!(narrow_window.counters().stream_member_count(), 1);
    assert_eq!(burst_window.counters().stream_member_count(), 2);
    assert_eq!(burst_window.counters().stream_coalesced_window_count(), 1);
    assert_eq!(
        burst_pressure.counters().stream_backpressure_signal_count(),
        1
    );
    assert_eq!(
        burst_pressure.stream_window_identity(),
        burst_window.stream_window_identity()
    );
}

#[test]
fn runtime_stream_identities_are_invariant_across_diagnostics_tiers() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let standard = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::Prefer(
            crate::stream::StreamCoalescingFamily::RoutingWindowCoalescing,
        ),
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::RouteInvalidations,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let exhaustive = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::Prefer(
            crate::stream::StreamCoalescingFamily::RoutingWindowCoalescing,
        ),
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::RouteInvalidations,
        crate::stream::StreamDiagnosticsPolicyClass::Exhaustive,
    );
    let standard_protocol = runtime
        .validate_change_stream_declaration(standard)
        .expect("standard declaration should validate");
    let exhaustive_protocol = runtime
        .validate_change_stream_declaration(exhaustive)
        .expect("exhaustive declaration should validate");
    let standard_contract = runtime
        .resolve_change_stream_consumer_contract(&standard_protocol)
        .expect("standard contract should resolve");
    let exhaustive_contract = runtime
        .resolve_change_stream_consumer_contract(&exhaustive_protocol)
        .expect("exhaustive contract should resolve");
    let standard_window = runtime
        .plan_change_stream_window(
            &standard_contract,
            vec![stream_envelope(1, 1), stream_envelope(2, 2)],
        )
        .expect("standard window should plan");
    let exhaustive_window = runtime
        .plan_change_stream_window(
            &exhaustive_contract,
            vec![stream_envelope(1, 1), stream_envelope(2, 2)],
        )
        .expect("exhaustive window should plan");

    assert_eq!(
        standard_protocol.stream_protocol_identity(),
        exhaustive_protocol.stream_protocol_identity()
    );
    assert_eq!(
        standard_contract.consumer_contract_identity(),
        exhaustive_contract.consumer_contract_identity()
    );
    assert_eq!(
        standard_window.stream_window_identity(),
        exhaustive_window.stream_window_identity()
    );
    assert_eq!(
        standard_window.member_set_digest(),
        exhaustive_window.member_set_digest()
    );
    let standard_delivery = runtime
        .deliver_change_stream_window(&standard_contract, &standard_window, execution)
        .expect("standard delivery should succeed");
    let exhaustive_delivery = runtime
        .deliver_change_stream_window(&exhaustive_contract, &exhaustive_window, execution)
        .expect("exhaustive delivery should succeed");

    assert_eq!(
        standard_delivery.summary().stream_digest(),
        exhaustive_delivery.summary().stream_digest()
    );
    assert_eq!(
        standard_delivery.summary().window_digest(),
        exhaustive_delivery.summary().window_digest()
    );
    assert_ne!(
        standard_delivery.summary().diagnostics_digest(),
        exhaustive_delivery.summary().diagnostics_digest()
    );
}

#[test]
fn legal_coalescing_changes_window_shape_without_changing_member_meaning() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let narrow_declaration = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::None,
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::RouteInvalidations,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let coalesced_declaration = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::Prefer(
            crate::stream::StreamCoalescingFamily::RoutingWindowCoalescing,
        ),
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::RouteInvalidations,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let narrow_protocol = runtime
        .validate_change_stream_declaration(narrow_declaration)
        .expect("narrow declaration should validate");
    let coalesced_protocol = runtime
        .validate_change_stream_declaration(coalesced_declaration)
        .expect("coalesced declaration should validate");
    let narrow_contract = runtime
        .resolve_change_stream_consumer_contract(&narrow_protocol)
        .expect("narrow contract should resolve");
    let coalesced_contract = runtime
        .resolve_change_stream_consumer_contract(&coalesced_protocol)
        .expect("coalesced contract should resolve");
    let left = runtime
        .plan_change_stream_window(
            &narrow_contract,
            vec![stream_envelope(1, 1), stream_envelope(2, 2)],
        )
        .expect("narrow window should plan");
    let right = runtime
        .plan_change_stream_window(
            &coalesced_contract,
            vec![stream_envelope(1, 1), stream_envelope(2, 2)],
        )
        .expect("coalesced window should plan");

    assert_eq!(left.member_set_digest(), right.member_set_digest());
    assert_eq!(
        left.members()
            .iter()
            .map(|member| member.stream_member_identity().to_owned())
            .collect::<Vec<_>>(),
        right
            .members()
            .iter()
            .map(|member| member.stream_member_identity().to_owned())
            .collect::<Vec<_>>()
    );
    assert_ne!(left.coalescing_family(), right.coalescing_family());
    assert_eq!(left.counters().stream_coalesced_window_count(), 0);
    assert_eq!(right.counters().stream_coalesced_window_count(), 1);
}

fn stream_envelope(commit_position: u64, patch_position: u64) -> BridgeCommittedPatchEnvelope {
    canonical_envelope(
        truth_branch("main"),
        truth_commit(commit_position),
        truth_patch(patch_position),
        truth_snapshot(1, 1),
    )
}

#[test]
fn runtime_rejects_unimplemented_stream_position_resume_during_contract_resolution() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let declaration = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromStreamPosition,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::None,
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::RouteInvalidations,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let protocol = runtime
        .validate_change_stream_declaration(declaration)
        .expect("declaration should validate");

    let error = runtime
        .resolve_change_stream_consumer_contract(&protocol)
        .expect_err("unsupported resume modes must be rejected during contract resolution");

    assert_eq!(
        error.kind(),
        crate::error::BridgeStreamErrorKind::UnsupportedResumeMode
    );
}

#[test]
fn runtime_rejects_delivery_intent_that_conflicts_with_consumer_shape() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let declaration = crate::stream::ChangeStreamDeclaration::new(
        crate::stream::StreamConsumerShape::RoutingConsumer,
        crate::stream::StreamResumeMode::FromCheckpointOnly,
        crate::stream::StreamCheckpointPublicationMode::PublishEveryWindow,
        crate::stream::StreamCoalescingIntent::None,
        crate::stream::StreamReplayMode::Enabled,
        crate::stream::StreamDeliveryIntent::ReplayAudit,
        crate::stream::StreamDiagnosticsPolicyClass::Standard,
    );
    let protocol = runtime
        .validate_change_stream_declaration(declaration)
        .expect("declaration should validate");

    let error = runtime
        .resolve_change_stream_consumer_contract(&protocol)
        .expect_err("consumer-shape and delivery-intent conflicts must be rejected");

    assert_eq!(
        error.kind(),
        crate::error::BridgeStreamErrorKind::UnsupportedConsumerShape
    );
}
