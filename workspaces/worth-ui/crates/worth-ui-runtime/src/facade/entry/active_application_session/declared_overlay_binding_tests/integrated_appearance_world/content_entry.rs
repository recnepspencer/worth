#[path = "content_entry/fixture.rs"]
mod fixture;
#[path = "content_entry/locality.rs"]
mod locality;

use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedPaintCommand, UiSemanticTextSlot,
};

fn accepted_value_order(
    world: &fixture::TextWorld,
    index: usize,
) -> Vec<UiMountedInstanceIdentity> {
    world
        .host
        .accepted_text_commands(world.surfaces[index])
        .expect("surface has accepted text work")
        .1
        .iter()
        .filter_map(|command| match command {
            UiMountedPaintCommand::SemanticText { mechanic, .. }
                if mechanic.slot() == UiSemanticTextSlot::Value =>
            {
                Some(mechanic.mounted_instance())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn split_surface_text_publication_settles_only_exact_accepted_occurrences() {
    let mut world = fixture::TextWorld::launch();
    world.execute(&[0, 1], 1, true);
    world.assert_text(0, "AB");
    world.assert_text(1, "AB");
    world.assert_pending(None);

    world.admit(1, "CD");
    let b = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[1])
        .unwrap();
    world.execute(&[0], 2, true);
    world.assert_text(0, "CD");
    world.assert_text(1, "AB");
    world.assert_pending(Some((2, "CD")));
    let a_frame = world.session.current_mounted_publication().unwrap().frame();
    world.execute(&[1], 3, false);
    assert_eq!(
        world.session.current_mounted_publication().unwrap().frame(),
        a_frame
    );
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[1]),
        Some(b)
    );
    world.assert_text(0, "CD");
    world.assert_text(1, "AB");
    world.assert_pending(Some((2, "CD")));
    world.execute(&[1], 4, true);
    world.assert_text(0, "CD");
    world.assert_text(1, "CD");
    world.assert_pending(None);

    world.admit(2, "EF");
    world.execute(&[0], 5, true);
    world.add_occurrence(0, [240.0, 100.0, 90.0, 25.0]);
    let expected_a_order = world
        .occurrences
        .iter()
        .filter(|(_, surface, _)| *surface == 0)
        .map(|(instance, _, _)| *instance)
        .collect::<Vec<_>>();
    world.execute(&[1], 6, true);
    world.assert_pending(Some((3, "EF")));
    world.assert_text(1, "EF");
    world.execute(&[0], 7, true);
    world.assert_text(0, "EF");
    world.assert_text(1, "EF");
    assert_eq!(accepted_value_order(&world, 0), expected_a_order);
    world.assert_pending(None);
    world.execute(&[1], 8, true);
    assert_eq!(
        accepted_value_order(&world, 0),
        expected_a_order,
        "publishing B preserves the exact mounted text order accepted on omitted A"
    );
    world.execute(&[0], 9, true);
    world.assert_text(0, "EF");
    world.assert_text(1, "EF");
    world.assert_pending(None);
    world.reconstruct_current_text_layouts(0);
    world.execute(&[1], 10, true);
    world.reconstruct_after_rejection(0, "EF", 11);
    world.assert_text(1, "EF");
    let _ = world.session.shutdown();
    assert_eq!(world.host.pending_presentation_count(), 0);
}

#[test]
fn surface_text_retirement_waits_for_its_own_acceptance_and_survives_rejection() {
    let mut world = fixture::TextWorld::launch();
    world.execute(&[0, 1], 1, true);
    world.admit(1, "CD");
    world.execute(&[0], 2, true);
    let a = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let retired = world.occurrences[0].0;
    let old = world
        .session
        .mounted
        .current_projection_rc_for_test()
        .unwrap();
    let replacement = world.replace_occurrence(0);
    assert_ne!(retired, replacement);
    world.execute(&[1], 3, true);
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[0]),
        Some(a)
    );
    world.assert_pending(Some((2, "CD")));
    world.assert_text(1, "CD");
    let before_rejection = world.session.current_mounted_publication().unwrap().frame();
    world.execute(&[0], 4, false);
    assert_eq!(
        world.session.current_mounted_publication().unwrap().frame(),
        before_rejection
    );
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(world.surfaces[0]),
        Some(a)
    );
    let old_view = old.view_for(a.binding()).unwrap();
    assert!(old_view
        .semantic_text()
        .rows()
        .iter()
        .any(|row| row.mounted_instance() == retired && row.text() == "CD"));
    world.assert_pending(Some((2, "CD")));
    world.execute(&[0], 5, true);
    world.assert_text(0, "CD");
    world.assert_text(1, "CD");
    world.assert_pending(None);
    let _ = world.session.shutdown();
    assert_eq!(world.host.pending_presentation_count(), 0);
}
