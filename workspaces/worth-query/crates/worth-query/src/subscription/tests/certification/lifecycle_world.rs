use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

use super::activation_world::roomy_admission_budget;

pub(super) fn active_lifecycle_certification_for(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    delta_kind: QuerySubscriptionMaintenanceDeltaKind,
    patch_width: u64,
    continuation_width: u64,
) -> LifecycleCertificationArtifacts {
    let live = LiveQueryAdmissionArtifact::for_test(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(live.clone(), roomy_budget()).unwrap();
    let context = SubscriptionLifecycleCertificationContext::from_live_selection(&live, &selection)
        .expect("fixture digest must fit");
    let declaration = declare_query_subscription(selection, roomy_slice_budget()).unwrap();
    let lowering =
        lower_query_subscription_to_bridge(declaration, roomy_lowering_budget()).unwrap();
    let admission = admit_query_subscription(lowering, roomy_admission_budget()).unwrap();
    let activation = prepare_subscription_activation(admission.clone());
    let scale_report = certify_query_subscription_scale_slope(
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Small,
            10 + patch_width + continuation_width,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            (10 + patch_width + continuation_width) * 10,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            (10 + patch_width + continuation_width) * 100,
            &activation,
        )
        .expect("fixture digest must fit"),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let active_admission = admit_active_subscription_lane(
        activation.clone(),
        ActiveSubscriptionWorkBudget::admitted(
            ActiveRegistryLookupWidth::measured(1),
            ActiveFanoutWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::LifecycleArena,
        ),
    )
    .unwrap();
    let handle = open_active_subscription_lane(&mut runtime, active_admission.clone()).unwrap();
    let attachment = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("employee-dashboard", "cursor")
            .expect("fixture digest must fit"),
        SubscriptionConsumerAttachmentBudget::admitted(
            ActiveFanoutWidth::measured(1),
            ConsumerDeliveryPacingWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            DeliveryBackpressurePolicy::RetainWithinWindow,
        ),
    )
    .unwrap();
    let window = open_query_delivery_window(
        &mut runtime,
        &attachment,
        QueryDeliveryWindowBudget::admitted(
            DeliveryWindowWidth::measured(3),
            PatchGroupWidth::measured(patch_width.max(1)),
            MaintenanceDeltaWidth::measured(patch_width.max(1)),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
            DeliveryBackpressurePolicy::RetainWithinWindow,
        ),
    )
    .unwrap();
    let (delta, continuation_report) = if continuation_width > 0 {
        let evidence = admit_subscription_continuation_evidence(
            attachment.lane_digest().clone(),
            SubscriptionContinuationClass::IdentityRemap,
            continuation_test_identity("employee:old"),
            continuation_test_identity("employee:new"),
            continuation_test_identity("basis:current"),
            continuation_test_identity("identity-authority"),
            ContinuationRemapWidth::measured(continuation_width),
        )
        .unwrap();
        let (continued_window, report) =
            apply_active_subscription_continuation(&mut runtime, window, evidence).unwrap();
        let (delta, _) = lower_subscription_continuation_report(&report);
        let (delta, lowering_report, _) =
            lower_query_subscription_maintenance_delta(delta).unwrap();
        let work_packet = build_active_delivery_work_packet(
            &mut runtime,
            &attachment,
            delta.clone(),
            lowering_report.clone(),
            ActiveDeliveryDensityPosture::SparseDelta,
            ActiveDeliveryAffectedLaneWidth::measured(1),
            ActiveDeliveryAffectedAttachmentWidth::measured(1),
            PatchGroupWidth::measured(patch_width.max(1)),
            ActiveDeliveryContinuationWidth::measured(continuation_width),
            ActiveDeliveryPreviewResidueWidth::measured(0),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::PatchScratch,
        )
        .unwrap();
        let delivery_batch =
            emit_query_delivery_batch(&mut runtime, continued_window, work_packet.clone()).unwrap();
        let acknowledged_attachment = advance_subscription_acknowledgement(
            &mut runtime,
            attachment.clone(),
            delivery_batch.receipt().clone(),
        )
        .unwrap();
        let closeout = close_subscription_lifecycle(
            &mut runtime,
            &handle,
            SubscriptionLifecycleCloseRequest::TerminateConsumer(acknowledged_attachment.clone()),
        )
        .unwrap();
        return LifecycleCertificationArtifacts {
            context,
            admission,
            activation,
            scale_report,
            active_admission,
            handle,
            attachment,
            delta,
            lowering_report,
            work_packet,
            delivery_batch,
            acknowledged_attachment,
            continuation_report: Some(report),
            preview: SubscriptionLifecyclePreviewCertificationArtifacts::None,
            closeout,
        };
    } else {
        (
            QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
                delta_kind,
                attachment.lane_digest().clone(),
                "affected-scope",
                MaintenanceDeltaWidth::measured(patch_width.max(1)),
            ),
            None,
        )
    };
    let (delta, lowering_report, _) =
        lower_query_subscription_maintenance_delta(delta.expect("fixture digest must fit"))
            .unwrap();
    let work_packet = build_active_delivery_work_packet(
        &mut runtime,
        &attachment,
        delta.clone(),
        lowering_report.clone(),
        ActiveDeliveryDensityPosture::SparseDelta,
        ActiveDeliveryAffectedLaneWidth::measured(1),
        ActiveDeliveryAffectedAttachmentWidth::measured(1),
        PatchGroupWidth::measured(patch_width.max(1)),
        ActiveDeliveryContinuationWidth::measured(0),
        ActiveDeliveryPreviewResidueWidth::measured(0),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::PatchScratch,
    )
    .unwrap();
    let delivery_batch =
        emit_query_delivery_batch(&mut runtime, window, work_packet.clone()).unwrap();
    let acknowledged_attachment = advance_subscription_acknowledgement(
        &mut runtime,
        attachment.clone(),
        delivery_batch.receipt().clone(),
    )
    .unwrap();
    let closeout = close_subscription_lifecycle(
        &mut runtime,
        &handle,
        SubscriptionLifecycleCloseRequest::TerminateConsumer(acknowledged_attachment.clone()),
    )
    .unwrap();

    LifecycleCertificationArtifacts {
        context,
        admission,
        activation,
        scale_report,
        active_admission,
        handle,
        attachment,
        delta,
        lowering_report,
        work_packet,
        delivery_batch,
        acknowledged_attachment,
        continuation_report,
        preview: SubscriptionLifecyclePreviewCertificationArtifacts::None,
        closeout,
    }
}
