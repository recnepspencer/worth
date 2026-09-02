use super::super::{
    UiOverlayMotionBinding, UiOverlayMotionOwnerExport, UiOverlayOwnerExportVector,
    UiOverlayPortalBinding, UiOverlayPortalBindingExport, UiOverlayPortalOwnerExport,
    UiOverlaySurfaceExtentSnapshot,
};
use crate::runtime::interaction::{UiPresentedInteractionGeometry, UiPresentedViewportGeometry};
use crate::runtime::portal::{
    UiPortalDismissalCause, UiPortalIdentity, UiPortalInputShielding, UiPortalRuntimeState,
    UiPortalServiceRequest,
};

pub(crate) fn owner_state() -> UiPortalRuntimeState {
    UiPortalRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
    )
}

pub(crate) fn owner_portal(graph: u64) -> UiPortalIdentity {
    UiPortalIdentity::for_test(graph)
}

pub(crate) fn idempotency(
    lineage: u64,
) -> crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity {
    crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, lineage)
}

fn presentation_geometry(epoch: u64) -> UiPresentedInteractionGeometry {
    let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound()
        .expect("test binding identity capacity");
    let presentation = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound()
            .expect("test host surface identity capacity"),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound()
            .expect("test frame identity capacity"),
        binding,
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(epoch),
    );
    UiPresentedInteractionGeometry::for_test(presentation)
}

pub(crate) fn open_request(
    portal: UiPortalIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    lineage: u64,
) -> UiPortalServiceRequest {
    let geometry = presentation_geometry(lineage);
    UiPortalServiceRequest::open(
        portal,
        idempotency(lineage),
        geometry,
        Some(UiPresentedViewportGeometry::for_test(
            geometry.clip_bounds(),
            geometry.presentation(),
        )),
        surface,
    )
}

pub(crate) fn nested_open_request(
    portal: UiPortalIdentity,
    parent: UiPortalIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    lineage: u64,
) -> UiPortalServiceRequest {
    let geometry = presentation_geometry(lineage);
    UiPortalServiceRequest::open_nested(
        portal,
        idempotency(lineage),
        geometry,
        UiPresentedViewportGeometry::for_test(geometry.clip_bounds(), geometry.presentation()),
        surface,
        parent,
        UiPortalInputShielding::ContentBounds,
    )
}

pub(crate) fn close_request(
    portal: UiPortalIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    lineage: u64,
) -> UiPortalServiceRequest {
    UiPortalServiceRequest::close(
        portal,
        idempotency(lineage),
        UiPortalDismissalCause::Escape,
        surface,
    )
}

pub(crate) fn commit_open(owner: &mut UiPortalRuntimeState, request: UiPortalServiceRequest) {
    let transition = owner.prepare(request).expect("owner open prepares");
    owner
        .commit_published(transition)
        .expect("owner open commits");
}

pub(crate) fn prepared_generation(
) -> crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity {
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .expect("prepared generation fixture freezes without a host")
        .generation_identity()
        .clone()
}

pub(crate) fn prepared_generation_variant(
) -> crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity {
    let observation = crate::runtime::observation::UiObservationProfile::bounded(
        crate::runtime::observation::UiObservationProfileInput {
            admitted_per_turn: 2,
            retained_bytes_per_turn: 2_048,
            queued_during_effecting_rebind: 1,
        },
    )
    .expect("variant observation profile is bounded");
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::new(
            observation,
            crate::runtime::rebind::UiRebindProfile::platform_pulse(),
        ))
        .freeze()
        .expect("variant generation fixture freezes without a host")
        .generation_identity()
        .clone()
}

pub(crate) fn motion_export(
    generation: &crate::facade::prepared_application_authority::
        WorthUiPreparedApplicationGenerationIdentity,
    owner_revision: u64,
    rows: impl IntoIterator<Item = UiOverlayMotionBinding>,
) -> UiOverlayMotionOwnerExport {
    UiOverlayMotionOwnerExport::from_prepared(generation.clone(), owner_revision, rows)
        .expect("motion owner export seals")
}

pub(crate) fn owner_exports(
    generation: &crate::facade::prepared_application_authority::
        WorthUiPreparedApplicationGenerationIdentity,
    owner: &UiPortalRuntimeState,
    extent: UiOverlaySurfaceExtentSnapshot,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    bindings: impl IntoIterator<Item = UiOverlayPortalBinding>,
    motion: Option<UiOverlayMotionOwnerExport>,
) -> UiOverlayOwnerExportVector {
    let binding_export = UiOverlayPortalBindingExport::from_prepared(
        generation.clone(),
        extent.runtime_surface(),
        owner.revision(),
        bindings,
    )
    .expect("binding owner export seals");
    UiOverlayOwnerExportVector::from_prepared(
        generation.clone(),
        UiOverlayPortalOwnerExport::from_owner(owner),
        extent,
        presentation,
        binding_export,
        motion,
    )
    .expect("coherent owner export vector seals")
}
