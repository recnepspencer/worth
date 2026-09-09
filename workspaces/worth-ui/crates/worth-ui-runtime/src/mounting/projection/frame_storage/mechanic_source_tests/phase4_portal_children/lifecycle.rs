use super::*;

#[test]
fn authored_portal_child_projects_only_during_the_exact_open_lifecycle() {
    let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let child = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let semantic = portal_semantic_projection(owner, child, surface, binding);
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
    let closed_owner_paint = owner_filled_rect(&closed, owner, surface, binding);

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
            .filter(|command| matches!(command, UiMountedPaintCommand::FilledRect { .. }))
            .count(),
        1
    );
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
    assert_eq!(
        owner_filled_rect(&open, owner, surface, binding).bounds(),
        closed_owner_paint.bounds(),
        "opening the Portal cannot move its ordinary trigger owner"
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
