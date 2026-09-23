use super::*;

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
        shadow.occurrence_allocation = UiMountedAllocationProjection::Known {
            bounds: surface_bounds(shadow_box),
            basis: UiMountedAllocationBasis::new(1, 2, 3, UiMountedTransformProjection::Identity),
        };
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
        let frame = world.frame(&[], None);
        let measured = frame
            .portal_content_extent(world.owners[0])
            .unwrap()
            .unwrap();
        assert_eq!(
            [
                measured.layout.x(),
                measured.layout.y(),
                measured.layout.width(),
                measured.layout.height()
            ],
            expected_layout
        );
        assert_eq!(
            [
                measured.paint.x(),
                measured.paint.y(),
                measured.paint.width(),
                measured.paint.height()
            ],
            expected_paint
        );
        complete_measured_content(&world, measured);
    }
}

#[test]
fn content_beginning_before_its_anchor_is_measured_from_the_union() {
    // The overlay is a viewport-inset panel wider than the control that opens
    // it, so its near edge precedes the anchor origin on both axes. Measuring
    // from the anchor origin denied that world outright and reported the
    // refusal as an incompatible coordinate space.
    let mut world = GeometryWorld::new();
    let frame = world.frame(&[], None);
    let measured = frame
        .portal_content_extent(world.owners[0])
        .unwrap()
        .unwrap();
    assert_eq!(
        [
            measured.paint.x(),
            measured.paint.y(),
            measured.paint.width(),
            measured.paint.height()
        ],
        [-12.0, -8.0, 220.0, 120.0]
    );
    assert_eq!(measured.layout, measured.paint);
    complete_measured_content(&world, measured);
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
