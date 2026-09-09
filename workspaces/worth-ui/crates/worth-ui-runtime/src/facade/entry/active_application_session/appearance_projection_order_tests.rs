use super::super::{mounting_fixture, support};
use super::fixture;
use worth_ui_host_contract::*;

#[test]
fn mounted_order_conflict_sees_retained_neighbor_and_preserves_other_surface() {
    let role = fixture::role();
    let (mut session, host) = fixture::session_with_second_order(&role, 65_536);
    let first_node = session
        .graph()
        .node_identities()
        .find(|identity| {
            session
                .graph()
                .lookup()
                .graph_node(*identity)
                .is_some_and(|node| {
                    node.value().declaration_identity().authored_semantic_name()
                        == format!("component:{}", support::APPEARANCE_NODE_A)
                })
        })
        .expect("first authored component exists");
    let (surface, _) = mounting_fixture::mount_graph_node(&mut session, 1_000, first_node);
    let first_handle = session.mounted_graph_node(first_node).unwrap();
    let retained = session.mounted_instances_for(first_handle).unwrap()[0];
    let second_node = session
        .graph()
        .node_identities()
        .find(|identity| {
            session
                .graph()
                .lookup()
                .graph_node(*identity)
                .is_some_and(|node| {
                    node.value().declaration_identity().authored_semantic_name()
                        == format!("component:{}", support::APPEARANCE_NODE_B)
                })
        })
        .expect("second authored component exists");
    let second_handle = session.mounted_graph_node(second_node).unwrap();
    let second = session.mounted_instances_for(second_handle).unwrap()[0];
    session.unmount_instance(second).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(
        &mut session,
        surface,
        2,
        &[(retained, None)],
    );

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
    let unrelated = session.mount_instance(first_handle, neighbor).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        neighbor,
    );
    fixture::close_source(&mut session, &role, "order-initial");
    session.advance_mounted_identity_frame().unwrap();
    let frame = prepare(&mut session);
    let initial = frame.lower_unpublished_appearance_for_test();
    assert_eq!(initial.fragments().len(), 2);
    fixture::publish(&mut session, &host, frame, 1);

    let arriving = session.mount_instance(second_handle, surface).unwrap();
    let overlapping = crate::facade::entry::mounted_occurrence_geometry_test_support::expected_nonoverlapping_bounds(0);
    let mut layout = session.begin_mounted_layout();
    let basis = layout.basis(surface).unwrap();
    layout
        .complete_surface_geometry(crate::mounting::UiMountedSurfaceGeometryBatch::new(
            basis,
            crate::mounting::UiMountedLayoutRevision::new(3).unwrap(),
            crate::facade::entry::mounted_occurrence_geometry_test_support::surface_viewport_bounds(
            ),
            [
                crate::mounting::UiMountedOccurrenceGeometry::surface(retained, overlapping),
                crate::mounting::UiMountedOccurrenceGeometry::surface(arriving, overlapping),
            ],
        ))
        .unwrap();
    let frame = prepare(&mut session);
    frame.verify_retained_order_conflict(surface, retained, arriving);
    drop(frame);
    // No owner/source change is needed, and abandoning preparation must not
    // consume the exact new-mount contribution.
    let frame = prepare(&mut session);
    frame.verify_retained_order_conflict(surface, retained, arriving);
    drop(frame);

    // The same arriving declaration is lawful once the old peer departs in the
    // same batch. The other surface's copy is never selected or removed.
    session.unmount_instance(retained).unwrap();
    let frame = prepare(&mut session);
    let replacement = frame.lower_unpublished_appearance_for_test();
    let mut removed = Vec::new();
    let mut inserted = Vec::new();
    for fragment in replacement.fragments() {
        for change in fragment.work().changes() {
            match change {
                UiMountedAppearanceMechanicChange::Remove(
                    UiMountedAppearanceMechanicIdentity::Surface(id),
                ) => removed.push(*id),
                UiMountedAppearanceMechanicChange::Insert(
                    UiMountedAppearanceMechanic::Surface(m),
                ) => inserted.push(m.node_receipt().mounted_instance()),
                _ => panic!(
                    "replacement contains only the exact departing and arriving surface mechanics"
                ),
            }
        }
    }
    assert_eq!(removed, [retained]);
    assert_eq!(inserted, [arriving]);
    assert!(!removed.contains(&unrelated));
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(&replacement)
        .unwrap();
    fixture::publish(&mut session, &host, frame, 3);
    let _ = session.shutdown();
}

fn prepare(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
) -> crate::mounting::UiPreparedMountedFrame {
    session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("ordinary mounted candidate prepares"))
}
