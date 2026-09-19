use super::*;

#[test]
fn authored_portal_child_projects_only_during_the_exact_open_lifecycle() {
    let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let child = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let mut semantic = portal_semantic_projection(owner, child, surface, binding);
    // The completed occurrence has moved from the declaration receipt's box.
    // Its host position is (38, 42); the anchor is (20, 20) and the presented
    // Portal starts at (20, 60), so the independently expected position is (38, 82).
    let mut relocated = semantic.node(child).unwrap().clone();
    let UiMountedAllocationProjection::Known { basis, .. } = relocated.occurrence_allocation else {
        panic!("fixture supplies concrete occurrence geometry")
    };
    relocated.occurrence_allocation = UiMountedAllocationProjection::Known {
        bounds: surface_bounds([38.0, 42.0, 220.0, 120.0]),
        basis,
    };
    semantic.insert_node(relocated);
    let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
    let fonts = Arc::new(fonts);

    let closed = projection_frame(
        semantic.clone(),
        surface,
        binding,
        owner,
        child,
        Arc::clone(&fonts),
        Default::default(),
        Vec::new(),
        1,
    );
    assert_child_suppressed(&closed, child, surface, binding);
    let closed_owner_allocation = appearance_allocation(&closed, owner);

    let open_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let overlay = portal_overlay(open_frame, owner, surface, binding);
    let open = projection_frame_with_identity(
        open_frame,
        semantic.clone(),
        surface,
        binding,
        owner,
        child,
        Arc::clone(&fonts),
        closed.mechanic_source(),
        vec![overlay],
        2,
    );
    let affinity = open
        .portal_presentation_affinity_for_instance(child, surface, binding)
        .expect("the open authored child has exact Portal presentation affinity");
    assert_eq!(affinity.owner(), owner);
    let child_commands = open.presentation_commands_for_instance(child, surface, binding);
    assert_eq!(
        child_commands
            .iter()
            .filter(|command| matches!(command, UiMountedPaintCommand::SemanticText { .. }))
            .count(),
        2,
        "the authored body text and its paint move as one Portal group"
    );
    assert!(child_commands
        .iter()
        .all(|command| command_bounds(command).x() > 0.0));
    let open_hits = open
        .visual_region_basis()
        .for_binding(binding, open.receipt_basis.clone())
        .hit_test();
    let child_hit = open_hits
        .iter()
        .find(|hit| hit.mechanic().mounted_instance() == child)
        .expect("the open authored child contributes one translated hit region");
    assert_eq!(child_hit.portal().map(|portal| portal.owner()), Some(owner));
    let expected = bounds([38.0, 82.0, 220.0, 120.0]);
    let complete = open.view_for(binding).unwrap();
    let projected = complete
        .nodes()
        .iter()
        .find(|node| node.mounted_instance() == child)
        .unwrap();
    assert!(
        matches!(projected.allocation(), UiMountedAllocationProjection::Known { bounds, .. } if bounds == expected)
    );
    let hit = complete
        .hit_tests()
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == child)
        .unwrap();
    assert_eq!(hit.bounds(), expected);
    let changes = open.presentation_node_changes(&[child], surface, binding);
    let [worth_ui_host_contract::UiMountedPresentationNodeChange::Upsert(delta)] =
        changes.as_slice()
    else {
        panic!("the visible child has one complete delta node")
    };
    assert!(
        matches!(delta.allocation(), UiMountedAllocationProjection::Known { bounds, .. } if bounds == expected)
    );
    assert_eq!(appearance_allocation(&open, owner), closed_owner_allocation);
    assert_eq!(
        open.presentation_commands_for_instance(owner, surface, binding)
            .iter()
            .filter(|command| matches!(command, UiMountedPaintCommand::PortalOverlay { .. }))
            .count(),
        1,
        "the owner contributes its Portal overlay without moving its ordinary appearance allocation"
    );

    let closed_successor = projection_frame(
        semantic,
        surface,
        binding,
        owner,
        child,
        fonts,
        open.mechanic_source(),
        Vec::new(),
        3,
    );
    assert_child_suppressed(&closed_successor, child, surface, binding);
}

fn appearance_allocation(
    frame: &UiMountedProjectionFrame,
    instance: UiMountedInstanceIdentity,
) -> UiMountedAllocationProjection {
    frame
        .appearance_node_inputs_for_reconstruction()
        .expect("the mounted frame exposes reconstructible appearance inputs")
        .into_iter()
        .find(|context| context.mounted_instance == instance)
        .expect("the owner retains one appearance input")
        .allocation
}
