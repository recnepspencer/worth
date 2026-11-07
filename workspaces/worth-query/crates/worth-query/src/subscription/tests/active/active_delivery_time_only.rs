use super::runtime_harness::{attached_consumer, delivery_budget};
use super::*;
use crate::live::LiveQueryFamily;

fn emit_time_only(
    runtime: &mut ActiveSubscriptionRuntime,
    attachment: &SubscriptionConsumerAttachment,
    cause_kind: QuerySubscriptionDeliveryCauseKind,
    evidence_digest: &str,
) -> QueryDeliveryBatch {
    let window = open_query_delivery_window(runtime, attachment, delivery_budget(1, 1)).unwrap();
    let cause = QuerySubscriptionDeliveryCause::time_only(cause_kind, evidence_digest)
        .expect("fixture digest must fit");
    emit_query_time_only_delivery_batch(runtime, window, cause).unwrap()
}

#[test]
fn freshness_only_delivery_is_canonical_without_relational_patch() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::OrderedCollection,
        None,
        "consumer-time-only",
        1,
        1,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );

    let batch = emit_time_only(
        &mut runtime,
        &attachment,
        QuerySubscriptionDeliveryCauseKind::FreshnessOnly,
        "time:freshness-window",
    );

    assert_eq!(
        batch.delivery_cause_kind(),
        QuerySubscriptionDeliveryCauseKind::FreshnessOnly
    );
    assert!(!batch.has_relational_patch());
    assert_eq!(
        batch.patch_group().kind(),
        QueryPatchGroupKind::TimeOnlyDeliveryGroup
    );
    assert_eq!(batch.patch_group().width(), 0);
}

#[test]
fn time_only_and_relational_delivery_batches_keep_distinct_cause_identity() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::OrderedCollection,
        None,
        "consumer-time-only",
        1,
        1,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );

    let time_only = emit_time_only(
        &mut runtime,
        &attachment,
        QuerySubscriptionDeliveryCauseKind::WindowEntry,
        "time:window-entry",
    );
    let acknowledged =
        advance_subscription_acknowledgement(&mut runtime, attachment, time_only.receipt().clone())
            .unwrap();

    let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::CollectionMembershipDelta,
        acknowledged.lane_digest().clone(),
        "task:membership",
        MaintenanceDeltaWidth::measured(1),
    )
    .expect("fixture digest must fit");
    let (delta, lowering_report, _) = lower_query_subscription_maintenance_delta(delta).unwrap();
    let window =
        open_query_delivery_window(&mut runtime, &acknowledged, delivery_budget(1, 1)).unwrap();
    let packet = build_active_delivery_work_packet(
        &mut runtime,
        &acknowledged,
        delta,
        lowering_report,
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
    let relational = emit_query_delivery_batch(&mut runtime, window, packet).unwrap();

    assert_ne!(
        time_only
            .delivery_cause()
            .delivery_cause_projection()
            .label(),
        relational
            .delivery_cause()
            .delivery_cause_projection()
            .label()
    );
    assert_ne!(
        time_only.patch_group().patch_group_projection().label(),
        relational.patch_group().patch_group_projection().label()
    );
    assert!(!time_only.has_relational_patch());
    assert!(relational.has_relational_patch());
}
