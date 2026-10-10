use super::support::*;

#[test]
fn continuation_index_plans_one_to_one_replace_without_registry_scan() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );

    let index = runtime
        .build_subscription_continuation_index(
            &active,
            vec![
                crate::facade::BridgeSubscriptionContinuationCandidateInput::one_to_one_replace(
                    "lineage:replace:entity-1:v2",
                    "subscription-locality:entity-1/profile/name",
                    "basis:entity-1-v2",
                ),
            ],
        )
        .expect("replace continuation index should build");

    assert_eq!(
        index
            .counters()
            .subscription_continuation_index_build_count(),
        1
    );
    assert_eq!(
        index
            .counters()
            .subscription_continuation_full_registry_scan_count(),
        0
    );

    let decision = runtime
        .plan_subscription_continuation(&active, &index, 0)
        .expect("replace continuation should plan");

    assert_eq!(
        decision.continuation_kind(),
        crate::facade::BridgeSubscriptionContinuationKind::OneToOneReplace
    );
    assert_eq!(decision.children().len(), 1);
}

#[test]
fn continuation_split_emits_attributable_child_records() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let index = runtime
        .build_subscription_continuation_index(
            &active,
            vec![
                crate::facade::BridgeSubscriptionContinuationCandidateInput::one_to_many_split(
                    "lineage:split:entity-1",
                    "subscription-locality:entity-1/profile/name",
                    vec![Arc::from("basis:child-a"), Arc::from("basis:child-b")],
                ),
            ],
        )
        .expect("split continuation index should build");

    let decision = runtime
        .plan_subscription_continuation(&active, &index, 0)
        .expect("split continuation should plan");

    assert_eq!(
        decision.continuation_kind(),
        crate::facade::BridgeSubscriptionContinuationKind::OneToManySplit
    );
    assert_eq!(decision.children().len(), 2);
    assert_ne!(
        decision.children()[0].digest(),
        decision.children()[1].digest()
    );
}

#[test]
fn continuation_closed_table_admits_unchanged_merge_like_and_branch_local() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let index = runtime
        .build_subscription_continuation_index(
            &active,
            vec![
                crate::facade::BridgeSubscriptionContinuationCandidateInput::unchanged(
                    "lineage:unchanged",
                    "subscription-locality:entity-1/profile/name",
                    "basis:same",
                ),
                crate::facade::BridgeSubscriptionContinuationCandidateInput::merge_like_continue(
                    "lineage:merge-like",
                    "subscription-locality:entity-1/profile/name",
                    "basis:merge",
                ),
                crate::facade::BridgeSubscriptionContinuationCandidateInput::branch_local_continue(
                    "lineage:branch-local",
                    "subscription-locality:entity-1/profile/name",
                    "basis:branch-local",
                ),
            ],
        )
        .expect("closed-table continuation candidates should index");

    assert_eq!(
        runtime
            .plan_subscription_continuation(&active, &index, 0)
            .expect("unchanged continuation should plan")
            .continuation_kind(),
        crate::facade::BridgeSubscriptionContinuationKind::Unchanged
    );
    assert_eq!(
        runtime
            .plan_subscription_continuation(&active, &index, 1)
            .expect("merge-like continuation should plan")
            .continuation_kind(),
        crate::facade::BridgeSubscriptionContinuationKind::MergeLikeContinue
    );
    assert_eq!(
        runtime
            .plan_subscription_continuation(&active, &index, 2)
            .expect("branch-local continuation should plan")
            .continuation_kind(),
        crate::facade::BridgeSubscriptionContinuationKind::BranchLocalContinue
    );
}

#[test]
fn continuation_closed_table_rejects_unsupported_ambiguous_authority_denied_and_branch_leak() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let index = runtime
        .build_subscription_continuation_index(
            &active,
            vec![
                crate::facade::BridgeSubscriptionContinuationCandidateInput::rejected_unsupported(
                    "lineage:unsupported",
                    "subscription-locality:entity-1/profile/name",
                ),
                crate::facade::BridgeSubscriptionContinuationCandidateInput::rejected_ambiguous(
                    "lineage:ambiguous",
                    "subscription-locality:entity-1/profile/name",
                ),
                crate::facade::BridgeSubscriptionContinuationCandidateInput::rejected_authority_denied(
                    "lineage:authority-denied",
                    "subscription-locality:entity-1/profile/name",
                ),
                crate::facade::BridgeSubscriptionContinuationCandidateInput::rejected_branch_leak(
                    "lineage:branch-leak",
                    "subscription-locality:entity-1/profile/name",
                ),
            ],
        )
        .expect("rejected continuation candidates are indexable proof inputs");

    let expected = [
        crate::facade::BridgeSubscriptionContinuationRejectionKind::Unsupported,
        crate::facade::BridgeSubscriptionContinuationRejectionKind::Ambiguous,
        crate::facade::BridgeSubscriptionContinuationRejectionKind::AuthorityDenied,
        crate::facade::BridgeSubscriptionContinuationRejectionKind::BranchLeak,
    ];
    for (slot, expected_kind) in expected.into_iter().enumerate() {
        let rejection = runtime
            .plan_subscription_continuation(&active, &index, slot)
            .expect_err("rejected continuation candidate must stay rejected");
        assert_eq!(rejection.rejection_kind(), expected_kind);
        assert_eq!(
            rejection
                .counters()
                .subscription_continuation_full_registry_scan_count(),
            0
        );
    }
}

#[test]
fn continuation_rejects_structurally_invalid_split_before_delivery() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let index = runtime
        .build_subscription_continuation_index(
            &active,
            vec![
                crate::facade::BridgeSubscriptionContinuationCandidateInput::one_to_many_split(
                    "lineage:split:entity-1",
                    "subscription-locality:entity-1/profile/name",
                    vec![Arc::from("basis:only-child")],
                ),
            ],
        )
        .expect("candidate shape can be indexed before decision validation");

    let rejection = runtime
        .plan_subscription_continuation(&active, &index, 0)
        .expect_err("single-child split must reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionContinuationRejectionKind::SplitRequiresMultipleChildren
    );
}

#[test]
fn continuation_rejects_active_subscription_drift() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let index = runtime
        .build_subscription_continuation_index(
            &active,
            vec![
                crate::facade::BridgeSubscriptionContinuationCandidateInput::one_to_one_replace(
                    "lineage:replace:entity-1:v2",
                    "subscription-locality:entity-1/profile/name",
                    "basis:entity-1-v2",
                ),
            ],
        )
        .expect("replace continuation index should build");
    let (other_runtime, other_active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::BoundedCoalescedWindow,
        resource_request,
    );

    let rejection = other_runtime
        .plan_subscription_continuation(&other_active, &index, 0)
        .expect_err("continuation index belongs to the original active subscription");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionContinuationRejectionKind::ActiveSubscriptionMismatch
    );
}
