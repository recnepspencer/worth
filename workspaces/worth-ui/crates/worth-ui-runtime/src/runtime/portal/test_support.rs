use super::{
    UiPortalIdentity, UiPortalOwnerIdentity, UiPortalRuntimeState, UiPortalServiceRequest,
};

pub(super) fn state() -> UiPortalRuntimeState {
    UiPortalRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
    )
}

pub(super) fn portal(graph_node: u64, mounted_instance: u64) -> UiPortalIdentity {
    UiPortalIdentity::for_owner(UiPortalOwnerIdentity::for_test(
        graph_node,
        mounted_instance,
    ))
}

pub(super) fn idempotency(
    lineage: u64,
) -> crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity {
    crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, lineage)
}

pub(super) fn open_request(portal: UiPortalIdentity, lineage: u64) -> UiPortalServiceRequest {
    let geometry = presented_geometry(1);
    UiPortalServiceRequest::open(
        portal,
        idempotency(lineage),
        geometry,
        Some(viewport_bounds(geometry)),
        semantic_surface(),
    )
}

pub(super) fn viewport_bounds(
    geometry: crate::runtime::interaction::UiPresentedInteractionGeometry,
) -> crate::runtime::interaction::UiPresentedViewportGeometry {
    crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
        geometry.clip_bounds(),
        geometry.presentation(),
    )
}

pub(super) fn semantic_surface() -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
    worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
        .expect("test semantic surface identity capacity")
}

pub(super) fn presented_geometry(
    epoch: u64,
) -> crate::runtime::interaction::UiPresentedInteractionGeometry {
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
    crate::runtime::interaction::UiPresentedInteractionGeometry::for_test(presentation)
}
