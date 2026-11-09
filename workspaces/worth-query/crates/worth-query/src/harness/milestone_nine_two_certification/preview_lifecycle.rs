use super::bundle_projection::shipped_bundle;
use super::delivery_evidence::deliver_to_attachment;
use super::subscription_fixtures::{
    active_attachment, active_budget, admission_budget, attachment_budget, delivery_budget,
    lowering_budget, scale_slope_report, slice_budget, work_budget,
};
use super::SubscriptionLifecycleCertificationBundle;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::digest_parts;
use crate::live::LiveQueryFamily;
use crate::subscription::{
    admit_active_subscription_lane, admit_preview_subscription_isolation, admit_query_subscription,
    attach_subscription_consumer, certify_subscription_lifecycle, close_subscription_lifecycle,
    declare_query_subscription, discard_preview_subscription,
    lower_query_subscription_maintenance_delta, lower_query_subscription_to_bridge,
    measure_preview_subscription_residue, open_active_subscription_lane,
    open_query_delivery_window, prepare_subscription_activation, promote_preview_subscription,
    select_query_subscription_family, ActiveDeliveryDensityPosture,
    ActiveSubscriptionAllocationPosture, ActiveSubscriptionRuntime, LiveQueryAdmissionArtifact,
    MaintenanceDeltaWidth, PreviewResidueWidth, QuerySubscriptionConstructionSource,
    QuerySubscriptionMaintenanceDelta, QuerySubscriptionMaintenanceDeltaKind,
    SubscriptionConsumerAttachmentRequest, SubscriptionLifecycleCertificationContext,
    SubscriptionLifecycleCloseRequest, SubscriptionLifecyclePreviewCertification,
};

pub(super) fn preview_discard_lane(
) -> Result<SubscriptionLifecycleCertificationBundle, IdentityDerivationDenial> {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    )?;
    let selection = select_query_subscription_family(live.clone(), work_budget()).unwrap();
    let context =
        SubscriptionLifecycleCertificationContext::from_live_selection(&live, &selection)?;
    let declaration = declare_query_subscription(selection, slice_budget()).unwrap();
    let lowering = lower_query_subscription_to_bridge(declaration, lowering_budget()).unwrap();
    let admission = admit_query_subscription(lowering, admission_budget()).unwrap();
    let activation = prepare_subscription_activation(admission.clone());
    let scale_report = scale_slope_report(&activation, 1, 0)?;
    let mut runtime = ActiveSubscriptionRuntime::new();
    let active_admission =
        admit_active_subscription_lane(activation.clone(), active_budget()).unwrap();
    let handle = open_active_subscription_lane(&mut runtime, active_admission.clone()).unwrap();
    let attachment = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("preview-consumer", "cursor")?,
        attachment_budget(),
    )
    .unwrap();
    let window = open_query_delivery_window(&mut runtime, &attachment, delivery_budget()).unwrap();
    let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        "preview",
        MaintenanceDeltaWidth::measured(1),
    )?;
    let (delta, lowering_report, lowering_counters) =
        lower_query_subscription_maintenance_delta(delta).unwrap();
    let evidence = deliver_to_attachment(
        &mut runtime,
        attachment.clone(),
        window,
        delta.clone(),
        lowering_report.clone(),
        ActiveDeliveryDensityPosture::SparseDelta,
        1,
        1,
        1,
        0,
        0,
        1,
        ActiveSubscriptionAllocationPosture::PatchScratch,
    );
    let residue_report = measure_preview_subscription_residue(
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(1),
    );
    let isolation = admit_preview_subscription_isolation(
        &evidence.acknowledged_attachment,
        "preview-epoch",
        PreviewResidueWidth::measured(2),
    )
    .unwrap();
    let discard_closeout =
        discard_preview_subscription(isolation.clone(), residue_report.clone()).unwrap();
    let lifecycle_closeout = close_subscription_lifecycle(
        &mut runtime,
        &handle,
        SubscriptionLifecycleCloseRequest::PreviewDiscard(discard_closeout.clone()),
    )
    .unwrap();
    let mut bundle = shipped_bundle(
        certify_subscription_lifecycle(
            context,
            &admission,
            &activation,
            &scale_report,
            &active_admission,
            &handle,
            &attachment,
            evidence.delivery_batch.delivery_window_identity(),
            &delta,
            &lowering_report,
            &evidence.work_packet,
            &evidence.delivery_batch,
            &evidence.acknowledged_attachment,
            None,
            SubscriptionLifecyclePreviewCertification::Discard {
                isolation: &isolation,
                residue_report: &residue_report,
                discard_closeout: &discard_closeout,
            },
            &lifecycle_closeout,
        )
        .unwrap(),
    );
    let mut counter_evidence = bundle.counter_evidence.clone();
    counter_evidence.push(format!(
        "lowering:{}",
        lowering_counters.counter_projection().label()
    ));
    counter_evidence.extend([
        "authoritative_routing_residue:0".to_string(),
        "authoritative_checkpoint_residue:0".to_string(),
        "authoritative_replay_residue:0".to_string(),
        "authoritative_diagnostics_residue:0".to_string(),
        "authoritative_writeback_residue:0".to_string(),
    ]);
    bundle.counter_snapshot = digest_parts(&counter_evidence);
    bundle.counter_evidence = counter_evidence;
    Ok(bundle)
}

