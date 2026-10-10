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
    let admitted_preview = runtime
        .admit_preview_session(
            crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(format!(
                "preview-session:{suffix}"
            )),
            subscription_preview_declaration_for_truth(
                suffix,
                truth_branch_identity,
                snapshot_identity,
            ),
        )
        .expect("preview session should admit");
    let (active_preview, execution_record) =
        runtime.activate_preview_session(admitted_preview, 3, 1, 2);
    runtime
        .admit_subscription_preview_basis(&active_preview, &execution_record)
        .expect("preview basis should admit")
}

pub(crate) fn preview_promotion_detail_subscription(
    suffix: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::facade::RuntimeBridge,
    crate::facade::BridgePreviewActiveSubscription,
    crate::facade::BridgePreviewPromotionRecord,
    crate::facade::BridgeSubscriptionActivationReady,
) {
    let (runtime, ready) = activation_ready_detail_subscription(resource_request);
    let admitted_preview = runtime
        .admit_preview_session(
            crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(format!(
                "preview-session:{suffix}"
            )),
            subscription_preview_declaration(suffix),
        )
        .expect("preview session should admit");
    let (active_preview_session, execution_record) =
        runtime.activate_preview_session(admitted_preview, 3, 1, 2);
    let preview_basis = runtime
        .admit_subscription_preview_basis(&active_preview_session, &execution_record)
        .expect("preview basis should admit");
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
    let proof = active_preview_session.promotion_admissibility_proof();
    let (_promoted_session, promotion_record) = runtime
        .promote_preview_session(active_preview_session, &execution_record, &proof)
        .expect("speculation promotion should succeed");
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

pub(crate) fn preview_active_collection_subscription(
    suffix: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::facade::RuntimeBridge,
    crate::facade::BridgePreviewActiveSubscription,
) {
    preview_active_subscription_from_ready(
        activation_ready_collection_subscription(resource_request),
        suffix,
    )
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
    let admitted_preview = runtime
        .admit_preview_session(
            crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(format!(
                "preview-session:{suffix}"
            )),
            subscription_preview_declaration(suffix),
        )
        .expect("preview session should admit");
    let (active_preview, execution_record) =
        runtime.activate_preview_session(admitted_preview, 3, 1, 2);
    let preview_basis = runtime
        .admit_subscription_preview_basis(&active_preview, &execution_record)
        .expect("preview basis should admit");
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

pub(crate) fn subscription_preview_declaration(
    suffix: &str,
) -> crate::facade::BridgePreviewSessionDeclaration {
    subscription_preview_declaration_for_truth(
        suffix,
        crate::truth_identity_fixtures::truth_branch_fixture(format!("truth-branch:{suffix}")),
        crate::truth_identity_fixtures::truth_snapshot_fixture(format!("snapshot:{suffix}")),
    )
}

pub(crate) fn subscription_preview_declaration_for_truth(
    suffix: &str,
    truth_branch_identity: TruthBranchIdentity,
    snapshot_identity: crate::facade::TruthSnapshotIdentity,
) -> crate::facade::BridgePreviewSessionDeclaration {
    let selector_branch_identity = truth_branch_identity.clone();
    crate::facade::BridgePreviewSessionDeclaration::new(
        crate::facade::BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(format!(
            "preview-declaration:{suffix}"
        )),
        crate::facade::BridgeRequestKind::Preview,
        crate::facade::BridgeSpeculativeBranchBinding::new(
            crate::facade::BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(format!(
                "preview-binding:{suffix}"
            )),
            truth_branch_identity,
            crate::facade::BridgeSignalBranchIdentity::admit_bridge_owned(format!(
                "signal-branch:{suffix}"
            )),
        ),
        crate::facade::BridgePreviewSessionBasis::new(
            crate::facade::BridgeTruthViewSelector::branch_snapshot(
                selector_branch_identity,
                snapshot_identity,
            ),
            crate::facade::BridgeSourceCapabilitySet::new(vec![
                crate::facade::BridgeSourceCapability::SnapshotRead,
                crate::facade::BridgeSourceCapability::BranchRead,
            ]),
            crate::facade::BridgePreviewRetainedArtifactSchema::PreviewLifecycleArtifactsV1,
        ),
    )
}

pub(crate) fn zero_preview_residue_inputs(
    runtime: &crate::facade::RuntimeBridge,
    preview_active: &crate::facade::BridgePreviewActiveSubscription,
) -> Vec<crate::facade::BridgeSubscriptionPreviewResidueArtifactInput> {
    preview_work_trace(runtime, preview_active, "preview-residue").zero_residue_inputs()
}

pub(crate) fn preview_residue_inputs_with_count(
    runtime: &crate::facade::RuntimeBridge,
    preview_active: &crate::facade::BridgePreviewActiveSubscription,
    nonzero_category: crate::facade::BridgeSubscriptionPreviewResidueCategory,
    residue_count: usize,
) -> Vec<crate::facade::BridgeSubscriptionPreviewResidueArtifactInput> {
    let work_trace = preview_work_trace(runtime, preview_active, "preview-residue");
    work_trace
        .zero_residue_inputs()
        .into_iter()
        .map(|input| {
            if input.category() == nonzero_category {
                crate::facade::BridgeSubscriptionPreviewResidueArtifactInput::from_preview_work_trace(
                    nonzero_category,
                    residue_count,
                    &work_trace,
                )
            } else {
                input
            }
        })
        .collect()
}
