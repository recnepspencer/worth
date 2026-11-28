use super::super::support::*;

#[test]
fn preview_subscription_activation_binds_preview_basis_and_activation_ready_proof() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, ready) = activation_ready_detail_subscription(resource_request);
    let active_preview = runtime.begin_preview(
        crate::facade::BridgePreviewSessionIdentity::from_stable_name("preview-subscription"),
        preview_branches(
            "signal",
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        ),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot"),
    );
    let preview_basis = runtime.admit_subscription_preview_basis(&active_preview);
    let preview_basis_identity = preview_basis.preview_basis_identity().clone();
    let preview_residue_scope_identity = preview_basis.preview_residue_scope_identity().clone();
    let admitted_subscription_identity = ready.admitted().admitted_subscription_identity().clone();
    let cost_profile = runtime
        .admit_subscription_delivery_cost_profile(
            BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            4,
            4,
            1,
        )
        .expect("cost profile should admit");
    let consumer = canonical_consumer_contract(&runtime);

    let preview_active = runtime.activate_preview_subscription_delivery(
        ready,
        preview_basis,
        cost_profile,
        consumer,
    );

    assert_eq!(
        preview_active.preview_basis_identity(),
        &preview_basis_identity
    );
    assert_eq!(
        preview_active.preview_residue_scope_identity(),
        &preview_residue_scope_identity
    );
    assert_eq!(
        preview_active.preview_session_identity(),
        active_preview.session_identity()
    );
    assert_eq!(
        preview_active.admitted_subscription_identity(),
        &admitted_subscription_identity
    );
    assert_eq!(
        preview_active
            .counters()
            .subscription_preview_activation_count(),
        1
    );
    assert_eq!(
        preview_active
            .counters()
            .subscription_callback_identity_scan_count(),
        0
    );
}
