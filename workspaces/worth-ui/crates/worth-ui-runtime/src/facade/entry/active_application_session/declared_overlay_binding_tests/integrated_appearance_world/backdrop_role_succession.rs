use super::{authored, session::World};

#[test]
fn backdrop_only_role_body_edit_repaints_and_recovers() {
    let mut world = World::launch();
    let frame = world.prepare();
    world.publish(frame, 1, true);
    world.open(0, "overlay.menu", None, 10);
    let frame = world.prepare_surface_with_current_portals(world.surfaces[0]);
    world.publish(frame, 30, false);
    for (revision, (slot, expected)) in [
        ("overlay.unused.background", [1, 2, 3, 255]),
        ("overlay.scrim.background", [4, 8, 12, 255]),
    ]
    .into_iter()
    .enumerate()
    {
        let source = authored::source().replace(
            "background use token(overlay.scrim.background)",
            &format!("background use token({slot})"),
        );
        let submission = crate::runtime::WorthUiReloadDebounce::default()
            .debounce(
                crate::runtime::WorthUiSourceProvider::in_memory("integrated-overlay")
                    .with_file("app/main.wui", &source),
                &[crate::runtime::WorthUiWatcherEvent::provider_revision(
                    "integrated-overlay",
                )],
                revision as u64 + 1,
            )
            .unwrap()
            .attempt_candidate_for_certification(world.session.capabilities())
            .unwrap();
        let mut turn = world.session.begin_observation_turn().unwrap();
        turn.admit_source(submission).unwrap();
        let observations = turn.seal().unwrap();
        let crate::runtime::observation::UiChangeClassificationOutcome::Changed(changed) =
            world.session.classify_observations(observations).unwrap()
        else {
            panic!("backdrop-only role meaning requires successor presentation");
        };
        assert_eq!(changed.facts().len(), 2);
        for surface in [
            "surface:workspace.surface.overlay",
            "surface:workspace.surface.secondary",
        ] {
            assert!(changed
                .facts()
                .iter()
                .any(|fact| fact.authored_source().unwrap().selector()
                    == &crate::fact_contract::UiAuthoredFactSelector::node(surface)));
        }
        let lifecycle = world
            .session
            .resolve_affected_scope(changed)
            .unwrap()
            .resolve_identity_lifecycle()
            .unwrap();
        let plan = world
            .session
            .compile_rebind_plan(
                lifecycle,
                crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
            )
            .unwrap();
        for _ in world.surfaces {
            world.host.push_native_display_settled_without_effects();
        }
        match world
            .session
            .prepare_rebind(
                plan,
                crate::runtime::rebind::UiRebindExecutionRequest::new(40),
            )
            .unwrap()
            .execute(40)
        {
            crate::runtime::rebind::UiRebindOutcome::Published(receipt) => {
                assert!(receipt.mounted_publication().is_some())
            }
            crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) => {
                panic!("backdrop role cutover denied: {:?}", denial.cause())
            }
            _ => panic!("backdrop role replacement must publish"),
        }
        let output = world
            .session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .unwrap();
        let mut backdrops = 0;
        for fragment in output.fragments() {
            for mechanic in fragment.work().successor().mechanics() {
                if let worth_ui_host_contract::UiMountedAppearanceMechanic::Backdrop(backdrop) =
                    mechanic
                {
                    backdrops += 1;
                    assert_eq!(backdrop.background().straight_srgba(), expected);
                }
            }
        }
        assert_eq!(
            backdrops, 3,
            "two Portal-relative backdrops and the secondary ambient backdrop"
        );
    }
    let _ = world.session.shutdown();
}