pub(super) fn preview_promotion_lane(
) -> Result<SubscriptionLifecycleCertificationBundle, IdentityDerivationDenial> {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    )?;
    let selection = select_query_subscription_family(live.clone(), work_budget()).unwrap();
    let context =
        SubscriptionLifecycleCertificationContext::from_live_selection(&live, &selection)?;
    let declaration = declare_query_subscription(selection, slice_budget()).unwrap();
    let lowering = lower_query_subscription_to_bridge(declaration, lowering_budget()).unwrap();
    let admission = admit_query_subscription(lowering, admission_budget()).unwrap();
    let activation = prepare_subscription_activation(admission.clone());
    let scale_report = scale_slope_report(&activation, 1, 0)?;
    let mut runtime = ActiveSubscriptionRuntime::new();
    let active_admission =
        admit_active_subscription_lane(activation.clone(), active_budget()).unwrap();
    let handle = open_active_subscription_lane(&mut runtime, active_admission.clone()).unwrap();
    let attachment = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("preview-consumer", "cursor")?,
        attachment_budget(),
    )
    .unwrap();
    let window = open_query_delivery_window(&mut runtime, &attachment, delivery_budget()).unwrap();
    let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        "preview-promotion",
        MaintenanceDeltaWidth::measured(1),
    )?;
    let (delta, lowering_report, lowering_counters) =
        lower_query_subscription_maintenance_delta(delta).unwrap();
    let evidence = deliver_to_attachment(
        &mut runtime,
        attachment.clone(),
        window,
        delta.clone(),
        lowering_report.clone(),
        ActiveDeliveryDensityPosture::SparseDelta,
        1,
        1,
        1,
        0,
        0,
        1,
        ActiveSubscriptionAllocationPosture::PatchScratch,
    );
    let residue_report = measure_preview_subscription_residue(
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(0),
    );
    let isolation = admit_preview_subscription_isolation(
        &evidence.acknowledged_attachment,
        "preview-epoch-promotion",
        PreviewResidueWidth::measured(1),
    )
    .unwrap();
    let authoritative = active_attachment(&mut runtime)?.0;
    let handoff = promote_preview_subscription(
        isolation.clone(),
        &residue_report,
        &authoritative,
        "authority",
    )
    .unwrap();
    let lifecycle_closeout = close_subscription_lifecycle(
        &mut runtime,
        &handle,
        SubscriptionLifecycleCloseRequest::PreviewPromotion(handoff.clone()),
    )
    .unwrap();
    let mut bundle = shipped_bundle(
        certify_subscription_lifecycle(
            context,
            &admission,
            &activation,
            &scale_report,
            &active_admission,
            &handle,
            &attachment,
            evidence.delivery_batch.delivery_window_identity(),
            &delta,
            &lowering_report,
            &evidence.work_packet,
            &evidence.delivery_batch,
            &evidence.acknowledged_attachment,
            None,
            SubscriptionLifecyclePreviewCertification::Promotion {
                isolation: &isolation,
                residue_report: &residue_report,
                promotion_handoff: &handoff,
            },
            &lifecycle_closeout,
        )
        .unwrap(),
    );
    let mut counter_evidence = bundle.counter_evidence.clone();
    counter_evidence.push(format!(
        "lowering:{}",
        lowering_counters.counter_projection().label()
    ));
    bundle.counter_snapshot = digest_parts(&counter_evidence);
    bundle.counter_evidence = counter_evidence;
    Ok(bundle)
}
