use super::active_lifecycle::lifecycle_lane;
use super::delivery_evidence::deliver_to_attachment;
use super::subscription_fixtures::{active_budget, attachment_budget, delivery_budget};
use super::SubscriptionLifecycleCertificationBundle;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::digest_parts;
use crate::live::LiveQueryFamily;
use crate::subscription::{
    admit_active_subscription_lane, attach_subscription_consumer, join_active_subscription_lane,
    lower_query_subscription_maintenance_delta, open_active_subscription_lane,
    open_query_delivery_window, ActiveDeliveryDensityPosture, ActiveSubscriptionAllocationPosture,
    ActiveSubscriptionRuntime, MaintenanceDeltaWidth, QuerySubscriptionConstructionSource,
    QuerySubscriptionMaintenanceDelta, QuerySubscriptionMaintenanceDeltaKind,
    SubscriptionConsumerAttachmentRequest,
};

pub(super) fn sharing_lane(
    first_consumer: &str,
    second_consumer: &str,
) -> Result<SubscriptionLifecycleCertificationBundle, IdentityDerivationDenial> {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let activation = super::subscription_fixtures::activation_for(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    )?;
    let admission = admit_active_subscription_lane(activation.clone(), active_budget()).unwrap();
    let handle = open_active_subscription_lane(&mut runtime, admission).unwrap();
    let open_counter_digest = runtime.counters().counter_projection().label().to_string();
    let join_admission = admit_active_subscription_lane(activation, active_budget()).unwrap();
    let joined = join_active_subscription_lane(&mut runtime, &handle, join_admission).unwrap();
    let join_counter_digest = runtime.counters().counter_projection().label().to_string();
    let first = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted(first_consumer, "cursor-a")?,
        attachment_budget(),
    )
    .unwrap();
    let first_attachment_counter_digest =
        runtime.counters().counter_projection().label().to_string();
    let second = attach_subscription_consumer(
        &mut runtime,
        &joined,
        SubscriptionConsumerAttachmentRequest::admitted(second_consumer, "cursor-b")?,
        attachment_budget(),
    )
    .unwrap();
    let second_attachment_counter_digest =
        runtime.counters().counter_projection().label().to_string();
    let first_window = open_query_delivery_window(&mut runtime, &first, delivery_budget()).unwrap();
    let first_window_counter_digest = runtime.counters().counter_projection().label().to_string();
    let second_window =
        open_query_delivery_window(&mut runtime, &second, delivery_budget()).unwrap();
    let second_window_counter_digest = runtime.counters().counter_projection().label().to_string();
    let first_delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        first.lane_digest().clone(),
        "shared-manager",
        MaintenanceDeltaWidth::measured(1),
    )?;
    let (first_delta, first_lowering, first_lowering_counters) =
        lower_query_subscription_maintenance_delta(first_delta).unwrap();
    let second_delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        second.lane_digest().clone(),
        "shared-manager",
        MaintenanceDeltaWidth::measured(1),
    )?;
    let (second_delta, second_lowering, second_lowering_counters) =
        lower_query_subscription_maintenance_delta(second_delta).unwrap();
    let first_digest = first.attachment_projection().label().to_string();
    let second_digest = second.attachment_projection().label().to_string();
    let first_evidence = deliver_to_attachment(
        &mut runtime,
        first,
        first_window,
        first_delta,
        first_lowering,
        ActiveDeliveryDensityPosture::SparseDelta,
        1,
        2,
        1,
        0,
        0,
        1,
        ActiveSubscriptionAllocationPosture::PatchScratch,
    );
    let second_evidence = deliver_to_attachment(
        &mut runtime,
        second,
        second_window,
        second_delta,
        second_lowering,
        ActiveDeliveryDensityPosture::SparseDelta,
        1,
        2,
        1,
        0,
        0,
        1,
        ActiveSubscriptionAllocationPosture::PatchScratch,
    );
    let mut bundle = lifecycle_lane(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        "shared-manager",
        1,
        0,
    )?;
    bundle.active_lane_digest = handle.lane_projection().label().to_string();
    bundle.active_lane_handle_digest = digest_parts(&[
        format!("opened:{}", handle.lane_projection().label()),
        format!("joined:{}", joined.lane_projection().label()),
        format!("same_lane:{}", handle.lane_digest() == joined.lane_digest()),
    ]);
    bundle.consumer_attachment_digest = digest_parts(&[first_digest, second_digest]);
    bundle.subscription_performance_receipt_digest = digest_parts(&[
        first_evidence.performance_receipt_digest,
        second_evidence.performance_receipt_digest,
    ]);
    bundle.acknowledgement_frontier_digest = digest_parts(&[
        first_evidence.acknowledgement_frontier_digest,
        second_evidence.acknowledgement_frontier_digest,
    ]);
    bundle.delivery_window_digest = digest_parts(&[
        first_evidence.delivery_window_digest,
        second_evidence.delivery_window_digest,
    ]);
    bundle.maintenance_delta_digest = digest_parts(&[
        first_evidence.maintenance_delta_digest,
        second_evidence.maintenance_delta_digest,
    ]);
    bundle.active_delivery_work_packet_digest = digest_parts(&[
        first_evidence.active_delivery_work_packet_digest,
        second_evidence.active_delivery_work_packet_digest,
    ]);
    bundle.delivery_batch_digest = digest_parts(&[
        first_evidence.delivery_batch_digest,
        second_evidence.delivery_batch_digest,
    ]);
    bundle.patch_group_digest = digest_parts(&[
        first_evidence.patch_group_digest,
        second_evidence.patch_group_digest,
    ]);
    bundle.delivery_receipt_digest = digest_parts(&[
        first_evidence.delivery_receipt_digest,
        second_evidence.delivery_receipt_digest,
    ]);
    let sharing_counter_evidence = vec![
        format!("open:{open_counter_digest}"),
        format!("join:{join_counter_digest}"),
        format!("first_attach:{first_attachment_counter_digest}"),
        format!("second_attach:{second_attachment_counter_digest}"),
        format!("first_window:{first_window_counter_digest}"),
        format!("second_window:{second_window_counter_digest}"),
        format!(
            "first_lowering:{}",
            first_lowering_counters.counter_projection().label()
        ),
        format!(
            "second_lowering:{}",
            second_lowering_counters.counter_projection().label()
        ),
        format!("first_packet:{}", first_evidence.work_packet_counter_digest),
        format!(
            "second_packet:{}",
            second_evidence.work_packet_counter_digest
        ),
        format!("first_batch:{}", first_evidence.batch_counter_digest),
        format!("second_batch:{}", second_evidence.batch_counter_digest),
        format!("first_ack:{}", first_evidence.ack_counter_digest),
        format!("second_ack:{}", second_evidence.ack_counter_digest),
        bundle.counter_snapshot,
        format!("shared_lane:{}", joined.lane_projection().label()),
        "same_lane:true".to_string(),
        "consumer_local_delivery_count:2".to_string(),
    ];
    bundle.counter_snapshot = digest_parts(&sharing_counter_evidence);
    bundle.counter_evidence = sharing_counter_evidence;
    Ok(bundle)
}
