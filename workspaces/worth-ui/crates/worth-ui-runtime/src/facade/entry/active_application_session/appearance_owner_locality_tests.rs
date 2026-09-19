use worth_ui_host_contract::*;

#[test]
fn pointer_owner_invalidation_preserves_mounted_neighborhood_and_surface_scope() {
    let role = super::fixture::role();
    let declarations = super::support::appearance_fixture(&role).with_component_appearance_role(
        "appearance/consumer",
        super::support::APPEARANCE_NODE_B,
        worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
            role.role().clone(),
            role.revision(),
        ),
    );
    let (mut session, host) = super::fixture::session_with_declarations(&role, declarations);
    let (surface, graph_node) = super::mounting_fixture::mount(&mut session, 1_000);
    let node = session.mounted_graph_node(graph_node).unwrap();
    let mut unrelated = Vec::new();
    for _ in 0..2 {
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
        unrelated.push(session.mount_instance(node, neighbor).unwrap());
        crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
            &mut session,
            neighbor,
        );
    }
    close_source(&mut session, &role, "appearance-neighborhood-initial");
    session.advance_mounted_identity_frame().unwrap();
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("initial neighborhood frame must prepare"));
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        4
    );
    for _ in frame.surfaces() {
        host.push_native_display_presented();
    }
    assert!(matches!(
        session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(100),
            1,
        ),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    let presentation = session
        .mounted
        .current_publication()
        .unwrap()
        .presentation_for_surface(surface)
        .unwrap();
    let hit = session
        .mounted
        .interaction_hit_test_basis(presentation)
        .unwrap();
    let row = hit
        .rows()
        .iter()
        .find(|row| {
            session
                .mounted
                .current_mounted_identity_basis(row.mounted_instance())
                .is_some_and(|basis| basis.graph_node_identity() == graph_node)
        })
        .expect("real mounted target for the repeated declaration");
    let target = row.mounted_instance();
    assert!(!unrelated.contains(&target));
    let inside = interior(*row);
    let peer_row = hit
        .rows()
        .iter()
        .find(|row| row.mounted_instance() != target)
        .unwrap();
    let peer = peer_row.mounted_instance();
    let peer_position = interior(*peer_row);
    let batch = super::pointer_batch(
        session.host_session.identity().as_u64(),
        presentation,
        1,
        UiHostPointerIdentity::new(1),
        inside,
        None,
        false,
    );
    let ingress = session.admit_host_interaction_batch(batch);
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = ingress
    else {
        panic!("neighborhood motion must reach the owner: {ingress:?}");
    };
    assert!(receipt.pointer_presence_denials().is_empty());
    assert_eq!(receipt.pointer_presence_transitions().len(), 1);
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_pointer_presence_transition(receipt.pointer_presence_transitions()[0].clone())
        .unwrap();
    let observations = turn.seal().unwrap();
    session.classify_observations(observations).unwrap();
    let pending_revision = session
        .presentation
        .appearance_invalidation_batch()
        .unwrap()
        .revision();
    let same_target = session.admit_host_interaction_batch(super::pointer_batch(
        session.host_session.identity().as_u64(),
        presentation,
        2,
        UiHostPointerIdentity::new(1),
        inside,
        None,
        false,
    ));
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(same_target) =
        same_target
    else {
        panic!("same-target motion must reach the owner");
    };
    assert!(same_target.pointer_presence_transitions().is_empty());
    close_source(&mut session, &role, "appearance-neighborhood-same-target");
    assert_eq!(
        session
            .presentation
            .appearance_invalidation_batch()
            .unwrap()
            .revision(),
        pending_revision,
        "sequence-only motion must queue no additional appearance work"
    );
    let frame = scoped_frame(&mut session, &[target], (surface, 1, target));
    publish(&mut session, &host, frame, 2);

    // A different pointer displaces the primary. Its raw previous target is
    // absent, but both the displaced and newly hovered consumers must update.
    admit_motion(&mut session, surface, 3, 2, peer_position);
    let frame = scoped_frame(&mut session, &[target, peer], (surface, 2, peer));
    publish(&mut session, &host, frame, 3);

    // The non-primary first pointer moves to another surface. The second
    // pointer remains primary on the original surface and receives no work.
    let neighbor = session
        .mounted
        .current_mounted_identity_basis(unrelated[0])
        .unwrap()
        .semantic_surface_identity();
    admit_motion(&mut session, neighbor, 4, 1, inside);
    let frame = scoped_frame(&mut session, &[unrelated[0]], (neighbor, 1, unrelated[0]));
    drop(frame);
    let pending_revision = session
        .presentation
        .appearance_invalidation_batch()
        .unwrap()
        .revision();

    // Pressed changes use the same mounted scope without selecting the
    // other pointer's hovered neighborhood or another copy of this role.
    // Keep the presentation unchanged. The queued revision must advance
    // for Pressed even though the earlier Hover target is already pending.
    let presentation = session
        .mounted
        .current_publication()
        .unwrap()
        .presentation_for_surface(neighbor)
        .unwrap();
    let ingress = session.admit_host_interaction_batch(super::pointer_batch(
        session.host_session.identity().as_u64(),
        presentation,
        5,
        UiHostPointerIdentity::new(1),
        inside,
        Some(UiHostPointerButtonTransition::Pressed),
        true,
    ));
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = ingress
    else {
        panic!("neighborhood press must reach the owner");
    };
    assert!(receipt.pointer_presence_denials().is_empty());
    assert!(receipt.pointer_presence_transitions().is_empty());
    let pressed = session.interaction.pressed_appearance_snapshot();
    assert_eq!(pressed.postures().len(), 1);
    assert_eq!(pressed.postures()[0].target(), unrelated[0]);
    assert_eq!(
        pressed.postures()[0].class(),
        crate::runtime::interaction::gesture::UiPressedAppearanceClass::ArmedInside,
    );
    close_source(&mut session, &role, "appearance-neighborhood-pressed");
    assert_eq!(
        session
            .presentation
            .appearance_invalidation_batch()
            .unwrap()
            .revision(),
        pending_revision + 1,
        "Pressed must queue work independently of unchanged Hover"
    );
    let frame = scoped_frame(&mut session, &[unrelated[0]], (neighbor, 1, unrelated[0]));
    drop(frame);
    let _ = session.shutdown();
}

