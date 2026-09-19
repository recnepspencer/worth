#[path = "appearance_projection_selection_candidate_fixture.rs"]
mod candidate_fixture;
#[path = "appearance_projection_selection_fixture.rs"]
mod fixture;
#[path = "appearance_projection_selection_lifecycle.rs"]
mod lifecycle;
use crate::runtime::selection::UiSelectionRequest;
use worth_ui_query_binding::*;

#[test]
fn mounted_selection_keys_share_an_owner_without_crossing_neighborhoods() {
    let (mut query, entities) =
        certification::seeded_collection_projection_workspace_with_item_keys(
            vec![
                ("alpha".into(), "Alpha".into(), 11),
                ("beta".into(), "Beta".into(), 22),
            ],
            certification::WorthUiCollectionProjectionSeedPosture::Complete,
        );
    let domain = query.worth_ui().unwrap();
    let registration =
        UiCollectionProjectionRegistration::text(
            domain.projection_view(fixture::PROJECTION).unwrap(),
            UiProjectionFieldRequirement::identity_id(),
            [UiProjectionFieldRequirement::query_text_status()],
            false,
            false,
        )
        .unwrap()
        .with_unsigned64_application_item_key_field(
            UiProjectionFieldRequirement::collection_item_key(),
        );
    let UiCollectionProjectionBindingAdmission::Ready(binding) = registration.clone().admit(&query)
    else {
        panic!("Query binding")
    };
    let UiCollectionProjectionOpenOutcome::Opened(opened) = binding.open(
        UiCollectionProjectionBudget::new(2, 2, 0, 1024).unwrap(),
        &mut query,
    ) else {
        panic!("Query collection")
    };
    let (mut live, fact) = opened.into_parts();
    let UiProjectionAvailability::Present(UiPresentProjection::Current(value)) =
        fact.availability()
    else {
        panic!("current collection")
    };
    let rows = value
        .rows()
        .iter()
        .map(|row| (row.application_item_key().unwrap().get(), row.row().clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let projection = registration.view().identity().clone();
    let (mut session, host) = fixture::session(registration);
    let item_graph = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| {
            node.declaration_identity()
                .authored_semantic_name()
                .contains(super::support::APPEARANCE_NODE_B)
        })
        .unwrap()
        .graph_node_identity();
    let (surface, item_graph) =
        super::mounting_fixture::mount_graph_node(&mut session, 1000, item_graph);
    let owner_graph = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| {
            node.declaration_identity()
                .authored_semantic_name()
                .contains(super::support::APPEARANCE_NODE_A)
        })
        .unwrap()
        .graph_node_identity();
    let owner_handle = session.mounted_graph_node(owner_graph).unwrap();
    let item_handle = session.mounted_graph_node(item_graph).unwrap();
    let owner = session.mounted_instances_for(owner_handle).unwrap()[0];
    let bootstrap_item = session.mounted_instances_for(item_handle).unwrap()[0];
    session.unmount_instance(bootstrap_item).unwrap();
    let initial_geometry = crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(
        &mut session,
        surface,
        2,
        &[],
    );
    session.advance_mounted_identity_frame().unwrap();
    let initial = fixture::frame(&mut session);
    crate::facade::entry::mounted_occurrence_geometry_test_support::assert_prepared_surface_geometry(
        &session,
        &initial,
        surface,
        &initial_geometry,
    );
    fixture::publish(&mut session, &host, initial, 1);
    fixture::publish_query(
        &mut session,
        &host,
        UiProjectionObservation::Collection(fact.into_observation()),
    );
    let option_a = session
        .current_projection_option(&projection, &rows[&11])
        .unwrap();
    let option_b = session
        .current_projection_option(&projection, &rows[&22])
        .unwrap();

    let first = session.mount_instance(item_handle, surface).unwrap();
    let second = session.mount_instance(item_handle, surface).unwrap();
    let conflicting_owner = session.mount_instance(owner_handle, surface).unwrap();
    let unbound_item = session.mount_instance(item_handle, surface).unwrap();
    let neighbor = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            neighbor,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1000,
                crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let other_owner = session.mount_instance(owner_handle, neighbor).unwrap();
    let other_first = session.mount_instance(item_handle, neighbor).unwrap();
    let other_second = session.mount_instance(item_handle, neighbor).unwrap();
    let _ = crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(
        &mut session,
        surface,
        3,
        &[
            (owner, None),
            (first, Some(owner)),
            (second, Some(owner)),
            (conflicting_owner, None),
            (unbound_item, Some(conflicting_owner)),
        ],
    );
    let neighbor_geometry = crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(
        &mut session,
        neighbor,
        1,
        &[
            (other_owner, None),
            (other_first, Some(other_owner)),
            (other_second, Some(other_owner)),
        ],
    );
    session.advance_mounted_identity_frame().unwrap();
    for (owner, item, option) in [
        (owner, first, &option_a),
        (owner, second, &option_b),
        (other_owner, other_first, &option_a),
        (other_owner, other_second, &option_b),
    ] {
        session
            .bind_selection_item(
                fixture::receipt(&session, owner),
                fixture::receipt(&session, item),
                option.clone(),
            )
            .unwrap();
    }
    lifecycle::assert_invalid_bindings_and_retire_unbound(
        &mut session,
        conflicting_owner,
        second,
        option_b.clone(),
        unbound_item,
        option_a.clone(),
    );
    let surface_geometry = crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        surface,
    );
    fixture::close(&mut session, "selection-bound");
    let frame = fixture::frame(&mut session);
    crate::facade::entry::mounted_occurrence_geometry_test_support::assert_prepared_surface_geometry(
        &session,
        &frame,
        surface,
        &surface_geometry,
    );
    crate::facade::entry::mounted_occurrence_geometry_test_support::assert_prepared_surface_geometry(
        &session,
        &frame,
        neighbor,
        &neighbor_geometry,
    );
    fixture::assert_output(
        &frame,
        &[
            (first, 10),
            (second, 10),
            (other_first, 10),
            (other_second, 10),
        ],
    );
    fixture::publish(&mut session, &host, frame, 1);
    let a = session.mounted.selection_mapping_for_item(first).unwrap();
    let b = session.mounted.selection_mapping_for_item(second).unwrap();
    assert_eq!(a.owner, b.owner);
    assert_eq!(a.incarnation, b.incarnation);
    assert_ne!(a.key, b.key);
    for (step, (request, expected)) in [
        (UiSelectionRequest::SelectSingle(a.key), vec![(first, 50)]),
        (
            UiSelectionRequest::Add(b.key),
            vec![(first, 30), (second, 40)],
        ),
        (
            UiSelectionRequest::Add(a.key),
            vec![(first, 50), (second, 20)],
        ),
        (UiSelectionRequest::Add(a.key), vec![]),
    ]
    .into_iter()
    .enumerate()
    {
        session
            .selection
            .as_mut()
            .unwrap()
            .apply(a.owner, a.incarnation, request)
            .unwrap();
        fixture::close(&mut session, &format!("selection-state-{step}"));
        let frame = fixture::frame(&mut session);
        fixture::assert_output(&frame, &expected);
        fixture::publish(&mut session, &host, frame, 3);
    }
    use worth_ui_dsl::UiAppearanceAxisClass as Class;
    let selected = [
        (first, Some(Class::SelectedAnchorCursor)),
        (second, Some(Class::SelectionSelected)),
        (other_first, Some(Class::SelectionUnselected)),
        (other_second, Some(Class::SelectionUnselected)),
    ];
    let snapshot = session
        .appearance_owner_snapshot_for_test()
        .unwrap()
        .selection()
        .unwrap()
        .clone();
    let prior_output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .cloned();
    certification::update_projection_identity(&mut query, entities[1].clone(), "00-beta");
    let UiCollectionProjectionRefreshOutcome::Applied(reordered) =
        live.refresh(&mut query).unwrap()
    else {
        panic!("real reorder delivers")
    };
    let reordered = reordered.into_fact();
    let slot = option_a.owner_revision().slot();
    let mut candidate = candidate_fixture::collection_candidate(
        &mut session,
        reordered.intent_input_transition(slot),
    );
    candidate_fixture::assert_candidate(&mut candidate, &selected, snapshot.owner_revision());
    drop(candidate);
    assert_eq!(
        session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .cloned(),
        prior_output
    );
    assert_eq!(
        session.mounted.selection_mapping_for_item(first).unwrap(),
        a
    );
    fixture::publish_query(
        &mut session,
        &host,
        UiProjectionObservation::Collection(reordered.into_observation()),
    );
    let retained = fixture::frame(&mut session);
    assert_eq!(
        retained
            .appearance_selection_cost_report()
            .selected_instance_count(),
        0,
        "the next Retain candidate must not inherit the previous input change scope"
    );
    drop(retained);

    let current_output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .cloned();
    let current_owner = session
        .selection
        .as_ref()
        .unwrap()
        .appearance_owner_snapshot();
    certification::remove_projection_entity(&mut query, entities[0].clone());
    let UiCollectionProjectionRefreshOutcome::Applied(removed) = live.refresh(&mut query).unwrap()
    else {
        panic!("real removal delivers")
    };
    let removed = removed.into_fact();
    let mut candidate = candidate_fixture::collection_candidate(
        &mut session,
        removed.intent_input_transition(slot),
    );
    assert_eq!(
        session
            .mounted
            .selection_mapping_for_prepared_item(first, &candidate),
        Err(crate::mounting::UiMountedSelectionBindingDenial::OptionNotCurrent)
    );
    assert_eq!(
        session
            .mounted
            .selection_mapping_for_prepared_item(second, &candidate)
            .unwrap(),
        b
    );
    candidate_fixture::assert_candidate(
        &mut candidate,
        &[
            (first, None),
            (second, Some(Class::SelectionSelected)),
            (other_first, None),
            (other_second, Some(Class::SelectionUnselected)),
        ],
        snapshot.owner_revision(),
    );
    drop(candidate);
    for (capacity, merge) in [(slot.index() + 1, false), (slot.index() + 2, true)] {
        let mut candidate = candidate_fixture::empty_replacement(&mut session, capacity, merge);
        candidate_fixture::assert_candidate(
            &mut candidate,
            &[
                (first, None),
                (second, None),
                (other_first, None),
                (other_second, None),
            ],
            snapshot.owner_revision(),
        );
    }
    assert_eq!(
        session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .cloned(),
        current_output
    );
    assert!(
        session
            .selection
            .as_ref()
            .unwrap()
            .appearance_owner_snapshot()
            == current_owner,
        "abandoned collection candidates preserve the actual Selection owner"
    );
    assert_eq!(
        session.mounted.selection_mapping_for_item(first).unwrap(),
        a
    );
    assert!(
        session
            .appearance_owner_snapshot_for_test()
            .unwrap()
            .selection()
            .unwrap()
            .changes_since(&snapshot)
            .is_empty(),
        "candidate collection changes do not mutate sealed Selection"
    );

    lifecycle::assert_surface_local_retirement(
        &mut session,
        owner,
        first,
        second,
        other_first,
        other_second,
        option_a,
    );
    assert!(matches!(
        live.close(&mut query),
        UiLiveCollectionProjectionCloseOutcome::Closed(_)
    ));
    let _ = session.shutdown();
}
