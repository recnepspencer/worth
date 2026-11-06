use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::live::LiveQueryFamily;
use crate::subscription::{
    admit_active_subscription_lane, admit_query_subscription, attach_subscription_consumer,
    certify_query_subscription_scale_slope, declare_query_subscription,
    lower_query_subscription_to_bridge, open_active_subscription_lane,
    prepare_subscription_activation, select_query_subscription_family, ActiveAllocationScopeWidth,
    ActiveFanoutWidth, ActiveRegistryLookupWidth, ActiveSubscriptionAllocationPolicy,
    ActiveSubscriptionWorkBudget, ConsumerDeliveryPacingWidth, DeliveryBackpressurePolicy,
    DeliveryWindowWidth, LiveQueryAdmissionArtifact, MaintenanceDeltaWidth, PatchGroupWidth,
    QueryDeliveryWindowBudget, QuerySubscriptionAdmissionBudget,
    QuerySubscriptionBridgeLoweringBudget, QuerySubscriptionConstructionSource,
    QuerySubscriptionScaleCounterSnapshot, QuerySubscriptionScaleFixtureSize,
    QuerySubscriptionSliceBudget, QuerySubscriptionWorkBudget, SubscriptionActivationInput,
    SubscriptionConsumerAttachmentBudget, SubscriptionConsumerAttachmentRequest,
};
use crate::view_shape_live::LiveViewShapeFamily;

pub(super) fn activation_for(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    construction_source: QuerySubscriptionConstructionSource,
) -> Result<SubscriptionActivationInput, IdentityDerivationDenial> {
    let live = LiveQueryAdmissionArtifact::for_test(live_family, view_family, construction_source)?;
    Ok(activation_from_live(live)?)
}

pub(super) fn activation_with_context(
    policy: &str,
) -> Result<SubscriptionActivationInput, IdentityDerivationDenial> {
    let live = LiveQueryAdmissionArtifact::for_test_with_context(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
        crate::subscription::QuerySubscriptionBasisPosture::CurrentHead,
        crate::subscription::QuerySubscriptionFutureSelection::ordinary()?,
        Some(policy.to_string()),
        Some("tenant".to_string()),
        Some("relationship-proof".to_string()),
        crate::subscription::QuerySubscriptionRelationshipProofPosture::Admitted,
    )?;
    Ok(activation_from_live(live)?)
}

pub(super) fn activation_from_live(
    live: LiveQueryAdmissionArtifact,
) -> Result<SubscriptionActivationInput, IdentityDerivationDenial> {
    let selection = select_query_subscription_family(live, work_budget()).unwrap();
    let declaration = declare_query_subscription(selection, slice_budget()).unwrap();
    let lowering = lower_query_subscription_to_bridge(declaration, lowering_budget()).unwrap();
    let admission = admit_query_subscription(lowering, admission_budget()).unwrap();
    Ok(prepare_subscription_activation(admission)?)
}

pub(super) fn active_attachment(
    runtime: &mut crate::subscription::ActiveSubscriptionRuntime,
) -> Result<
    (
        crate::subscription::ActiveSubscriptionLaneHandle,
        crate::subscription::SubscriptionConsumerAttachment,
    ),
    IdentityDerivationDenial,
> {
    let activation = activation_for(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    )?;
    let admission = admit_active_subscription_lane(activation, active_budget()).unwrap();
    let handle = open_active_subscription_lane(runtime, admission).unwrap();
    let attachment = attach_subscription_consumer(
        runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("preview-consumer", "cursor")?,
        attachment_budget(),
    )
    .unwrap();
    Ok((handle, attachment))
}

pub(super) fn scale_slope_report(
    activation: &SubscriptionActivationInput,
    patch_width: u64,
    continuation_width: u64,
) -> Result<crate::subscription::QuerySubscriptionScaleSlopeReport, IdentityDerivationDenial> {
    let row_factor = 10 + patch_width + continuation_width;
    Ok(certify_query_subscription_scale_slope(
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Small,
            row_factor,
            activation,
        )?,
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            row_factor * 10,
            activation,
        )?,
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            row_factor * 100,
            activation,
        )?,
    )
    .unwrap())
}

pub(super) fn scale_axis_evidence(patch_width: u64, continuation_width: u64) -> Vec<String> {
    vec![
        "scale_axis:unrelated_row_count".to_string(),
        "scale_axis:active_lane_count".to_string(),
        "scale_axis:consumers_per_lane".to_string(),
        format!("scale_axis:patch_width:{patch_width}"),
        "scale_axis:group_count".to_string(),
        "scale_axis:delivery_window_width:3".to_string(),
        format!("scale_axis:continuation_remap_width:{continuation_width}"),
        "scale_axis:preview_residue_width:0".to_string(),
        "scale_axis:allocation_scope_width:1".to_string(),
    ]
}

pub(super) fn work_budget() -> QuerySubscriptionWorkBudget {
    QuerySubscriptionWorkBudget::scratch_buffer_only(8, 8, 8, 64, 1)
}

pub(super) fn slice_budget() -> QuerySubscriptionSliceBudget {
    QuerySubscriptionSliceBudget::scratch_buffer_only(8, 8, 8, 8, 8, 8, 8, 8)
}

pub(super) fn lowering_budget() -> QuerySubscriptionBridgeLoweringBudget {
    QuerySubscriptionBridgeLoweringBudget::admitted(1, 8, 8, 1, 1)
}

pub(super) fn admission_budget() -> QuerySubscriptionAdmissionBudget {
    QuerySubscriptionAdmissionBudget::admitted(1, 1, 1, 1, 1)
}

pub(super) fn active_budget() -> ActiveSubscriptionWorkBudget {
    ActiveSubscriptionWorkBudget::admitted(
        ActiveRegistryLookupWidth::measured(1),
        ActiveFanoutWidth::measured(2),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPolicy::LifecycleArena,
    )
}

pub(super) fn attachment_budget() -> SubscriptionConsumerAttachmentBudget {
    SubscriptionConsumerAttachmentBudget::admitted(
        ActiveFanoutWidth::measured(2),
        ConsumerDeliveryPacingWidth::measured(1),
        ActiveAllocationScopeWidth::measured(1),
        DeliveryBackpressurePolicy::RetainWithinWindow,
    )
}

pub(super) fn delivery_budget() -> QueryDeliveryWindowBudget {
    QueryDeliveryWindowBudget::admitted(
        DeliveryWindowWidth::measured(3),
        PatchGroupWidth::measured(3),
        MaintenanceDeltaWidth::measured(3),
        ActiveAllocationScopeWidth::measured(1),
        crate::subscription::ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    )
}
