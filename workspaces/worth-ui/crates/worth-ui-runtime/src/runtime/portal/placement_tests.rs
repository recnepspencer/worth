use super::{
    UiPortalIdentity, UiPortalOwnerIdentity, UiPortalPlacementSide, UiPortalRuntimeState,
    UiPortalServiceRequest,
};

#[test]
fn open_uses_exact_presented_anchor_and_prefers_below() {
    let portal = portal(11, 12);
    let request = open_request(portal, 13, [120.0, 80.0, 48.0, 24.0], viewport());
    let transition = state().prepare(request).expect("placement prepares");
    let placement = transition.placement().expect("open has placement");

    assert_eq!(placement.anchor().x(), 120.0);
    assert_eq!(placement.anchor().y(), 80.0);
    assert_eq!(
        placement.bounds().components(),
        [120.0, 112.0, 280.0, 320.0]
    );
    assert_eq!(placement.side(), UiPortalPlacementSide::Below);
    assert_eq!(placement.presentation().epoch().diagnostic_value(), 4);
    assert_eq!(placement.layer().portal(), portal);
    assert_eq!(placement.layer().parent(), None);
    assert_eq!(placement.layer().depth(), 0);
}

#[test]
fn placement_flips_above_and_clamps_horizontally() {
    let portal = portal(21, 22);
    let request = open_request(portal, 23, [900.0, 520.0, 44.0, 24.0], viewport());
    let placement = state()
        .prepare(request)
        .expect("placement prepares")
        .placement()
        .expect("open has placement");

    assert_eq!(placement.side(), UiPortalPlacementSide::Above);
    assert_eq!(
        placement.bounds().components(),
        [664.0, 192.0, 280.0, 320.0]
    );
}

#[test]
fn constrained_viewport_uses_larger_side_without_leaving_boundary() {
    let portal = portal(31, 32);
    let request = open_request(
        portal,
        33,
        [100.0, 120.0, 40.0, 20.0],
        [0.0, 0.0, 260.0, 220.0],
    );
    let placement = state()
        .prepare(request)
        .expect("partial placement prepares")
        .placement()
        .expect("open has placement");

    assert_eq!(placement.side(), UiPortalPlacementSide::Above);
    assert_eq!(placement.bounds().components(), [16.0, 16.0, 228.0, 96.0]);
}

#[test]
fn viewport_fit_keeps_a_portal_presentable_when_the_anchor_consumes_both_sides() {
    let portal = portal(34, 35);
    let request = open_request(
        portal,
        36,
        [16.0, 12.0, 608.0, 456.0],
        [16.0, 12.0, 608.0, 456.0],
    );
    let placement = state()
        .prepare(request)
        .expect("viewport-fit placement prepares")
        .placement()
        .expect("open has placement");

    assert_eq!(placement.side(), UiPortalPlacementSide::ViewportFit);
    assert_eq!(placement.bounds().components(), [32.0, 28.0, 280.0, 320.0]);
}

#[test]
fn changed_anchor_is_not_coalesced_as_an_exact_duplicate() {
    let mut state = state();
    let portal = portal(41, 42);
    let first_request = open_request(portal, 43, [40.0, 40.0, 40.0, 20.0], viewport());
    let first_surface = first_request.semantic_surface();
    let first = state
        .prepare(first_request)
        .expect("first placement prepares");
    state
        .commit_published(first)
        .expect("first placement commits");
    let moved = state
        .prepare(open_request_on_surface(
            portal,
            43,
            [80.0, 40.0, 40.0, 20.0],
            viewport(),
            first_surface,
        ))
        .expect("moved placement prepares");

    assert_eq!(
        moved.disposition(),
        super::UiPortalServiceDisposition::Opened
    );
}

#[test]
fn local_anchor_clip_cannot_masquerade_as_the_presented_viewport() {
    let portal = portal(51, 52);
    let anchor = [544.0, 432.0, 80.0, 16.0];
    let presentation = presentation();
    let request = UiPortalServiceRequest::open(
        portal,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, 53),
        crate::runtime::interaction::UiPresentedInteractionGeometry::for_test_with_components(
            presentation,
            anchor,
            [528.0, 416.0, 112.0, 48.0],
        ),
        Some(presented_viewport(viewport(), presentation)),
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
            .expect("test semantic surface identity capacity"),
    );
    let placement = state()
        .prepare(request)
        .expect("committed viewport evidence prepares placement")
        .placement()
        .expect("open has placement");

    assert_eq!(
        placement.bounds().components(),
        [544.0, 104.0, 280.0, 320.0]
    );
    assert_eq!(placement.clip_bounds(), canonical_box(viewport()));
}

#[test]
fn missing_presented_viewport_denies_before_portal_truth_changes() {
    let portal = portal(61, 62);
    let request = UiPortalServiceRequest::open(
        portal,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, 63),
        crate::runtime::interaction::UiPresentedInteractionGeometry::for_test(presentation()),
        None,
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
            .expect("test semantic surface identity capacity"),
    );

    assert!(matches!(
        state().prepare(request),
        Err(super::UiPortalServiceTransitionDenial::Placement(
            super::UiPortalPlacementDenial::MissingPresentedViewport
        ))
    ));
}

fn state() -> UiPortalRuntimeState {
    UiPortalRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
    )
}

fn viewport() -> [f32; 4] {
    [0.0, 0.0, 960.0, 600.0]
}

fn portal(graph_node: u64, mounted_instance: u64) -> UiPortalIdentity {
    UiPortalIdentity::for_owner(UiPortalOwnerIdentity::for_test(
        graph_node,
        mounted_instance,
    ))
}

fn open_request(
    portal: UiPortalIdentity,
    lineage: u64,
    anchor: [f32; 4],
    clip: [f32; 4],
) -> UiPortalServiceRequest {
    open_request_on_surface(
        portal,
        lineage,
        anchor,
        clip,
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
            .expect("test semantic surface identity capacity"),
    )
}

fn open_request_on_surface(
    portal: UiPortalIdentity,
    lineage: u64,
    anchor: [f32; 4],
    clip: [f32; 4],
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> UiPortalServiceRequest {
    let presentation = presentation();
    UiPortalServiceRequest::open(
        portal,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, lineage),
        crate::runtime::interaction::UiPresentedInteractionGeometry::for_test_with_components(
            presentation,
            anchor,
            anchor,
        ),
        Some(presented_viewport(clip, presentation)),
        surface,
    )
}

fn canonical_box(components: [f32; 4]) -> worth_ui_host_contract::UiMountedCanonicalBox {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: components[0],
            y: components[1],
            width: components[2],
            height: components[3],
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .expect("test viewport is canonical")
}

fn presented_viewport(
    components: [f32; 4],
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
) -> crate::runtime::interaction::UiPresentedViewportGeometry {
    crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
        canonical_box(components),
        presentation,
    )
}

fn presentation() -> worth_ui_host_contract::UiHostObservationPresentationBasis {
    let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound()
        .expect("test binding identity capacity");
    worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound()
            .expect("test host surface identity capacity"),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound()
            .expect("test frame identity capacity"),
        binding,
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(4),
    )
}
