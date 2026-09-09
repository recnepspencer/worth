use worth_ui_host_contract::*;

#[path = "appearance_projection_focus_fixture.rs"]
mod fixture;
#[path = "appearance_projection_order_tests.rs"]
mod order_tests;

#[path = "appearance_projection_mount_lifecycle_tests.rs"]
mod mount_lifecycle_tests;

#[path = "appearance_projection_text_tests.rs"]
mod text_tests;

#[test]
fn focus_owner_changes_touch_only_old_and_new_mounted_neighborhoods() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role);
    let (surface, graph_node) = super::mounting_fixture::mount(&mut session, 1_000);
    let neighbor = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            neighbor,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1_000,
                crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let node = session.mounted_graph_node(graph_node).unwrap();
    let unrelated = session.mount_instance(node, neighbor).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        neighbor,
    );
    fixture::close_source(&mut session, &role, "focus-neighborhood-initial");
    session.advance_mounted_identity_frame().unwrap();
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("initial focus frame must prepare"));
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        3
    );
    fixture::publish(&mut session, &host, frame, 1);

    // Owner modality changes with no focused target cause no appearance work.
    observe(&mut session, surface, true);
    fixture::close_source(&mut session, &role, "focus-window-without-target");
    drop(project(&mut session, &[]));

    let first = traverse(&mut session, surface).expect("first real mounted participant");
    assert_ne!(first, unrelated);
    fixture::close_source(&mut session, &role, "focus-first-target");
    let frame = project(&mut session, &[(first, 30)]);
    fixture::publish(&mut session, &host, frame, 2);

    let second = traverse(&mut session, surface).expect("second real mounted participant");
    assert_ne!(first, second);
    assert_ne!(second, unrelated);
    fixture::close_source(&mut session, &role, "focus-second-target");
    let frame = project(&mut session, &[(first, 10), (second, 30)]);
    fixture::publish(&mut session, &host, frame, 3);

    observe(&mut session, surface, false);
    fixture::close_source(&mut session, &role, "focus-window-inactive");
    let frame = project(&mut session, &[(second, 40)]);
    fixture::publish(&mut session, &host, frame, 4);

    observe(&mut session, surface, false);
    fixture::close_source(&mut session, &role, "focus-window-inactive-again");
    drop(project(&mut session, &[]));

    assert_eq!(traverse(&mut session, surface), None);
    fixture::close_source(&mut session, &role, "focus-cleared");
    let frame = project(&mut session, &[(second, 10)]);
    fixture::publish(&mut session, &host, frame, 5);

    assert_eq!(traverse(&mut session, neighbor), Some(unrelated));
    fixture::close_source(&mut session, &role, "focus-other-surface");
    drop(project(&mut session, &[(unrelated, 40)]));
    let _ = session.shutdown();
}

fn observe(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    focused: bool,
) {
    let presentation = session
        .mounted
        .current_publication()
        .unwrap()
        .presentation_for_surface(surface)
        .unwrap();
    session.focus.as_mut().unwrap().observe_host_payload(
        &UiHostObservationPayload::WindowFocus {
            surface: presentation.host_surface(),
            focused,
        },
        presentation,
    );
}

fn traverse(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
) -> Option<UiMountedInstanceIdentity> {
    let presentation = session
        .mounted
        .current_publication()
        .unwrap()
        .presentation_for_surface(surface)
        .unwrap();
    let focus = session.focus.as_mut().unwrap();
    focus.observe_host_payload(
        &UiHostObservationPayload::Keyboard {
            logical_key: UiHostKey::Tab,
            physical_key: None,
            modifiers: UiHostKeyboardModifiers::default(),
            transition: UiHostKeyTransition::Pressed { repeat: false },
        },
        presentation,
    );
    let scope = focus.default_scope_for_surface(surface).unwrap();
    focus
        .commit_host_traversal(
            scope,
            crate::runtime::focus::UiHostFocusTraversalDirection::Forward,
            false,
        )
        .unwrap();
    focus
        .current_semantic_focus()
        .map(|target| target.mounted_instance())
}

fn project(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    expected: &[(UiMountedInstanceIdentity, u8)],
) -> crate::mounting::UiPreparedMountedFrame {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("focus appearance frame must prepare"));
    let cost = frame.appearance_selection_cost_report();
    assert_eq!(cost.selected_instance_count(), expected.len());
    assert_eq!(cost.materialized_context_count(), expected.len());
    assert_eq!(cost.index_entries_touched(), expected.len());
    if expected.is_empty() {
        frame.assert_no_unpublished_appearance_for_test();
        return frame;
    }
    let output = frame.lower_unpublished_appearance_for_test();
    assert_eq!(output.fragments().len(), expected.len());
    let mut observed = Vec::new();
    for fragment in output.fragments() {
        let UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            successor: Some(receipt),
            ..
        } = fragment.identity()
        else {
            panic!("focus output must carry the mounted successor identity");
        };
        assert_eq!(fragment.work().successor().mechanics().len(), 1);
        let UiMountedAppearanceMechanic::Surface(mechanic) =
            &fragment.work().successor().mechanics()[0]
        else {
            panic!("focus background must lower to a surface");
        };
        let red = expected
            .iter()
            .find(|(instance, _)| *instance == receipt.mounted_instance())
            .expect("unrelated mounted target received focus appearance work")
            .1;
        assert_eq!(
            mechanic.paint(),
            &UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                red, 0, 0, 255
            ]),)
        );
        observed.push(receipt.mounted_instance());
    }
    observed.sort_unstable();
    let mut expected = expected
        .iter()
        .map(|(instance, _)| *instance)
        .collect::<Vec<_>>();
    expected.sort_unstable();
    assert_eq!(observed, expected);
    frame
}

#[path = "appearance_projection_focus_motion_tests.rs"]
mod motion_epoch;

#[path = "appearance_motion_acceptance_tests.rs"]
mod motion_acceptance_tests;