fn scoped_frame(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    expected: &[UiMountedInstanceIdentity],
    expected_pointer: (UiSemanticSurfaceIdentity, u64, UiMountedInstanceIdentity),
) -> crate::mounting::UiPreparedMountedFrame {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("owner-local appearance frame must prepare"));
    let cost = frame.appearance_selection_cost_report();
    assert_eq!(
        cost.selected_instance_count(),
        expected.len(),
        "one pointer must not select copies on unrelated surfaces"
    );
    assert_eq!(cost.materialized_context_count(), expected.len());
    assert_eq!(cost.index_entries_touched(), expected.len());
    let output = frame.lower_unpublished_appearance_for_test();
    assert_eq!(
        output.fragments().len(),
        expected.len() + 1,
        "only changed nodes and the independently scoped pointer may emit work"
    );
    let mut pointers = Vec::new();
    let mut changed = output
        .fragments()
        .iter()
        .filter_map(|fragment| match fragment.identity() {
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                successor: Some(receipt),
                ..
            } => Some(receipt.mounted_instance()),
            UiUnpublishedAppearanceFragmentIdentity::SurfacePointer { surface, pointer } => {
                let [UiMountedAppearanceMechanic::Pointer(mechanic)] =
                    fragment.work().successor().mechanics()
                else {
                    panic!("pointer output must contain exactly its independent mechanic");
                };
                assert_eq!(mechanic.surface(), surface);
                assert_eq!(mechanic.pointer(), pointer);
                pointers.push((surface, pointer, mechanic.target()));
                None
            }
            _ => panic!("unexpected appearance fragment family"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        pointers,
        vec![(
            expected_pointer.0,
            UiHostPointerIdentity::new(expected_pointer.1),
            expected_pointer.2
        )]
    );
    changed.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(changed, expected);
    frame
}

fn interior(row: crate::mounting::UiPresentedHitTestRow) -> UiHostSurfacePosition {
    let bounds = row.bounds();
    let clip = row.clip_bounds();
    UiHostSurfacePosition::viewport_logical(
        ((bounds.x().max(clip.x()) + (bounds.x() + bounds.width()).min(clip.x() + clip.width()))
            * 500.0) as i64,
        ((bounds.y().max(clip.y()) + (bounds.y() + bounds.height()).min(clip.y() + clip.height()))
            * 500.0) as i64,
    )
}

fn admit_motion(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    sequence: u64,
    pointer: u64,
    position: UiHostSurfacePosition,
) {
    let presentation = session
        .mounted
        .current_publication()
        .unwrap()
        .presentation_for_surface(surface)
        .unwrap();
    let batch = super::pointer_batch(
        session.host_session.identity().as_u64(),
        presentation,
        sequence,
        UiHostPointerIdentity::new(pointer),
        position,
        None,
        false,
    );
    let ingress = session.admit_host_interaction_batch(batch);
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = ingress
    else {
        panic!("neighborhood motion must reach the owner: {ingress:?}");
    };
    assert!(receipt.pointer_presence_denials().is_empty());
    assert_eq!(receipt.pointer_presence_transitions().len(), 1);
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_pointer_presence_transition(receipt.pointer_presence_transitions()[0].clone())
        .unwrap();
    let observations = turn.seal().unwrap();
    session.classify_observations(observations).unwrap();
}

fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    frame: crate::mounting::UiPreparedMountedFrame,
    now: u64,
) {
    for _ in frame.surfaces() {
        host.push_native_display_settled_without_effects();
    }
    assert!(matches!(
        session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(100),
            now
        ),
        crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
    ));
}

fn close_source(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    source_name: &str,
) {
    let mut module =
        worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
            .with_appearance_role(role.clone());
    for component in [
        super::support::APPEARANCE_NODE_A,
        super::support::APPEARANCE_NODE_B,
    ] {
        module = module
            .with_component_appearance_role(
                component,
                worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
                    role.role().clone(),
                    role.revision(),
                ),
            )
            .unwrap();
    }
    let candidate =
        crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
            crate::runtime::WorthUiSourceProvider::rust_authored(source_name)
                .with_rust_authored_input(
                    worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([module]),
                ),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(
                source_name,
            )],
            session.capabilities(),
        );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    session.classify_observations(observations).unwrap();
}
