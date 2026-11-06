use super::bundle_projection::shipped_bundle;
use super::delivery_evidence::deliver_to_attachment;
use super::subscription_fixtures::{
    active_budget, admission_budget, attachment_budget, delivery_budget, lowering_budget,
    scale_axis_evidence, scale_slope_report, slice_budget, work_budget,
};
use super::SubscriptionLifecycleCertificationBundle;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::digest_parts;
use crate::live::LiveQueryFamily;
use crate::subscription::{
    admit_active_subscription_lane, admit_query_subscription,
    admit_subscription_continuation_evidence, apply_active_subscription_continuation,
    attach_subscription_consumer, certify_subscription_lifecycle, close_subscription_lifecycle,
    declare_query_subscription, lower_query_subscription_maintenance_delta,
    lower_query_subscription_to_bridge, lower_subscription_continuation_report,
    open_active_subscription_lane, open_query_delivery_window, prepare_subscription_activation,
    select_query_subscription_family, ActiveDeliveryDensityPosture,
    ActiveSubscriptionAllocationPosture, ActiveSubscriptionRuntime, ContinuationRemapWidth,
    LiveQueryAdmissionArtifact, MaintenanceDeltaWidth, QuerySubscriptionConstructionSource,
    QuerySubscriptionMaintenanceDelta, QuerySubscriptionMaintenanceDeltaKind,
    SubscriptionActivationInput, SubscriptionConsumerAttachmentRequest,
    SubscriptionContinuationClass, SubscriptionLifecycleCertificationContext,
    SubscriptionLifecycleCloseRequest, SubscriptionLifecyclePreviewCertification,
};

fn continuation_harness_identity(
    label: &str,
) -> Result<crate::WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(crate::WorthQueryEvidenceIdentity::compose(
        crate::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
    )
    .field_shape(
        crate::WorthQueryEvidenceTag::new("identity_family"),
        "subscription_continuation_harness_identity_v1",
    )
    .field_shape(crate::WorthQueryEvidenceTag::new("label"), label)
    .seal()?)
}

pub(super) fn lifecycle_lane(
    live_family: LiveQueryFamily,
    view_family: Option<crate::view_shape_live::LiveViewShapeFamily>,
    delta_kind: QuerySubscriptionMaintenanceDeltaKind,
    affected_scope: &str,
    patch_width: u64,
    continuation_width: u64,
) -> SubscriptionLifecycleCertificationBundle {
    lifecycle_lane_with_delivery_profile(
        live_family,
        view_family,
        delta_kind,
        affected_scope,
        patch_width,
        continuation_width,
        ActiveDeliveryDensityPosture::SparseDelta,
        1,
        ActiveSubscriptionAllocationPosture::PatchScratch,
    )
}

pub(super) fn lifecycle_lane_with_posture() -> SubscriptionLifecycleCertificationBundle {
    lifecycle_lane_with_delivery_profile(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        "manager_id",
        1,
        0,
        ActiveDeliveryDensityPosture::BurstCoalesced,
        2,
        ActiveSubscriptionAllocationPosture::HeapAllocationDebtExplicit,
    )
}

fn lifecycle_lane_with_delivery_profile(
    live_family: LiveQueryFamily,
    view_family: Option<crate::view_shape_live::LiveViewShapeFamily>,
    delta_kind: QuerySubscriptionMaintenanceDeltaKind,
    affected_scope: &str,
    patch_width: u64,
    continuation_width: u64,
    density_posture: ActiveDeliveryDensityPosture,
    allocation_scope_width: u64,
    allocation_posture: ActiveSubscriptionAllocationPosture,
) -> SubscriptionLifecycleCertificationBundle {
    let live = LiveQueryAdmissionArtifact::for_test(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::FacadeLive,
    );
    let selection = select_query_subscription_family(live.clone(), work_budget()).unwrap();
    let context = SubscriptionLifecycleCertificationContext::from_live_selection(&live, &selection);
    let declaration = declare_query_subscription(selection, slice_budget()).unwrap();
    let lowering = lower_query_subscription_to_bridge(declaration, lowering_budget()).unwrap();
    let admission_artifact = admit_query_subscription(lowering, admission_budget()).unwrap();
    let activation = prepare_subscription_activation(admission_artifact.clone());
    let scale_report = scale_slope_report(&activation, patch_width, continuation_width);
    let mut runtime = ActiveSubscriptionRuntime::new();
    let active_admission =
        admit_active_subscription_lane(activation.clone(), active_budget()).unwrap();
    let handle = open_active_subscription_lane(&mut runtime, active_admission.clone()).unwrap();
    let attachment = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("employee-dashboard", "cursor"),
        attachment_budget(),
    )
    .unwrap();
    let window = open_query_delivery_window(&mut runtime, &attachment, delivery_budget()).unwrap();
    let (window, delta, continuation_report, extra_counters) = prepare_maintenance_delta(
        &mut runtime,
        window,
        &attachment,
        delta_kind,
        affected_scope,
        patch_width,
        continuation_width,
    );
    let (delta, lowering_report, lowering_counters) =
        lower_query_subscription_maintenance_delta(delta).unwrap();
    finish_delivery_lane(
        &mut runtime,
        context,
        admission_artifact,
        activation,
        scale_report,
        active_admission,
        handle,
        attachment,
        window,
        delta,
        lowering_report,
        patch_width,
        continuation_width,
        continuation_report,
        &[
            extra_counters,
            vec![lowering_counters.counter_projection().label().to_string()],
            scale_axis_evidence(patch_width, continuation_width),
        ]
        .concat(),
        density_posture,
        allocation_scope_width,
        allocation_posture,
    )
}

