use super::*;

pub(crate) fn preview_active_detail_subscription(
    suffix: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::facade::RuntimeBridge,
    crate::facade::BridgePreviewActiveSubscription,
) {
    preview_active_subscription_from_ready(
        activation_ready_detail_subscription(resource_request),
        suffix,
    )
}

pub(crate) fn admitted_preview_basis_for_truth(
    runtime: &crate::facade::RuntimeBridge,
    suffix: &str,
    truth_branch_identity: TruthBranchIdentity,
    snapshot_identity: crate::facade::TruthSnapshotIdentity,
) -> crate::facade::BridgeSubscriptionPreviewBasisBinding {
    let active_preview = runtime.begin_preview(
        crate::preview::BridgePreviewSessionIdentity::admit_bridge_owned(format!(
            "preview-session:{suffix}"
        )),
        preview_branches(suffix, truth_branch_identity),
        snapshot_identity,
    );
    runtime.admit_subscription_preview_basis(&active_preview)
}

pub(crate) fn preview_promotion_detail_subscription(
    suffix: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::facade::RuntimeBridge,
    crate::facade::BridgePreviewActiveSubscription,
    crate::facade::FinishedBridgePreviewSession,
    crate::facade::BridgeSubscriptionActivationReady,
) {
    let (runtime, ready) = activation_ready_detail_subscription(resource_request);
    let active_preview = runtime.begin_preview(
        crate::preview::BridgePreviewSessionIdentity::admit_bridge_owned(format!(
            "preview-session:{suffix}"
        )),
        preview_branches(
            suffix,
            crate::truth_identity_fixtures::truth_branch_fixture(format!("truth-branch:{suffix}")),
        ),
        crate::truth_identity_fixtures::truth_snapshot_fixture(format!("snapshot:{suffix}")),
    );
    let preview_basis = runtime.admit_subscription_preview_basis(&active_preview);
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
    let promotion_record = active_preview.finish();
    let promoted_ready =
        activation_ready_detail_subscription_in_runtime(&runtime, resource_request);

    (runtime, preview_active, promotion_record, promoted_ready)
}

pub(crate) fn preview_work_trace(
    runtime: &crate::facade::RuntimeBridge,
    preview_active: &crate::facade::BridgePreviewActiveSubscription,
    _suffix: &str,
) -> BridgeSubscriptionPreviewWorkTrace {
    runtime
        .record_preview_subscription_work(
            preview_active,
            vec![
                BridgeSubscriptionPreviewWorkInput::routing(preview_active),
                BridgeSubscriptionPreviewWorkInput::delivery(preview_active),
                BridgeSubscriptionPreviewWorkInput::diagnostics(preview_active),
                BridgeSubscriptionPreviewWorkInput::continuation(preview_active),
            ],
        )
        .expect("preview work trace should bind all preview work categories")
}

pub(crate) fn preview_active_subscription_from_ready(
    (runtime, ready): (
        crate::facade::RuntimeBridge,
        crate::facade::BridgeSubscriptionActivationReady,
    ),
    suffix: &str,
) -> (
    crate::facade::RuntimeBridge,
    crate::facade::BridgePreviewActiveSubscription,
) {
    let active_preview = runtime.begin_preview(
        crate::preview::BridgePreviewSessionIdentity::admit_bridge_owned(format!(
            "preview-session:{suffix}"
        )),
        preview_branches(
            suffix,
            crate::truth_identity_fixtures::truth_branch_fixture(format!("truth-branch:{suffix}")),
        ),
        crate::truth_identity_fixtures::truth_snapshot_fixture(format!("snapshot:{suffix}")),
    );
    let preview_basis = runtime.admit_subscription_preview_basis(&active_preview);
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
    (runtime, preview_active)
}

pub(crate) fn preview_branches(
    suffix: &str,
    branch: TruthBranchIdentity,
) -> crate::facade::BridgePreviewBranchBinding {
    crate::facade::BridgePreviewBranchBinding::new(
        branch,
        crate::facade::BridgeSignalBranchIdentity::from_stable_name(suffix),
    )
}
