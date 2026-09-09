use super::*;
use worth_ui_host_contract::{
    UiMountedPresentationNodeChange, UiMountedPresentationNodeHitTest,
    UiMountedPresentationNodePaint,
};

#[test]
fn mounted_portal_child_clipping_suppresses_and_restores_every_presented_family() {
    let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let child = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);
    let mut mechanics = Default::default();
    let mut frames = Vec::new();
    for (generation, child_x) in [(1, 8.0), (2, 800.0), (3, 8.0)] {
        let semantic = semantic_at(owner, child, surface, binding, child_x);
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let projection = projection_frame_with_identity(
            frame,
            semantic,
            surface,
            binding,
            owner,
            child,
            Arc::clone(&fonts),
            mechanics,
            vec![portal_overlay(frame, owner, surface, binding)],
            generation,
        );
        assert_presented_families(&projection, child, surface, binding, child_x == 8.0);
        mechanics = projection.mechanic_source();
        frames.push(projection);
    }
    // Successors cannot mutate the retained predecessor's visible evidence.
    assert_presented_families(&frames[0], child, surface, binding, true);
    let text = |frame: &UiMountedProjectionFrame| {
        frame
            .presentation_commands_for_instance(child, surface, binding)
            .into_iter()
            .filter_map(|command| match command {
                UiMountedPaintCommand::SemanticText { mechanic, .. } => Some(mechanic.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let original = text(&frames[0]);
    let restored = text(&frames[2]);
    assert_eq!(original.len(), 2);
    assert_eq!(restored.len(), original.len());
    for (before, after) in original.iter().zip(&restored) {
        assert_eq!(before.text(), after.text());
        assert_eq!(
            before.qualified_layout_identity(),
            after.qualified_layout_identity()
        );
        assert_eq!(
            before.qualified_layout_request(),
            after.qualified_layout_request()
        );
        assert_eq!(before.bounds(), after.bounds());
        assert_eq!(before.clip_bounds(), after.clip_bounds());
        assert_eq!(
            (before.origin_x(), before.origin_y()),
            (after.origin_x(), after.origin_y())
        );
    }
}

fn assert_presented_families(
    frame: &UiMountedProjectionFrame,
    child: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    visible: bool,
) {
    // Clipping suppresses mechanics while the open Portal and mounted node stay legal.
    assert!(frame
        .portal_presentation_affinity_for_instance(child, surface, binding)
        .is_some());
    assert!(frame
        .presentation_instance_order(surface, binding)
        .iter()
        .any(|instance| *instance == child));
    let commands = frame.presentation_commands_for_instance(child, surface, binding);
    assert_eq!(commands.len(), if visible { 3 } else { 0 });
    let basis = frame
        .visual_region_basis()
        .for_binding(binding, frame.receipt_basis.clone());
    let indexed = basis
        .presented_hits
        .at_point(
            binding,
            [40.0, 80.0],
            crate::mounting::spatial_index::UiMountedSpatialBudget {
                node_visits: 64,
                candidates: 16,
            },
        )
        .unwrap();
    assert_eq!(
        indexed
            .rows
            .iter()
            .any(|row| row.mounted_instance() == child),
        visible,
        "the retained point index must follow Portal clipping and restoration"
    );
    assert_eq!(
        basis
            .hit_test()
            .iter()
            .any(|row| row.mechanic().mounted_instance() == child),
        visible
    );
    assert_eq!(
        basis
            .paint()
            .iter()
            .any(|row| row.mounted_instance() == child),
        visible
    );
    assert_eq!(
        basis
            .unsupported_paint()
            .iter()
            .any(|row| row.node_receipt().mounted_instance() == child),
        visible
    );
    let view = frame.view_for(binding).unwrap();
    assert!(view
        .nodes()
        .iter()
        .any(|node| node.mounted_instance() == child));
    assert_eq!(
        view.hit_tests()
            .rows()
            .iter()
            .any(|row| row.mounted_instance() == child),
        visible
    );
    assert_eq!(
        view.filled_rects()
            .rows()
            .iter()
            .any(|row| row.mounted_instance() == child),
        visible
    );
    assert_eq!(
        view.semantic_text()
            .rows()
            .iter()
            .any(|row| row.mounted_instance() == child),
        visible
    );
    let changes = frame.presentation_node_changes(&[child], surface, binding);
    let [UiMountedPresentationNodeChange::Upsert(node)] = changes.as_slice() else {
        panic!("an open clipped child retains node authority with omitted mechanics");
    };
    assert_eq!(
        matches!(node.paint(), UiMountedPresentationNodePaint::Command(_)),
        visible
    );
    assert_eq!(
        matches!(node.hit_test(), UiMountedPresentationNodeHitTest::Region(_)),
        visible
    );
    if !visible {
        assert!(matches!(
            node.paint(),
            UiMountedPresentationNodePaint::Omitted(_)
        ));
        assert!(matches!(
            node.hit_test(),
            UiMountedPresentationNodeHitTest::Omitted(_)
        ));
    }
}

fn semantic_at(
    owner: UiMountedInstanceIdentity,
    child: UiMountedInstanceIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    child_x: f32,
) -> UiMountedSemanticProjection {
    let owner_component = crate::capability::ComponentId::new("phase4.portal.trigger").unwrap();
    UiMountedSemanticProjection::initial(
        vec![
            node(
                owner,
                4_151,
                surface,
                bounds([20.0, 20.0, 80.0, 32.0]),
                Some(owner_component.clone()),
                None,
                false,
            ),
            node(
                child,
                4_152,
                surface,
                bounds([child_x, 12.0, 220.0, 120.0]),
                Some(crate::capability::ComponentId::new("phase4.portal.content").unwrap()),
                Some(owner_component),
                true,
            ),
        ],
        vec![UiMountedProjectionSurface {
            surface,
            binding,
            audience: UiMountedProjectionAudience::full(),
        }],
    )
}
