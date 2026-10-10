use std::sync::Arc;

use super::super::support::*;
use crate::facade::{
    BridgeRuntimePolicy, BridgeSubscriptionContinuationCandidateInput,
    BridgeSubscriptionContinuationKind, BridgeSubscriptionContinuationRejectionKind,
    BridgeSubscriptionDeliveryDensityPosture,
};

#[test]
fn bridge_harness_subscription_suite_32_continuation_identity_evolution_is_typed() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bridge = runtime(BridgeRuntimePolicy::development());
    let detail = detail_subscription(&bridge);
    let active = active_subscription_for(
        &bridge,
        &detail,
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        1,
        resource_request,
    );
    let candidates = vec![
        BridgeSubscriptionContinuationCandidateInput::one_to_one_replace(
            "lineage:suite-32:replace",
            "subscription-locality:entity-1/profile/name",
            "basis:suite-32:replace",
        ),
        BridgeSubscriptionContinuationCandidateInput::one_to_many_split(
            "lineage:suite-32:split",
            "subscription-locality:entity-1/profile/name",
            vec![
                Arc::from("basis:suite-32:child-a"),
                Arc::from("basis:suite-32:child-b"),
            ],
        ),
        BridgeSubscriptionContinuationCandidateInput::merge_like_continue(
            "lineage:suite-32:merge-like",
            "subscription-locality:entity-1/profile/name",
            "basis:suite-32:merge",
        ),
        BridgeSubscriptionContinuationCandidateInput::branch_local_continue(
            "lineage:suite-32:branch-local",
            "subscription-locality:entity-1/profile/name",
            "basis:suite-32:branch-local",
        ),
        BridgeSubscriptionContinuationCandidateInput::rejected_ambiguous(
            "lineage:suite-32:ambiguous",
            "subscription-locality:entity-1/profile/name",
        ),
    ];
    let index = bridge
        .build_subscription_continuation_index(&active, candidates.clone())
        .expect("continuation index should build from locality candidates");

    let replace = bridge
        .plan_subscription_continuation(&active, &index, 0)
        .expect("replace continuation should plan");
    let split = bridge
        .plan_subscription_continuation(&active, &index, 1)
        .expect("split continuation should plan");
    let merge_like = bridge
        .plan_subscription_continuation(&active, &index, 2)
        .expect("merge-like continuation should plan");
    let branch_local = bridge
        .plan_subscription_continuation(&active, &index, 3)
        .expect("branch-local continuation should plan");
    let ambiguous = bridge
        .plan_subscription_continuation(&active, &index, 4)
        .expect_err("ambiguous continuation must reject typed");

    assert_eq!(
        replace.continuation_kind(),
        BridgeSubscriptionContinuationKind::OneToOneReplace
    );
    assert_eq!(replace.children().len(), 1);
    assert_eq!(
        split.continuation_kind(),
        BridgeSubscriptionContinuationKind::OneToManySplit
    );
    assert_eq!(split.children().len(), 2);
    assert_ne!(split.children()[0].digest(), split.children()[1].digest());
    assert_eq!(
        merge_like.continuation_kind(),
        BridgeSubscriptionContinuationKind::MergeLikeContinue
    );
    assert_eq!(
        branch_local.continuation_kind(),
        BridgeSubscriptionContinuationKind::BranchLocalContinue
    );
    let unrelated_collection = collection_subscription(&bridge);
    let unrelated_active = active_subscription_for(
        &bridge,
        &unrelated_collection,
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        1,
        resource_request,
    );
    let unrelated_rejection = bridge
        .plan_subscription_continuation(&unrelated_active, &index, 3)
        .expect_err("branch-local continuation index must not leak to unrelated active");
    assert_eq!(
        unrelated_rejection.rejection_kind(),
        BridgeSubscriptionContinuationRejectionKind::ActiveSubscriptionMismatch
    );
    assert_eq!(
        ambiguous.rejection_kind(),
        BridgeSubscriptionContinuationRejectionKind::Ambiguous
    );
    assert_eq!(
        index
            .counters()
            .subscription_continuation_full_registry_scan_count(),
        0
    );
    assert_eq!(
        ambiguous
            .counters()
            .subscription_continuation_full_registry_scan_count(),
        0
    );

    let restart_runtime = runtime(BridgeRuntimePolicy::development());
    let restart_detail = detail_subscription(&restart_runtime);
    let restart_active = active_subscription_for(
        &restart_runtime,
        &restart_detail,
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        1,
        resource_request,
    );
    let restart_index = restart_runtime
        .build_subscription_continuation_index(&restart_active, candidates)
        .expect("restart continuation index should rebuild from retained candidate basis");
    let restart_replace = restart_runtime
        .plan_subscription_continuation(&restart_active, &restart_index, 0)
        .expect("restart replace continuation should plan");
    let restart_split = restart_runtime
        .plan_subscription_continuation(&restart_active, &restart_index, 1)
        .expect("restart split continuation should plan");
    let restart_ambiguous = restart_runtime
        .plan_subscription_continuation(&restart_active, &restart_index, 4)
        .expect_err("restart ambiguous continuation must reject typed");

    assert_eq!(active.digest(), restart_active.digest());
    assert_eq!(index.digest(), restart_index.digest());
    assert_eq!(replace.digest(), restart_replace.digest());
    assert_eq!(split.digest(), restart_split.digest());
    assert_eq!(ambiguous.digest(), restart_ambiguous.digest());
}
