use super::*;
use crate::UiMountedSemanticTextMechanic;

#[test]
fn portal_child_families_preserve_child_and_portal_clip_intersections() {
    // Literal expected rectangles are independent of the projection algorithm.
    // Text uses an inert qualified contract fixture: this proves preservation,
    // not shaping. The mounted companion proof uses the real font/layout lane.
    let cases = [
        (
            [40.0, 40.0, 30.0, 20.0],
            [100.0, 200.0, 280.0, 320.0],
            [100.0, 200.0, 280.0, 320.0],
            Some([140.0, 240.0, 30.0, 20.0]),
        ),
        (
            [32.0, 32.0, 160.0, 96.0],
            [100.0, 200.0, 280.0, 320.0],
            [150.0, 250.0, 10.0, 20.0],
            Some([150.0, 250.0, 10.0, 20.0]),
        ),
        (
            [32.0, 32.0, 160.0, 96.0],
            [100.0, 200.0, 100.0, 320.0],
            [100.0, 200.0, 280.0, 320.0],
            Some([132.0, 232.0, 68.0, 96.0]),
        ),
        (
            [40.0, 40.0, 30.0, 20.0],
            [100.0, 200.0, 280.0, 320.0],
            [180.0, 240.0, 20.0, 20.0],
            None,
        ),
        // Touching edges have no area.
        (
            [40.0, 40.0, 30.0, 20.0],
            [100.0, 200.0, 280.0, 320.0],
            [170.0, 240.0, 20.0, 20.0],
            None,
        ),
        // A clip inside the Portal can still be disjoint from the child bounds.
        (
            [0.0, 0.0, 10.0, 10.0],
            [100.0, 200.0, 280.0, 320.0],
            [100.0, 200.0, 280.0, 320.0],
            None,
        ),
    ];
    for (child_clip, portal_bounds, portal_clip, expected) in cases {
        let mut input = crate::mounted_projection::semantic_text::tests::fixture();
        input.bounds = rectangle(
            [
                input.bounds.x() + 88.0,
                input.bounds.y() + 160.0,
                input.bounds.width(),
                input.bounds.height(),
            ],
            input.bounds.coordinate_space(),
        );
        input.origin_x += 88.0;
        input.origin_y += 160.0;
        input.clip_bounds = rectangle(
            [
                child_clip[0] + 88.0,
                child_clip[1] + 160.0,
                child_clip[2],
                child_clip[3],
            ],
            input.bounds.coordinate_space(),
        );
        let portal = portal_region(
            input.frame,
            input.surface,
            input.binding,
            rectangle(portal_bounds, UiMountedCoordinateSpace::Viewport),
            rectangle(portal_clip, UiMountedCoordinateSpace::Viewport),
        );
        let paint = UiMountedFilledRectMechanic::complete_from_runtime_mounting(
            UiMountedFilledRectCompletionInput {
                frame: input.frame,
                surface: input.surface,
                binding: input.binding,
                mounted_instance: input.mounted_instance,
                node_receipt: input.node_receipt,
                allocation_basis: input.allocation_basis,
                bounds: input.bounds,
                clip_bounds: input.clip_bounds,
                color: UiMountedRgba8::new(30, 40, 50, 255),
                layer_semantic_order: input.layer_semantic_order,
            },
        )
        .unwrap();
        let hit = UiMountedHitTestMechanic::complete_from_runtime_mounting(
            UiMountedHitTestCompletionInput {
                frame: input.frame,
                surface: input.surface,
                binding: input.binding,
                mounted_instance: input.mounted_instance,
                node_receipt: input.node_receipt,
                bounds: input.bounds,
                clip_bounds: input.clip_bounds,
                order: UiMountedHitTestOrder::from_runtime_plan(9),
            },
        )
        .unwrap();
        let text = UiMountedSemanticTextMechanic::complete_from_runtime_mounting(input).unwrap();
        let painted = paint.presented_within_portal(portal).unwrap();
        let targeted = hit.presented_within_portal(portal).unwrap();
        let written = text.presented_within_portal(portal).unwrap();
        assert_eq!(painted.is_some(), expected.is_some());
        assert_eq!(targeted.is_some(), expected.is_some());
        assert_eq!(written.is_some(), expected.is_some());
        if let Some(expected) = expected {
            let (painted, targeted, written) =
                (painted.unwrap(), targeted.unwrap(), written.unwrap());
            let bounds = rectangle(
                [132.0, 232.0, 160.0, 96.0],
                UiMountedCoordinateSpace::Viewport,
            );
            let clip = rectangle(expected, UiMountedCoordinateSpace::Viewport);
            assert_eq!(
                [painted.bounds(), targeted.bounds(), written.bounds()],
                [bounds; 3]
            );
            assert_eq!(
                [
                    painted.clip_bounds(),
                    targeted.clip_bounds(),
                    written.clip_bounds()
                ],
                [clip; 3]
            );
            assert_eq!(painted.layer_semantic_order(), 2_008);
            assert_eq!(targeted.order().rank(), 9);
            assert_text_preserved(&text, &written);
        }
    }
}

fn assert_text_preserved(
    before: &UiMountedSemanticTextMechanic,
    after: &UiMountedSemanticTextMechanic,
) {
    assert_eq!((after.origin_x(), after.origin_y()), (132.0, 240.0));
    assert_eq!(after.text(), before.text());
    assert_eq!(after.foregrounds(), before.foregrounds());
    assert_eq!(
        after.qualified_layout_identity(),
        before.qualified_layout_identity()
    );
    assert_eq!(
        after.qualified_layout_request(),
        before.qualified_layout_request()
    );
    assert_eq!(
        after.qualified_layout_profile(),
        before.qualified_layout_profile()
    );
    assert_eq!(
        after.qualified_layout_fonts(),
        before.qualified_layout_fonts()
    );
    assert_eq!(
        after.qualified_layout_scale(),
        before.qualified_layout_scale()
    );
    assert_eq!(
        after.qualified_layout_width(),
        before.qualified_layout_width()
    );
    assert_eq!(
        after.performed_layout_cost(),
        before.performed_layout_cost()
    );
    assert_eq!(after.node_receipt(), before.node_receipt());
    assert_eq!(after.layer_semantic_order(), 2_008);
    assert_ne!(after.semantic_digest(), before.semantic_digest());
}

fn rectangle(
    [x, y, width, height]: [f32; 4],
    space: UiMountedCoordinateSpace,
) -> UiMountedCanonicalBox {
    canonical_box(x, y, width, height, space)
}
