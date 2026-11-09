use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

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
        DeliveryWindowWidth::measured(2),
        PatchGroupWidth::measured(2),
        MaintenanceDeltaWidth::measured(2),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    )
}

pub(super) fn declaration_for(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
) -> QuerySubscriptionDeclarationArtifact {
    let live = LiveQueryAdmissionArtifact::for_test(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::Direct,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(live, roomy_budget()).unwrap();
    declare_query_subscription(selection, roomy_slice_budget()).unwrap()
}

pub(super) fn admission_for(
    declaration: &QuerySubscriptionDeclarationArtifact,
) -> QuerySubscriptionAdmissionArtifact {
    let lowering =
        lower_query_subscription_to_bridge(declaration.clone(), roomy_lowering_budget()).unwrap();
    admit_query_subscription(
        lowering,
        QuerySubscriptionAdmissionBudget::admitted(1, 1, 1, 1, 1),
    )
    .unwrap()
}

pub(super) fn continuation_report_for(
    declaration: &QuerySubscriptionDeclarationArtifact,
) -> (
    QuerySubscriptionAdmissionArtifact,
    SubscriptionContinuationReport,
) {
    let admission = admission_for(declaration);
    let activation = prepare_subscription_activation(admission.clone());
    let active_admission = admit_active_subscription_lane(activation, active_budget()).unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let handle = open_active_subscription_lane(&mut runtime, active_admission).unwrap();
    let attachment = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-a", "cursor-a")
            .expect("fixture digest must fit"),
        attachment_budget(),
    )
    .unwrap();
    let window = open_query_delivery_window(&mut runtime, &attachment, delivery_budget()).unwrap();
    let evidence = admit_subscription_continuation_evidence(
        attachment.lane_digest().clone(),
        SubscriptionContinuationClass::IdentityRemap,
        continuation_test_identity("employee:old"),
        continuation_test_identity("employee:new"),
        continuation_test_identity("basis:current"),
        continuation_test_identity("identity-evolution-authority"),
        ContinuationRemapWidth::measured(1),
    )
    .unwrap();
    let (_, report) =
        apply_active_subscription_continuation(&mut runtime, window, evidence).unwrap();
    (admission, report)
}

pub(super) fn preview_closeout_for(
    declaration: &QuerySubscriptionDeclarationArtifact,
) -> (
    QuerySubscriptionAdmissionArtifact,
    SubscriptionLifecycleCloseout,
) {
    let admission = admission_for(declaration);
    let activation = prepare_subscription_activation(admission.clone());
    let active_admission = admit_active_subscription_lane(activation, active_budget()).unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let handle = open_active_subscription_lane(&mut runtime, active_admission).unwrap();
    let attachment = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("preview-a", "cursor-a")
            .expect("fixture digest must fit"),
        attachment_budget(),
    )
    .unwrap();
    let isolation = admit_preview_subscription_isolation(
        &attachment,
        "preview-epoch-a",
        PreviewResidueWidth::measured(1),
    )
    .unwrap();
    let residue = measure_preview_subscription_residue(
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(0),
    );
    let preview_closeout = discard_preview_subscription(isolation, residue).unwrap();
    let closeout = close_subscription_lifecycle(
        &mut runtime,
        &handle,
        SubscriptionLifecycleCloseRequest::PreviewDiscard(preview_closeout),
    )
    .unwrap();
    (admission, closeout)
}
