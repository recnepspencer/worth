use super::support::*;

#[test]
fn pricing_shock_discard_closes_preview_during_main_churn() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let scenario = generated_pricing_scenario();
    let discard = capture_pricing_discard_bundle(resource_request);

    assert_eq!(
        discard.live_main_snapshot,
        discard.post_discard_main_snapshot
    );
    assert_eq!(
        discard.speculative_rubber_cost_cents,
        scenario.speculative_rubber_cost
    );
    assert_eq!(
        discard.post_discard_main_steel_cost_cents,
        scenario.live_main_steel_cost
    );
    assert!(discard.session_closed);
}

#[test]
fn pricing_shock_finish_closes_preview_after_comparing_advanced_main() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let scenario = generated_pricing_scenario();
    let promotion = capture_pricing_promotion_bundle(resource_request);

    assert_ne!(promotion.main_snapshot, promotion.speculative_snapshot);
    assert_eq!(
        promotion.main_rubber_cost_cents,
        scenario.interleaved_main_rubber_cost
    );
    assert_eq!(
        promotion.speculative_rubber_cost_cents,
        scenario.speculative_rubber_cost
    );
    assert!(promotion.session_closed);
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
    assert!(fanout.second_source_commit.relational_commit_id().is_some());
    assert_eq!(fanout.second_snapshot, fanout.branch_snapshot);
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
    let serial = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let truth = RelationalPricing::new(execution, 100, 40, 56);
    let writeback = capture_pricing_writeback_bundle(BridgeRuntimePolicy::development(), &truth);

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
    assert_eq!(
        writeback.committed_effect_intent_digest,
        writeback.retried_effect_intent_digest
    );
    assert!(writeback.shared_authoritative_artifact);
    assert_eq!(writeback.authority_commit_count, 1);
}
