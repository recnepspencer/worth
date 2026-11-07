use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

pub(super) fn active_budget(
    lookup_width: u64,
    fanout_width: u64,
    allocation_posture: ActiveSubscriptionAllocationPosture,
) -> ActiveSubscriptionWorkBudget {
    ActiveSubscriptionWorkBudget::admitted(
        ActiveRegistryLookupWidth::measured(lookup_width),
        ActiveFanoutWidth::measured(fanout_width),
        ActiveAllocationScopeWidth::measured(1),
        allocation_posture,
    )
}

pub(super) fn attachment_budget(
    fanout_width: u64,
    policy: DeliveryBackpressurePolicy,
) -> SubscriptionConsumerAttachmentBudget {
    SubscriptionConsumerAttachmentBudget::admitted(
        ActiveFanoutWidth::measured(fanout_width),
        ConsumerDeliveryPacingWidth::measured(1),
        ActiveAllocationScopeWidth::measured(1),
        policy,
    )
}

pub(super) fn delivery_budget(
    patch_group_width: u64,
    maintenance_delta_width: u64,
) -> QueryDeliveryWindowBudget {
    QueryDeliveryWindowBudget::admitted(
        DeliveryWindowWidth::measured(2),
        PatchGroupWidth::measured(patch_group_width),
        MaintenanceDeltaWidth::measured(maintenance_delta_width),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    )
}

pub(super) fn activation_for(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
) -> SubscriptionActivationInput {
    let input = LiveQueryAdmissionArtifact::for_test(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    activation_from_input(input, roomy_lowering_budget())
}

pub(super) fn activation_for_future(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    future_selection: QuerySubscriptionFutureSelection,
) -> SubscriptionActivationInput {
    let input = LiveQueryAdmissionArtifact::for_test_with_future_selection(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::FacadeLive,
        future_selection,
    )
    .expect("fixture digest must fit");
    activation_from_input(input, roomy_lowering_budget())
}

pub(super) fn activation_for_with_context(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    basis_posture: QuerySubscriptionBasisPosture,
    future_selection: QuerySubscriptionFutureSelection,
) -> SubscriptionActivationInput {
    let input = LiveQueryAdmissionArtifact::for_test_with_context(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::FacadeLive,
        basis_posture.clone(),
        future_selection,
        Some("policy".to_string()),
        Some("tenant".to_string()),
        Some("relationship-proof".to_string()),
        QuerySubscriptionRelationshipProofPosture::Admitted,
    )
    .expect("fixture digest must fit");
    let lowering_budget = if basis_posture == QuerySubscriptionBasisPosture::PreviewScoped {
        roomy_lowering_budget().with_preview_basis_support()
    } else {
        roomy_lowering_budget()
    };
    activation_from_input(input, lowering_budget)
}

pub(super) fn active_lane_for(
    runtime: &mut ActiveSubscriptionRuntime,
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    lookup_width: u64,
    fanout_width: u64,
) -> ActiveSubscriptionLaneHandle {
    let activation = activation_for(live_family, view_family);
    let admission = admit_active_subscription_lane(
        activation,
        active_budget(
            lookup_width,
            fanout_width,
            ActiveSubscriptionAllocationPolicy::LifecycleArena,
        ),
    )
    .unwrap();
    open_active_subscription_lane(runtime, admission).unwrap()
}

pub(super) fn active_lane_for_with_context(
    runtime: &mut ActiveSubscriptionRuntime,
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    basis_posture: QuerySubscriptionBasisPosture,
    future_selection: QuerySubscriptionFutureSelection,
    lookup_width: u64,
    fanout_width: u64,
) -> ActiveSubscriptionLaneHandle {
    let activation =
        activation_for_with_context(live_family, view_family, basis_posture, future_selection);
    let admission = admit_active_subscription_lane(
        activation,
        active_budget(
            lookup_width,
            fanout_width,
            ActiveSubscriptionAllocationPolicy::LifecycleArena,
        ),
    )
    .unwrap();
    open_active_subscription_lane(runtime, admission).unwrap()
}

pub(super) fn attached_consumer(
    runtime: &mut ActiveSubscriptionRuntime,
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    consumer: &str,
    lookup_width: u64,
    fanout_width: u64,
    policy: DeliveryBackpressurePolicy,
) -> (ActiveSubscriptionLaneHandle, SubscriptionConsumerAttachment) {
    let handle = active_lane_for(
        runtime,
        live_family,
        view_family,
        lookup_width,
        fanout_width,
    );
    let attachment = attach_subscription_consumer(
        runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted(consumer, "cursor")
            .expect("fixture digest must fit"),
        attachment_budget(fanout_width, policy),
    )
    .unwrap();
    (handle, attachment)
}

pub(super) fn attached_future_consumer(
    runtime: &mut ActiveSubscriptionRuntime,
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    consumer: &str,
    future_selection: QuerySubscriptionFutureSelection,
    lookup_width: u64,
    fanout_width: u64,
    policy: DeliveryBackpressurePolicy,
) -> SubscriptionConsumerAttachment {
    let activation = activation_for_future(live_family, view_family, future_selection);
    let admission = admit_active_subscription_lane(
        activation,
        active_budget(
            lookup_width,
            fanout_width,
            ActiveSubscriptionAllocationPolicy::LifecycleArena,
        ),
    )
    .unwrap();
    let handle = open_active_subscription_lane(runtime, admission).unwrap();
    attach_subscription_consumer(
        runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted(consumer, "cursor")
            .expect("fixture digest must fit"),
        attachment_budget(fanout_width, policy),
    )
    .unwrap()
}

pub(super) fn emitted_receipt(
    runtime: &mut ActiveSubscriptionRuntime,
    attachment: &SubscriptionConsumerAttachment,
    scope: &str,
    patch_group_width: u64,
    maintenance_delta_width: u64,
) -> QueryDeliveryBatchReceipt {
    let window = open_query_delivery_window(
        runtime,
        attachment,
        delivery_budget(patch_group_width, maintenance_delta_width),
    )
    .unwrap();
    let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        scope,
        MaintenanceDeltaWidth::measured(1),
    )
    .expect("fixture digest must fit");
    let (delta, lowering_report, _) = lower_query_subscription_maintenance_delta(delta).unwrap();
    let packet = build_active_delivery_work_packet(
        runtime,
        attachment,
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
    emit_query_delivery_batch(runtime, window, packet)
        .unwrap()
        .receipt()
        .clone()
}

pub(super) fn zero_authoritative_residue() -> PreviewSubscriptionResidueReport {
    measure_preview_subscription_residue(
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(1),
    )
}

fn activation_from_input(
    input: LiveQueryAdmissionArtifact,
    lowering_budget: QuerySubscriptionBridgeLoweringBudget,
) -> SubscriptionActivationInput {
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
    let declaration = declare_query_subscription(selection, roomy_slice_budget()).unwrap();
    let lowering = lower_query_subscription_to_bridge(declaration, lowering_budget).unwrap();
    let admission = admit_query_subscription(
        lowering,
        QuerySubscriptionAdmissionBudget::admitted(1, 1, 1, 1, 1),
    )
    .unwrap();
    prepare_subscription_activation(admission)
}
