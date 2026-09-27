use super::*;
use crate::mounting::projection::appearance::UiMountedAppearanceClip as Clip;
use worth_ui_host_contract::UiAppearanceClip;

#[test]
fn mounted_shadow_caster_and_paint_are_measured_separately() {
    for (sigma, shadow_box, body_box, expected_layout, expected_paint) in [
        (
            12_000,
            [20.0, 20.0, 379.0, 325.0],
            [56.0, 56.0, 307.0, 253.0],
            [36.0, 36.0, 307.0, 253.0],
            [0.0, 0.0, 379.0, 325.0],
        ),
        (
            100,
            [20.0, 20.0, 100.0, 100.0],
            [20.3, 20.3, 99.4, 99.4],
            [0.0, 0.0, 100.0, 100.0],
            [0.0, 0.0, 100.0, 100.0],
        ),
    ] {
        let mut world = GeometryWorld::new();
        let owner = world.semantic.node(world.owners[0]).unwrap().clone();
        let mut shadow = world.semantic.node(world.children[0]).unwrap().clone();
        shadow.occurrence_allocation =
            crate::mounting::UiLaidOut::from_layout(UiMountedAllocationProjection::Known {
                bounds: surface_bounds(shadow_box),
                basis: UiMountedAllocationBasis::new(
                    1,
                    2,
                    3,
                    UiMountedTransformProjection::Identity,
                ),
            });
        shadow.surface_geometry = worth_ui_host_contract::UiSurfaceGeometry::SoftShadow(
            worth_ui_host_contract::UiSoftShadowGeometry::new(
                worth_ui_host_contract::UiAppearanceLogicalLength::new(sigma).unwrap(),
                worth_ui_host_contract::UiAppearanceLogicalLength::new(16_000).unwrap(),
            )
            .unwrap(),
        );
        shadow.semantic_text = None;
        world.semantic.insert_node(shadow);
        let body = node(
            UiMountedInstanceIdentity::mint_unbound().unwrap(),
            4_153,
            world.surfaces[0],
            surface_bounds(body_box),
            Some(crate::capability::ComponentId::new("phase4.portal.body").unwrap()),
            owner.component_id,
            false,
        );
        world.semantic.insert_node(body);
        let measured = world
            .semantic
            .portal_content_extent(world.owners[0], unclipped)
            .unwrap()
            .unwrap();
        assert_eq!(measured.layout.components(), expected_layout);
        assert_eq!(measured.paint.components(), expected_paint);
        complete_measured_content(&world, measured);
    }
}

#[test]
fn content_beginning_before_its_anchor_is_measured_from_the_union() {
    // The overlay is a viewport-inset panel wider than the control that opens
    // it, so its near edge precedes the anchor origin on both axes. Measuring
    // from the anchor origin denied that world outright and reported the
    // refusal as an incompatible coordinate space.
    let world = GeometryWorld::new();
    let measured = world
        .semantic
        .portal_content_extent(world.owners[0], unclipped)
        .unwrap()
        .unwrap();
    assert_eq!(measured.paint.components(), [-12.0, -8.0, 220.0, 120.0]);
    assert_eq!(measured.layout, measured.paint);
    complete_measured_content(&world, measured);
}

#[test]
fn content_is_measured_as_far_as_regions_inside_it_let_the_portal_show_it() {
    // The child is laid out at [8, 12, 220, 120], from an anchor at [20, 20],
    // and a region inside the Portal shows only its top 60 points.
    let world = GeometryWorld::new();
    let region = Clip::Ancestor(UiAppearanceClip::new(0, 12_000, 400_000, 60_000).unwrap());
    let measured = world
        .semantic
        .portal_content_extent(world.owners[0], |_, _| region)
        .unwrap()
        .unwrap();
    assert_eq!(measured.paint.components(), [-12.0, -8.0, 220.0, 60.0]);
    assert_eq!(measured.layout, measured.paint);
    complete_measured_content(&world, measured);
    // Content those regions leave nothing of cannot appear, so it has no
    // extent.
    assert_eq!(
        world
            .semantic
            .portal_content_extent(world.owners[0], |_, _| Clip::Suppressed)
            .unwrap(),
        None
    );
}

