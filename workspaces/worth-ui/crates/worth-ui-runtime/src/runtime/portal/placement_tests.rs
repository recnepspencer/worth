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

    assert_eq!(placement.anchor().components()[..2], [120.0, 80.0]);
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
    assert_eq!(
        placement.clip_bounds().canonical_box(),
        canonical_box(viewport())
    );
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

#[test]
fn a_fitted_modal_keeps_its_last_extent_when_a_successor_cannot_measure_it() {
    let anchor = [120.0, 80.0, 48.0, 24.0];
    let request = open_request(portal(41, 42), 43, anchor, viewport())
        .with_content_extent(Some(content([200.0, 100.0])));
    let placement = modal(request);
    assert_eq!(
        placement.bounds().components(),
        [380.0, 250.0, 200.0, 100.0]
    );

    let kept = placement
        .succeeded(published(anchor), published(viewport()), || None)
        .expect("the modal places again");
    assert_eq!(kept.bounds(), placement.bounds());

    let grown = placement
        .succeeded(published(anchor), published(viewport()), || {
            Some(content([240.0, 140.0]))
        })
        .expect("the modal places again");
    assert_eq!(grown.bounds().components(), [360.0, 230.0, 240.0, 140.0]);
}

#[test]
fn a_modal_at_its_declared_extent_never_measures_its_content() {
    let anchor = [120.0, 80.0, 48.0, 24.0];
    let placement = modal(open_request(portal(51, 52), 53, anchor, viewport()));
    let succeeded = placement
        .succeeded(published(anchor), published(viewport()), || {
            panic!("a Portal at its declared extent is never measured")
        })
        .expect("the modal places again");
    assert_eq!(succeeded.bounds(), placement.bounds());
}

fn modal(request: UiPortalServiceRequest) -> super::UiPreparedPortalPlacement {
    state()
        .prepare_authored(request, crate::declaration::UiPortalPolicy::modal_dialog())
        .expect("placement prepares")
        .placement()
        .expect("open has placement")
}

fn content([width, height]: [f32; 2]) -> super::UiPortalContentBounds {
    let extent = published([0.0, 0.0, width, height]);
    super::UiPortalContentBounds {
        layout: extent,
        paint: extent,
    }
}

fn published(components: [f32; 4]) -> crate::mounting::presentation::UiPublishedRect {
    crate::mounting::presentation::UiPublishedRect::from_committed_box(canonical_box(components))
}

fn state() -> UiPortalRuntimeState {
    UiPortalRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
    )
}

#[test]
fn shadow_gutter_does_not_change_body_anchor_gap() {
    use worth_ui_host_contract::{
        UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    };
    let local = |x, y, width, height| {
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: UiMountedCoordinateSpace::GraphNodeLocal,
        })
        .unwrap()
    };
    let content = super::UiPortalContentBounds {
        layout: crate::mounting::presentation::UiPublishedRect::from_committed_box(local(
            36.0, 36.0, 307.0, 253.0,
        )),
        paint: crate::mounting::presentation::UiPublishedRect::from_committed_box(local(
            0.0, 0.0, 379.0, 325.0,
        )),
    };
    // Independent product rectangles: body starts eight pixels below the
    // anchor, or ends eight pixels above it; shadow support extends 36 pixels.
    for (anchor, body, paint) in [
        (
            [120.0, 80.0, 48.0, 24.0],
            [120.0, 112.0, 307.0, 253.0],
            [84.0, 76.0, 379.0, 325.0],
        ),
        (
            [120.0, 500.0, 48.0, 24.0],
            [120.0, 239.0, 307.0, 253.0],
            [84.0, 203.0, 379.0, 325.0],
        ),
    ] {
        let mut runtime = state();
        let request =
            open_request(portal(71, 72), 73, anchor, viewport()).with_content_extent(Some(content));
        let surface = request.semantic_surface();
        let transition = runtime.prepare(request).unwrap();
        let placement = transition.placement().unwrap();
        assert_eq!(placement.bounds().components(), body);
        assert_eq!(placement.paint_bounds().components(), paint);
        runtime.commit_published(transition).unwrap();
        let press = |x: f32, y: f32| super::UiPortalDismissalTrigger::OutsidePress {
            semantic_surface: surface,
            point: crate::mounting::presentation::platform_point_for_test(x, y),
        };
        let idempotency =
            crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, 74);
        assert!(matches!(
            runtime
                .prepare_dismissal(press(body[0] + 1.0, body[1] + 1.0), None, idempotency)
                .unwrap(),
            super::UiPortalDismissalPreparation::Ignored(
                super::UiPortalDismissalIgnoreReason::InsideTopmostPortal
            )
        ));
        assert!(
            matches!(
                runtime
                    .prepare_dismissal(press(paint[0] + 1.0, paint[1] + 1.0), None, idempotency)
                    .unwrap(),
                super::UiPortalDismissalPreparation::Prepared(_)
            ),
            "paint gutter is not an input shield"
        );
    }
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
