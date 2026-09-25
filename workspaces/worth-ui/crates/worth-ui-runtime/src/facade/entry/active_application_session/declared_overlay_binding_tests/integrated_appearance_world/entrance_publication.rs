use super::session::World;

#[test]
fn portal_opening_publication_contains_its_first_motion_sample() {
    let mut world = World::launch();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    world.open(0, "overlay.menu", None, 10);
    let opening = world.host.last_appearance_samples();
    assert!(
        !opening.is_empty(),
        "the opening host call must already contain entrance paint"
    );
    for sample in &opening {
        assert_eq!(
            sample.opacity().factor(),
            0.0,
            "no full-opacity flash before Motion starts"
        );
        let transform = sample
            .transform()
            .expect("Portal entrance translates its complete group");
        assert_eq!(transform.sampled().x(), transform.source().x());
        assert_eq!(transform.sampled().y(), transform.source().y() + 8.0);
        assert_eq!(transform.sampled().width(), transform.source().width());
        assert_eq!(transform.sampled().height(), transform.source().height());
    }
    assert!(
        opening
            .iter()
            .any(|sample| sample.command().mounted_instance() == world.instances[4]),
        "authored Portal content participates with its owning Portal command"
    );
    let opened_basis = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let opened_hit = world
        .session
        .mounted
        .interaction_hit_test_basis(opened_basis.basis())
        .unwrap();
    let opened_content = *opened_hit
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == world.instances[4])
        .expect("Portal semantics preserve interaction visibility during entrance");
    assert_eq!(
        opened_content.bounds().platform_box().y(),
        opened_content.mounted().bounds().y() + 8.0
    );
    let ordinary = world.prepare_surface_with_current_portals(world.surfaces[0]);
    world.publish(ordinary, 11, false);
    let basis = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    for sample in &opening {
        let accepted = world
            .session
            .mounted
            .accepted_motion_for_command(basis.basis(), sample.command())
            .unwrap()
            .expect("ordinary publication retains the exact command's accepted entrance");
        assert_eq!(accepted.opacity_units(), 0);
        assert_eq!(
            accepted.geometry().unwrap().components()[1],
            accepted.base_geometry().unwrap().components()[1] + 8.0
        );
    }
    let hit = world
        .session
        .mounted
        .interaction_hit_test_basis(basis.basis())
        .unwrap();
    let content = hit
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == world.instances[4])
        .unwrap();
    assert_eq!(
        content.bounds().platform_box(),
        opened_content.bounds().platform_box(),
        "ordinary succession preserves the accepted Motion geometry without paint-controlled shielding"
    );
    let first = world.session.prepare_motion_tick(12, basis).unwrap();
    assert_eq!(first.receipt().samples().len(), 1);
    assert_eq!(
        first.receipt().samples()[0].opacity_units(),
        0,
        "the next Motion sample continues from the opening frame"
    );
}