#[test]
fn a_shadow_whose_body_a_region_hides_still_paints_its_blur() {
    // The shadow blurs 36 points out from the body casting it, and a region
    // inside the Portal shows only the 20 points above that body.
    let mut world = GeometryWorld::new();
    let mut shadow = world.semantic.node(world.children[0]).unwrap().clone();
    shadow.occurrence_allocation =
        crate::mounting::UiLaidOut::from_layout(UiMountedAllocationProjection::Known {
            bounds: surface_bounds([20.0, 20.0, 379.0, 325.0]),
            basis: UiMountedAllocationBasis::new(1, 2, 3, UiMountedTransformProjection::Identity),
        });
    shadow.surface_geometry = worth_ui_host_contract::UiSurfaceGeometry::SoftShadow(
        worth_ui_host_contract::UiSoftShadowGeometry::new(
            worth_ui_host_contract::UiAppearanceLogicalLength::new(12_000).unwrap(),
            worth_ui_host_contract::UiAppearanceLogicalLength::new(16_000).unwrap(),
        )
        .unwrap(),
    );
    world.semantic.insert_node(shadow);
    let region = Clip::Ancestor(UiAppearanceClip::new(0, 20_000, 400_000, 20_000).unwrap());
    let measured = world
        .semantic
        .portal_content_extent(world.owners[0], |_, _| region)
        .unwrap()
        .unwrap();
    // The blur paints; with no body showing, the content lays out as far as
    // it paints.
    assert_eq!(measured.paint.components(), [0.0, 0.0, 379.0, 20.0]);
    assert_eq!(measured.layout, measured.paint);
    complete_measured_content(&world, measured);
}

#[test]
fn content_that_shows_nothing_adds_nothing_to_the_extent() {
    // A container that only divides its allocation paints nothing and
    // carries no text, however far it is laid out; the content it lays out
    // counts itself.
    let mut world = GeometryWorld::new();
    let owner = world.semantic.node(world.owners[0]).unwrap().clone();
    let mut container = node(
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        4_153,
        world.surfaces[0],
        surface_bounds([0.0, 0.0, 900.0, 900.0]),
        Some(crate::capability::ComponentId::new("phase4.portal.container").unwrap()),
        owner.component_id,
        false,
    );
    container.has_appearance_attachment = false;
    world.semantic.insert_node(container.clone());
    let measured = world
        .semantic
        .portal_content_extent(world.owners[0], unclipped)
        .unwrap()
        .unwrap();
    assert_eq!(measured.paint.components(), [-12.0, -8.0, 220.0, 120.0]);
    assert_eq!(measured.layout, measured.paint);
    // The same container with text to show counts.
    container.semantic_text = Some(UiMountedSemanticTextSeed::scalar_for_test());
    world.semantic.insert_node(container);
    let measured = world
        .semantic
        .portal_content_extent(world.owners[0], unclipped)
        .unwrap()
        .unwrap();
    assert_eq!(measured.paint.components(), [-20.0, -20.0, 900.0, 900.0]);
}

/// No region inside the content clips it.
fn unclipped(_: UiSemanticSurfaceIdentity, _: UiMountedInstanceIdentity) -> Clip {
    Clip::Unclipped
}

fn complete_measured_content(
    world: &GeometryWorld,
    content: crate::runtime::portal::UiPortalContentBounds,
) {
    use crate::runtime::portal::*;
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let presentation = UiHostObservationPresentationBasis::new(
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        frame,
        world.bindings[0],
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let portal = UiPortalIdentity::for_owner(UiPortalOwnerIdentity::from_mounted_owner(
        crate::graph::UiGraphNodeIdentity::new(4_151),
        world.owners[0],
    ));
    let request = UiPortalServiceRequest::open(
        portal,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, 1),
        crate::runtime::interaction::UiPresentedInteractionGeometry::for_test_with_components(
            presentation,
            [20.0, 20.0, 80.0, 32.0],
            [20.0, 20.0, 80.0, 32.0],
        ),
        Some(
            crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
                bounds([0.0, 0.0, 960.0, 600.0]),
                presentation,
            ),
        ),
        world.surfaces[0],
    )
    .with_content_extent(Some(content));
    let state = UiPortalRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
    );
    let transition = state.prepare(request).unwrap();
    let input = crate::mounting::UiMountedPortalOverlayProjectionInput::new(
        portal.diagnostic_value(),
        transition
            .stack_ordinal()
            .expect("opening carries issued order"),
        world.owners[0],
        world.surfaces[0],
        transition.placement().unwrap(),
        UiPortalLifecyclePosture::Visible,
    );
    let receipt = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame)
        .unwrap()
        .receipt_for(world.owners[0]);
    input
        .mechanic_for(frame, world.bindings[0], receipt)
        .expect("measured integral and fractional shadows cross host completion");
}
