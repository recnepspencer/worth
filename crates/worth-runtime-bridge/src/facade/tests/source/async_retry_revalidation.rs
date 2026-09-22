mod runtime_binding;

use super::support::{
    activation_ready_for_branch_head, activation_ready_for_snapshot,
    cancellation_retry_rejection_for_cross_declaration, preview_active_subscription_with_basis,
    request_response_revalidation_lineage,
    request_response_revalidation_rejection_for_stale_signal_generation,
    retry_lineage_after_cancellation, retry_lineage_after_timeout,
    subscription_backed_revalidation_lineage,
};
use crate::facade::{
    BridgeAsyncForwardCausalityClass, BridgeAsyncForwardCausalityRejectionKind,
    BridgeAsyncRequestSubscriptionInstance, BridgeAsyncRequestTruthViewBasis, RuntimeBridge,
};
use crate::policy::BridgeRuntimePolicy;
use worth_signal::facade::NodeId;

fn runtime() -> RuntimeBridge {
    super::runtime(BridgeRuntimePolicy::development())
}

#[test]
fn timeout_retry_lineage_maps_to_retry_after_timeout() {
    let runtime = runtime();
    let lineage = retry_lineage_after_timeout(
        &runtime,
        NodeId::new(400, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        8,
    );

    assert_eq!(
        lineage.class(),
        BridgeAsyncForwardCausalityClass::RetryAfterTimeout
    );
    assert_eq!(lineage.counters().retry_after_timeout(), 1);
    assert_eq!(
        lineage
            .prior_request()
            .basis_binding()
            .truth_view_basis()
            .digest(),
        lineage
            .newer_request()
            .basis_binding()
            .truth_view_basis()
            .digest()
    );
    assert_eq!(lineage.receipt().class(), lineage.class());
}

#[test]
fn equivalent_timeout_retry_lineages_match_across_equivalent_runtimes() {
    let runtime_a = runtime();
    let runtime_b = runtime();
    let basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let lineage_a = retry_lineage_after_timeout(&runtime_a, NodeId::new(490, 0), basis.clone(), 8);
    let lineage_b = retry_lineage_after_timeout(&runtime_b, NodeId::new(490, 0), basis, 8);

    assert_eq!(lineage_a.digest(), lineage_b.digest());
    assert_eq!(lineage_a.receipt().digest(), lineage_b.receipt().digest());
}

#[test]
fn cancellation_retry_lineage_maps_to_retry_after_cancellation() {
    let runtime = runtime();
    let lineage = retry_lineage_after_cancellation(
        &runtime,
        NodeId::new(401, 0),
        BridgeAsyncRequestTruthViewBasis::branch_head(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-branch-head"),
        ),
    );

    assert_eq!(
        lineage.class(),
        BridgeAsyncForwardCausalityClass::RetryAfterCancellation
    );
    assert_eq!(lineage.counters().retry_after_cancellation(), 1);
    assert_eq!(
        lineage
            .prior_request()
            .basis_binding()
            .truth_view_basis()
            .digest(),
        lineage
            .newer_request()
            .basis_binding()
            .truth_view_basis()
            .digest()
    );
}

#[test]
fn cancellation_retry_rejects_cross_declaration_newer_request() {
    let runtime = runtime();
    let rejection = cancellation_retry_rejection_for_cross_declaration(
        &runtime,
        NodeId::new(401, 0),
        NodeId::new(499, 0),
        BridgeAsyncRequestTruthViewBasis::branch_head(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-branch-head"),
        ),
    );

    assert_eq!(
        rejection.kind(),
        BridgeAsyncForwardCausalityRejectionKind::PriorAndNewerDeclarationMismatch
    );
}

#[test]
fn truth_basis_drift_revalidation_maps_to_truth_basis_class() {
    let runtime = runtime();
    let lineage = request_response_revalidation_lineage(
        &runtime,
        NodeId::new(402, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        ),
    );

    assert_eq!(
        lineage.class(),
        BridgeAsyncForwardCausalityClass::RevalidationAfterTruthBasisDrift
    );
    assert_eq!(lineage.counters().revalidation_after_truth_basis_drift(), 1);
    assert_ne!(
        lineage
            .prior_request()
            .basis_binding()
            .truth_view_basis()
            .digest(),
        lineage
            .newer_request()
            .basis_binding()
            .truth_view_basis()
            .digest()
    );
}

#[test]
fn preview_basis_drift_revalidation_outranks_generic_subscription_drift() {
    let runtime = runtime();
    let prior_preview = preview_active_subscription_with_basis(
        &runtime,
        "prior",
        crate::truth_identity_fixtures::truth_branch_fixture("truth-preview"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-preview-a"),
    );
    let current_preview = preview_active_subscription_with_basis(
        &runtime,
        "current",
        crate::truth_identity_fixtures::truth_branch_fixture("truth-preview"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-preview-b"),
    );
    let lineage = request_response_revalidation_lineage(
        &runtime,
        NodeId::new(403, 0),
        BridgeAsyncRequestTruthViewBasis::preview(&prior_preview),
        BridgeAsyncRequestTruthViewBasis::preview(&current_preview),
    );

    assert_eq!(
        lineage.class(),
        BridgeAsyncForwardCausalityClass::RevalidationAfterPreviewBasisDrift
    );
    assert_eq!(
        lineage.counters().revalidation_after_preview_basis_drift(),
        1
    );
}

#[test]
fn subscription_instance_drift_revalidation_stays_distinct() {
    let runtime = runtime();
    let prior_subscription =
        BridgeAsyncRequestSubscriptionInstance::authoritative(&activation_ready_for_snapshot(
            &runtime,
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ));
    let current_subscription =
        BridgeAsyncRequestSubscriptionInstance::authoritative(&activation_ready_for_branch_head(
            &runtime,
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        ));
    let truth_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let lineage = subscription_backed_revalidation_lineage(
        &runtime,
        NodeId::new(404, 0),
        truth_basis.clone(),
        prior_subscription,
        truth_basis,
        current_subscription,
    );

    assert_eq!(
        lineage.class(),
        BridgeAsyncForwardCausalityClass::RevalidationAfterSubscriptionInstanceDrift
    );
    assert_eq!(
        lineage
            .counters()
            .revalidation_after_subscription_instance_drift(),
        1
    );
    assert_eq!(
        lineage
            .prior_request()
            .basis_binding()
            .truth_view_basis()
            .digest(),
        lineage
            .newer_request()
            .basis_binding()
            .truth_view_basis()
            .digest()
    );
    assert_ne!(
        lineage
            .prior_request()
            .subscription_instance()
            .expect("prior subscription-backed lineage should retain instance")
            .digest(),
        lineage
            .newer_request()
            .subscription_instance()
            .expect("newer subscription-backed lineage should retain instance")
            .digest()
    );
}

#[test]
fn stale_signal_generation_revalidation_rejects_typed() {
    let runtime = runtime();
    let rejection = request_response_revalidation_rejection_for_stale_signal_generation(
        &runtime,
        NodeId::new(405, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
    );

    assert_eq!(
        rejection.kind(),
        BridgeAsyncForwardCausalityRejectionKind::StaleSignalGenerationRejected
    );
}
