use super::support::*;

#[test]
fn pricing_shock_discard_stays_zero_residue_under_interleaved_main_churn() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let scenario = generated_pricing_scenario();
    let discard = capture_pricing_discard_bundle(resource_request);

    assert_eq!(
        discard.live_main_snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main-live")
    );
    assert_eq!(
        discard.speculative_rubber_cost_cents,
        scenario.speculative_rubber_cost
    );
    assert_eq!(
        discard.post_discard_main_snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main-live")
    );
    assert_eq!(
        discard.post_discard_main_steel_cost_cents,
        scenario.live_main_steel_cost
    );
    assert_eq!(
        discard.lifecycle_state,
        BridgePreviewLifecycleStateKind::Discarded
    );
    assert_eq!(discard.discard_record_count, 1);
    assert_eq!(discard.promotion_record_count, 0);
    assert_eq!(
        discard.replay_outcome,
        BridgePreviewLifecycleStateKind::Discarded
    );
    assert!(discard.has_discard_record);
    assert!(!discard.has_promotion_record);
}

#[test]
fn pricing_shock_promotion_stays_distinct_from_interleaved_main_truth() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let scenario = generated_pricing_scenario();
    let promotion = capture_pricing_promotion_bundle(resource_request);

    assert_eq!(
        promotion.main_snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main-interleaved")
    );
    assert_eq!(
        promotion.speculative_snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-shock")
    );
    assert_eq!(
        promotion.main_rubber_cost_cents,
        scenario.interleaved_main_rubber_cost
    );
    assert_eq!(
        promotion.speculative_rubber_cost_cents,
        scenario.speculative_rubber_cost
    );
    assert_eq!(
        promotion.lifecycle_state,
        BridgePreviewLifecycleStateKind::Promoted
    );
    assert_eq!(
        promotion.promotion_session_identity,
        BridgePreviewSessionIdentity::admit_bridge_owned("pricing:preview-promote-churn")
    );
    assert!(promotion
        .authoritative_commit_boundary_digest
        .starts_with("preview-promotion-commit-boundary:sha256:"));
    assert!(promotion
        .authoritative_artifact_digest
        .starts_with("preview-promotion-authoritative-artifact:sha256:"));
    assert_eq!(
        promotion.replay_outcome,
        BridgePreviewLifecycleStateKind::Promoted
    );
    assert!(promotion.has_promotion_explanation);
}

#[test]
fn pricing_shock_live_graph_shared_input_fans_out_across_one_hundred_products() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let scenario = generated_pricing_scenario();
    let fanout = capture_pricing_fanout_bundle(resource_request);

    assert_eq!(fanout.total_deliveries, 2);
    assert_eq!(fanout.first_delivery_target_count, 100);
    assert_eq!(fanout.second_delivery_target_count, 100);
    assert_eq!(
        fanout.second_source_commit,
        crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-fanout-b")
    );
    assert_eq!(
        fanout.second_snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-fanout-b")
    );
    assert_eq!(
        fanout.branch_snapshot,
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-fanout-b")
    );
    assert_eq!(
        fanout.branch_steel_cost_cents,
        scenario.fanout_second_steel_cost
    );
    assert_eq!(fanout.retained_target_count, 100);
    assert_eq!(fanout.first_target, "price:product-000");
    assert_eq!(fanout.last_target, "price:product-099");
}

#[test]
fn pricing_shock_writeback_lane_preserves_authority_boundary_and_noop_classification() {
    let writeback = capture_pricing_writeback_bundle(BridgeRuntimePolicy::development());

    assert_eq!(
        writeback.family_kind,
        BridgeWritebackFamilyKind::ProjectedStateDiff
    );
    assert_eq!(
        writeback.strategy_class,
        BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation
    );
    assert_eq!(
        writeback.commit_outcome_class,
        crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit
    );
    assert_eq!(
        writeback.noop_outcome_class,
        crate::facade::BridgeWritebackOutcomeClass::CanonicalNoop
    );
    assert_ne!(
        writeback.commit_replay_semantic_digest,
        writeback.noop_replay_semantic_digest
    );
    assert!(writeback.shared_authoritative_artifact);
    assert_eq!(writeback.authority_commit_count, 1);
    assert_eq!(writeback.execution_request_count, 1);
    assert_eq!(writeback.execution_commit_count, 1);
    assert_eq!(writeback.execution_noop_count, 1);
}
