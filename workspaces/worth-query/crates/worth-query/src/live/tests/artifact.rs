use crate::live::*;
#[test]
fn public_live_artifact_builder_summarizes_execution_and_rejection_lanes() {
    let preflight = crate::harness::fixtures::execution_preflights::direct_runtime_preflight();
    let live =
        promote_preflight_bundle_to_live(&preflight).expect("detail preflight should promote");
    let change = BridgeChangeSummary::default().with_field_delta(BridgeFieldDelta::new(
        "identity",
        "id",
        Some("user-1"),
        Some("user-2"),
    ));
    let artifact = MilestoneFiveLiveAdapter::artifact(
        "Public Live Artifact Test",
        &[
            MilestoneFiveLiveAdapter::canonical_lane("detail-patch", &live, &change)
                .expect("canonical lane should build"),
        ],
        &[MilestoneFiveLiveAdapter::refresh_rejection_lane(
            "forbidden-refresh",
            &live,
            RefreshAdmissionClass::WidthOverflow,
        )
        .expect("detail family should reject refresh admission")],
    );

    assert_eq!(artifact.suite_name(), "Public Live Artifact Test");
    assert_eq!(artifact.canonical_lane_count(), 1);
    assert_eq!(artifact.rejection_lane_count(), 1);
    assert!(!artifact.certification_digest().is_empty());
    assert!(!artifact.coverage_digest().is_empty());
    assert_eq!(artifact.counter_snapshot().live_patch_delivery_count(), 1);
    assert_eq!(artifact.counter_snapshot().live_refresh_denial_count(), 1);
}

#[test]
fn public_live_artifact_digest_binds_counter_evidence() {
    let preflight = crate::harness::fixtures::execution_preflights::direct_runtime_preflight();
    let live =
        promote_preflight_bundle_to_live(&preflight).expect("detail preflight should promote");
    let change = BridgeChangeSummary::default().with_field_delta(BridgeFieldDelta::new(
        "identity",
        "id",
        Some("user-1"),
        Some("user-2"),
    ));
    let canonical_lane = MilestoneFiveLiveAdapter::detail_patch_lane(&live, &change)
        .expect("detail patch lane should build");
    let low_counter_rejection = LiveCertificationRejectionLane::new(
        "forbidden-refresh",
        "forbidden-refresh-escape-hatch",
        "LiveRefreshError::ForbiddenAdmissionClass(WidthOverflow)",
        LivePolicyCounters::from_refresh_error(&LiveRefreshError::ForbiddenAdmissionClass(
            RefreshAdmissionClass::WidthOverflow,
        )),
    );
    let high_counter_rejection = LiveCertificationRejectionLane::new(
        "forbidden-refresh",
        "forbidden-refresh-escape-hatch",
        "LiveRefreshError::ForbiddenAdmissionClass(WidthOverflow)",
        LivePolicyCounters::from_width_assessment(&PatchWidthAssessment {
            measured_width: 33,
            budget_limit: 32,
            resolution: PatchWidthResolution::Reject,
        }),
    );

    let low_artifact = build_milestone_five_live_artifact(
        "Counter Digest Binding",
        std::slice::from_ref(&canonical_lane),
        std::slice::from_ref(&low_counter_rejection),
    );
    let high_artifact = build_milestone_five_live_artifact(
        "Counter Digest Binding",
        std::slice::from_ref(&canonical_lane),
        std::slice::from_ref(&high_counter_rejection),
    );

    assert_ne!(
        low_artifact.certification_digest(),
        high_artifact.certification_digest()
    );
}

#[test]
fn live_execution_report_emits_digest_fields() {
    let preflight = crate::harness::fixtures::execution_preflights::direct_runtime_preflight();
    let live =
        promote_preflight_bundle_to_live(&preflight).expect("detail preflight should promote");
    let change = BridgeChangeSummary::default().with_field_delta(BridgeFieldDelta::new(
        "identity",
        "id",
        Some("user-1"),
        Some("user-2"),
    ));

    let execution = execute_live_change(&live, &change).expect("detail change should execute");

    assert!(!execution.report().query_digest().is_empty());
    assert!(!execution.report().result_digest().is_empty());
    assert!(!execution.report().delivery_digest().is_empty());
    assert!(!execution.report().replay_digest().is_empty());
}

#[test]
fn live_execution_envelope_carries_patch_and_replay_artifacts() {
    let preflight = crate::harness::fixtures::execution_preflights::direct_runtime_preflight();
    let live =
        promote_preflight_bundle_to_live(&preflight).expect("detail preflight should promote");
    let change = BridgeChangeSummary::default().with_field_delta(BridgeFieldDelta::new(
        "identity",
        "id",
        Some("user-1"),
        Some("user-2"),
    ));

    let execution = execute_live_change(&live, &change).expect("detail change should execute");

    match execution.patch_envelope().payload() {
        LivePatchPayload::Detail(patch) => {
            assert_eq!(patch.field_deltas().len(), 1);
        }
        other => panic!("expected detail payload, got {other:?}"),
    }
    assert_eq!(
        execution.patch_envelope().delivery_digest(),
        execution.replay_bundle().delivery_digest()
    );
    assert_eq!(
        execution.patch_envelope().replay_digest(),
        execution.replay_bundle().replay_digest()
    );
}
