use super::super::support::*;

#[test]
fn subscription_preview_basis_admits_from_active_preview_session() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted_preview = runtime
        .admit_preview_session(
            crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(
                "preview-session:subscription-basis",
            ),
            subscription_preview_declaration("subscription-basis"),
        )
        .expect("preview session should admit");
    let (active_preview, execution_record) =
        runtime.activate_preview_session(admitted_preview, 2, 1, 1);

    let preview_basis = runtime
        .admit_subscription_preview_basis(&active_preview, &execution_record)
        .expect("active preview session should admit subscription preview basis");

    assert_eq!(
        preview_basis.preview_session_identity(),
        active_preview.session_identity()
    );
    assert_eq!(
        preview_basis.preview_execution_record_identity(),
        execution_record.record_identity()
    );
    assert_eq!(
        preview_basis.preview_declaration_digest(),
        active_preview.declaration().digest()
    );
    assert_eq!(
        preview_basis.preview_lifecycle_state_kind(),
        crate::facade::BridgePreviewLifecycleStateKind::Active
    );
    assert_eq!(
        preview_basis.branch_binding_digest(),
        active_preview
            .declaration()
            .declaration()
            .branch_binding()
            .digest()
    );
    assert_eq!(
        preview_basis.parent_truth_view_basis_digest(),
        active_preview
            .declaration()
            .declaration()
            .truth_view_basis_digest()
    );
    assert_eq!(
        preview_basis
            .counters()
            .subscription_preview_basis_admission_count(),
        1
    );
    assert_eq!(
        preview_basis
            .counters()
            .subscription_callback_identity_scan_count(),
        0
    );
}

#[test]
fn subscription_preview_basis_rejects_mismatched_execution_record() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted_preview = runtime
        .admit_preview_session(
            crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(
                "preview-session:subscription-a",
            ),
            subscription_preview_declaration("subscription-a"),
        )
        .expect("preview session should admit");
    let (active_preview, _) = runtime.activate_preview_session(admitted_preview, 2, 1, 1);
    let other_admitted_preview = runtime
        .admit_preview_session(
            crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(
                "preview-session:subscription-b",
            ),
            subscription_preview_declaration("subscription-b"),
        )
        .expect("other preview session should admit");
    let (_, other_execution_record) =
        runtime.activate_preview_session(other_admitted_preview, 2, 1, 1);

    let rejection = runtime
        .admit_subscription_preview_basis(&active_preview, &other_execution_record)
        .expect_err("mismatched preview execution record must reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionPreviewBasisRejectionKind::PreviewExecutionRecordMismatch
    );
    assert_eq!(
        rejection.rejection_context().preview_session_identity(),
        active_preview.session_identity()
    );
    assert_eq!(
        rejection
            .rejection_context()
            .supplied_execution_record_identity(),
        Some(other_execution_record.record_identity())
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_preview_basis_rejection_count(),
        1
    );
}

#[test]
fn preview_subscription_activation_binds_preview_basis_and_activation_ready_proof() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, ready) = activation_ready_detail_subscription(resource_request);
    let admitted_preview = runtime
        .admit_preview_session(
            crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(
                "preview-session:subscription-active",
            ),
            subscription_preview_declaration("subscription-active"),
        )
        .expect("preview session should admit");
    let (active_preview, execution_record) =
        runtime.activate_preview_session(admitted_preview, 3, 1, 2);
    let preview_basis = runtime
        .admit_subscription_preview_basis(&active_preview, &execution_record)
        .expect("preview basis should admit");
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
        preview_active.preview_execution_record_identity(),
        execution_record.record_identity()
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
