use super::super::{authored, geometry, session::World};

pub(super) fn advance(world: &mut World) {
    let predecessor = world.session.active_generation_identity();
    let bindings = world.session.authored_overlay_binding_exports().unwrap();
    let regions = ["workspace.surface.overlay", "workspace.surface.secondary"].map(|surface| {
        world
            .session
            .application
            .authored_overlay_material()
            .overlay_declaration_bindings()
            .region_named(surface, "workspace.region.primary")
            .unwrap()
    });
    let bounds = std::array::from_fn::<_, 2, _>(|index| {
        world
            .session
            .mounted
            .current_region_extents(
                world.surfaces[index],
                predecessor.prepared_generation(),
                regions[index],
            )
            .unwrap()
    });
    assert!(bounds[0].contains(&(
        world.instances[0],
        geometry::canonical([40.0, 50.0, 180.0, 60.0])
    )));
    assert!(bounds[1].contains(&(
        world.instances[3],
        geometry::canonical([50.0, 60.0, 120.0, 40.0])
    )));
    let calls = world.host.presentation_calls();
    let submission =
        crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission(
            crate::runtime::WorthUiSourceProvider::in_memory("integrated-overlay")
                .with_file("app/main.wui", format!("{}\n", authored::source())),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(
                "integrated-overlay",
            )],
            world.session.application.capabilities(),
        );
    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_source(submission).unwrap();
    let observations = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) =
        world.session.classify_observations(observations).unwrap()
    else {
        panic!("whitespace retains all authored meaning");
    };
    let plan = world
        .session
        .compile_preservation_rebind(
            evidence,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    assert!(matches!(
        world
            .session
            .prepare_rebind(
                plan,
                crate::runtime::rebind::UiRebindExecutionRequest::new(35),
            )
            .unwrap()
            .execute(35),
        crate::runtime::rebind::UiRebindOutcome::Published(_)
    ));
    let successor = world.session.active_generation_identity();
    assert_ne!(successor, predecessor);
    assert_eq!(
        world.host.presentation_calls(),
        calls,
        "evidence-only publication has no physical work"
    );
    let current = world.session.authored_overlay_binding_exports().unwrap();
    assert_eq!(current.len(), bindings.len());
    for (before, after) in bindings.iter().zip(current.iter()) {
        assert_eq!(after.generation(), successor.prepared_generation());
        assert_eq!(after.runtime_surface(), before.runtime_surface());
        assert_eq!(after.rows(), before.rows());
    }
    for index in 0..2 {
        assert_eq!(
            world
                .session
                .mounted
                .current_region_extents(
                    world.surfaces[index],
                    successor.prepared_generation(),
                    regions[index],
                )
                .unwrap(),
            bounds[index]
        );
        assert!(world
            .session
            .mounted
            .current_region_extents(
                world.surfaces[index],
                predecessor.prepared_generation(),
                regions[index],
            )
            .is_none());
    }
}
