use super::*;

#[test]
fn dense_refresh_denial_blocks_work_packet_before_batch_exists() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let attachment = attached_consumer(&mut runtime, LiveQueryFamily::Detail, None);
    let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        "employee.name",
        MaintenanceDeltaWidth::measured(1),
    )
    .expect("fixture digest must fit");
    let (delta, lowering_report, _) = lower_query_subscription_maintenance_delta(delta).unwrap();

    let error = build_active_delivery_work_packet(
        &mut runtime,
        &attachment,
        delta,
        lowering_report,
        ActiveDeliveryDensityPosture::DenseRefreshDenied,
        ActiveDeliveryAffectedLaneWidth::measured(1),
        ActiveDeliveryAffectedAttachmentWidth::measured(1),
        PatchGroupWidth::measured(1),
        ActiveDeliveryContinuationWidth::measured(0),
        ActiveDeliveryPreviewResidueWidth::measured(0),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::PatchScratch,
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QueryDeliveryDenialKind::DenseRefreshDenied
    );
    assert_eq!(
        error
            .counters()
            .active_delivery_density_dense_denial_count(),
        1
    );
}

#[test]
fn delivery_window_rejects_wrong_phase_or_denied_allocation_posture() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let attachment = attached_consumer(&mut runtime, LiveQueryFamily::Detail, None);

    let wrong_phase = open_query_delivery_window(
        &mut runtime,
        &attachment,
        QueryDeliveryWindowBudget::admitted(
            DeliveryWindowWidth::measured(2),
            PatchGroupWidth::measured(1),
            MaintenanceDeltaWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::LifecycleArena,
            DeliveryBackpressurePolicy::RetainWithinWindow,
        ),
    )
    .unwrap_err();
    assert_eq!(
        wrong_phase.denial_kind(),
        &QueryDeliveryDenialKind::AllocationPostureForbidden
    );
    assert_eq!(wrong_phase.counters().heap_allocation_denial_count(), 1);

    let denied = open_query_delivery_window(
        &mut runtime,
        &attachment,
        QueryDeliveryWindowBudget::admitted(
            DeliveryWindowWidth::measured(2),
            PatchGroupWidth::measured(1),
            MaintenanceDeltaWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::HeapAllocationDenied,
            DeliveryBackpressurePolicy::RetainWithinWindow,
        ),
    )
    .unwrap_err();
    assert_eq!(
        denied.denial_kind(),
        &QueryDeliveryDenialKind::AllocationPostureForbidden
    );
    assert_eq!(denied.counters().heap_allocation_denial_count(), 1);
}

#[test]
fn delivery_window_heap_allocation_debt_is_explicit() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let attachment = attached_consumer(&mut runtime, LiveQueryFamily::Detail, None);

    let window = open_query_delivery_window(
        &mut runtime,
        &attachment,
        QueryDeliveryWindowBudget::admitted(
            DeliveryWindowWidth::measured(2),
            PatchGroupWidth::measured(1),
            MaintenanceDeltaWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::HeapAllocationDebtExplicit,
            DeliveryBackpressurePolicy::RetainWithinWindow,
        ),
    )
    .unwrap();

    assert_eq!(
        window.allocation_posture(),
        ActiveSubscriptionAllocationPosture::HeapAllocationDebtExplicit
    );
    assert_eq!(runtime.counters().heap_allocation_debt_count(), 1);
}

#[test]
fn work_packet_rejects_wrong_phase_allocation_posture() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let attachment = attached_consumer(&mut runtime, LiveQueryFamily::Detail, None);
    let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        "employee.name",
        MaintenanceDeltaWidth::measured(1),
    )
    .expect("fixture digest must fit");
    let (delta, lowering_report, _) = lower_query_subscription_maintenance_delta(delta).unwrap();

    let error = build_active_delivery_work_packet(
        &mut runtime,
        &attachment,
        delta,
        lowering_report,
        ActiveDeliveryDensityPosture::SparseDelta,
        ActiveDeliveryAffectedLaneWidth::measured(1),
        ActiveDeliveryAffectedAttachmentWidth::measured(1),
        PatchGroupWidth::measured(1),
        ActiveDeliveryContinuationWidth::measured(0),
        ActiveDeliveryPreviewResidueWidth::measured(0),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QueryDeliveryDenialKind::AllocationPostureForbidden
    );
    assert_eq!(error.counters().heap_allocation_denial_count(), 1);
}

#[test]
fn performance_receipt_digest_binds_allocation_posture() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let attachment = attached_consumer(&mut runtime, LiveQueryFamily::Detail, None);

    let patch_delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        "employee.name",
        MaintenanceDeltaWidth::measured(1),
    )
    .expect("fixture digest must fit");
    let (patch_delta, patch_lowering, _) =
        lower_query_subscription_maintenance_delta(patch_delta).unwrap();
    let patch_packet = build_active_delivery_work_packet(
        &mut runtime,
        &attachment,
        patch_delta,
        patch_lowering,
        ActiveDeliveryDensityPosture::SparseDelta,
        ActiveDeliveryAffectedLaneWidth::measured(1),
        ActiveDeliveryAffectedAttachmentWidth::measured(1),
        PatchGroupWidth::measured(1),
        ActiveDeliveryContinuationWidth::measured(0),
        ActiveDeliveryPreviewResidueWidth::measured(0),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::PatchScratch,
    )
    .unwrap();

    let debt_delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        "employee.name",
        MaintenanceDeltaWidth::measured(1),
    )
    .expect("fixture digest must fit");
    let (debt_delta, debt_lowering, _) =
        lower_query_subscription_maintenance_delta(debt_delta).unwrap();
    let debt_packet = build_active_delivery_work_packet(
        &mut runtime,
        &attachment,
        debt_delta,
        debt_lowering,
        ActiveDeliveryDensityPosture::SparseDelta,
        ActiveDeliveryAffectedLaneWidth::measured(1),
        ActiveDeliveryAffectedAttachmentWidth::measured(1),
        PatchGroupWidth::measured(1),
        ActiveDeliveryContinuationWidth::measured(0),
        ActiveDeliveryPreviewResidueWidth::measured(0),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::HeapAllocationDebtExplicit,
    )
    .unwrap();

    assert_eq!(
        debt_packet.allocation_posture(),
        ActiveSubscriptionAllocationPosture::HeapAllocationDebtExplicit
    );
    assert_eq!(runtime.counters().heap_allocation_debt_count(), 1);
    assert_ne!(
        patch_packet
            .performance_receipt()
            .performance_receipt_projection()
            .label(),
        debt_packet
            .performance_receipt()
            .performance_receipt_projection()
            .label()
    );
    assert_ne!(
        patch_packet.work_packet_projection().label(),
        debt_packet.work_packet_projection().label()
    );
}
