use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

use super::activation_world::{activation_for, roomy_admission_budget};

pub(super) fn preview_promotion_certification_artifacts() -> LifecycleCertificationArtifacts {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None,
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
            11,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            110,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            1100,
            &activation,
        )
        .expect("fixture digest must fit"),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let preview_admission = admit_active_subscription_lane(
        activation.clone(),
        ActiveSubscriptionWorkBudget::admitted(
            ActiveRegistryLookupWidth::measured(1),
            ActiveFanoutWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::LifecycleArena,
        ),
    )
    .unwrap();
    let handle = open_active_subscription_lane(&mut runtime, preview_admission.clone()).unwrap();
    let attachment = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("preview-dashboard", "cursor")
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
            PatchGroupWidth::measured(1),
            MaintenanceDeltaWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
            DeliveryBackpressurePolicy::RetainWithinWindow,
        ),
    )
    .unwrap();
    let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        "preview-field",
        MaintenanceDeltaWidth::measured(1),
    )
    .expect("fixture digest must fit");
    let (delta, lowering_report, _) = lower_query_subscription_maintenance_delta(delta).unwrap();
    let work_packet = build_active_delivery_work_packet(
        &mut runtime,
        &attachment,
        delta.clone(),
        lowering_report.clone(),
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
    let delivery_batch =
        emit_query_delivery_batch(&mut runtime, window, work_packet.clone()).unwrap();
    let acknowledged_attachment = advance_subscription_acknowledgement(
        &mut runtime,
        attachment.clone(),
        delivery_batch.receipt().clone(),
    )
    .unwrap();
    let authoritative_activation = activation_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let authoritative_admission = admit_active_subscription_lane(
        authoritative_activation,
        ActiveSubscriptionWorkBudget::admitted(
            ActiveRegistryLookupWidth::measured(1),
            ActiveFanoutWidth::measured(1),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::LifecycleArena,
        ),
    )
    .unwrap();
    let authoritative_handle =
        open_active_subscription_lane(&mut runtime, authoritative_admission).unwrap();
    let isolation = admit_preview_subscription_isolation(
        &acknowledged_attachment,
        "preview-certification-promotion",
        PreviewResidueWidth::measured(1),
    )
    .unwrap();
    let residue_report = measure_preview_subscription_residue(
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(0),
    );
    let promotion_handoff = promote_preview_subscription(
        isolation.clone(),
        &residue_report,
        &authoritative_handle,
        "authority",
    )
    .unwrap();
    let closeout = close_subscription_lifecycle(
        &mut runtime,
        &handle,
        SubscriptionLifecycleCloseRequest::PreviewPromotion(promotion_handoff.clone()),
    )
    .unwrap();
    LifecycleCertificationArtifacts {
        context,
        admission,
        activation,
        scale_report,
        active_admission: preview_admission,
        handle,
        attachment,
        delta,
        lowering_report,
        work_packet,
        delivery_batch,
        acknowledged_attachment,
        continuation_report: None,
        preview: SubscriptionLifecyclePreviewCertificationArtifacts::Promotion {
            isolation,
            residue_report,
            promotion_handoff,
        },
        closeout,
    }
}