fn prepare_maintenance_delta(
    runtime: &mut ActiveSubscriptionRuntime,
    window: crate::subscription::QueryDeliveryWindow,
    attachment: &crate::subscription::SubscriptionConsumerAttachment,
    delta_kind: QuerySubscriptionMaintenanceDeltaKind,
    affected_scope: &str,
    patch_width: u64,
    continuation_width: u64,
) -> (
    crate::subscription::QueryDeliveryWindow,
    QuerySubscriptionMaintenanceDelta,
    Option<crate::subscription::SubscriptionContinuationReport>,
    Vec<String>,
) {
    if continuation_width > 0 {
        let evidence = admit_subscription_continuation_evidence(
            attachment.lane_digest().clone(),
            SubscriptionContinuationClass::IdentityRemap,
            continuation_harness_identity("employee:old"),
            continuation_harness_identity("employee:new"),
            continuation_harness_identity("basis:current"),
            continuation_harness_identity("identity-authority"),
            ContinuationRemapWidth::measured(continuation_width),
        )
        .unwrap();
        let (window, report) =
            apply_active_subscription_continuation(runtime, window, evidence).unwrap();
        let (delta, counters) = lower_subscription_continuation_report(&report);
        (
            window,
            delta,
            Some(report),
            vec![counters.counter_projection().label().to_string()],
        )
    } else {
        let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
            delta_kind,
            attachment.lane_digest().clone(),
            affected_scope,
            MaintenanceDeltaWidth::measured(patch_width),
        );
        (window, delta, None, Vec::new())
    }
}

fn finish_delivery_lane(
    runtime: &mut ActiveSubscriptionRuntime,
    context: crate::subscription::SubscriptionLifecycleCertificationContext,
    admission_artifact: crate::subscription::QuerySubscriptionAdmissionArtifact,
    activation: SubscriptionActivationInput,
    scale_report: crate::subscription::QuerySubscriptionScaleSlopeReport,
    active_admission: crate::subscription::ActiveSubscriptionLaneAdmission,
    active_lane_handle: crate::subscription::ActiveSubscriptionLaneHandle,
    attachment: crate::subscription::SubscriptionConsumerAttachment,
    window: crate::subscription::QueryDeliveryWindow,
    delta: QuerySubscriptionMaintenanceDelta,
    lowering_report: crate::subscription::QueryMaintenanceDeltaLoweringReport,
    patch_width: u64,
    continuation_width: u64,
    continuation_report: Option<crate::subscription::SubscriptionContinuationReport>,
    extra_counter_digests: &[String],
    density_posture: ActiveDeliveryDensityPosture,
    allocation_scope_width: u64,
    allocation_posture: ActiveSubscriptionAllocationPosture,
) -> SubscriptionLifecycleCertificationBundle {
    let evidence = deliver_to_attachment(
        runtime,
        attachment.clone(),
        window,
        delta.clone(),
        lowering_report.clone(),
        density_posture,
        1,
        1,
        patch_width,
        continuation_width,
        0,
        allocation_scope_width,
        allocation_posture,
    );
    let lifecycle_closeout = close_subscription_lifecycle(
        runtime,
        &active_lane_handle,
        SubscriptionLifecycleCloseRequest::TerminateConsumer(
            evidence.acknowledged_attachment.clone(),
        ),
    )
    .unwrap();
    let shipped = certify_subscription_lifecycle(
        context,
        &admission_artifact,
        &activation,
        &scale_report,
        &active_admission,
        &active_lane_handle,
        &attachment,
        evidence.delivery_batch.delivery_window_identity(),
        &delta,
        &lowering_report,
        &evidence.work_packet,
        &evidence.delivery_batch,
        &evidence.acknowledged_attachment,
        continuation_report.as_ref(),
        SubscriptionLifecyclePreviewCertification::None,
        &lifecycle_closeout,
    )
    .unwrap();
    let mut bundle = shipped_bundle(shipped);
    let mut counter_parts = bundle.counter_evidence.clone();
    counter_parts.push(format!("packet:{}", evidence.work_packet_counter_digest));
    counter_parts.push(format!("batch:{}", evidence.batch_counter_digest));
    counter_parts.push(format!("ack:{}", evidence.ack_counter_digest));
    counter_parts.push(format!(
        "closeout:{}",
        lifecycle_closeout.counters().counter_projection().label()
    ));
    counter_parts.extend(extra_counter_digests.iter().cloned());
    bundle.counter_snapshot = digest_parts(&counter_parts);
    bundle.counter_evidence = counter_parts;
    bundle
}
