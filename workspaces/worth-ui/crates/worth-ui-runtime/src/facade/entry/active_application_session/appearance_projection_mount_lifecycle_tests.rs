use super::super::{mounting_fixture, support};
use super::fixture;
use worth_ui_host_contract::*;

#[test]
fn newly_mounted_copy_is_selected_without_source_close_and_survives_abandonment() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role);
    let graph_node = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| {
            node.declaration_identity().authored_semantic_name()
                == format!("component:{}", support::APPEARANCE_NODE_A)
        })
        .unwrap()
        .graph_node_identity();
    mounting_fixture::mount_graph_node(&mut session, 1_000, graph_node);
    let handle = session.mounted_graph_node(graph_node).unwrap();
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
    fixture::close_source(&mut session, &role, "mount-lifecycle-initial");
    session.advance_mounted_identity_frame().unwrap();
    let initial = prepare(&mut session);
    fixture::publish(&mut session, &host, initial, 1);

    let arriving = session.mount_instance(handle, neighbor).unwrap();
    let cancelled = session.mount_instance(handle, neighbor).unwrap();
    session.unmount_instance(cancelled).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        neighbor,
    );
    for _ in 0..2 {
        let frame = prepare(&mut session);
        assert_eq!(
            frame
                .appearance_selection_cost_report()
                .selected_instance_count(),
            1
        );
        let output = frame.lower_unpublished_appearance_for_test();
        assert_eq!(output.fragments().len(), 1);
        let fragment = &output.fragments()[0];
        assert_eq!(fragment.work().successor().semantic_surface(), neighbor);
        assert!(matches!(fragment.work().changes(),
            [UiMountedAppearanceMechanicChange::Insert(UiMountedAppearanceMechanic::Surface(mechanic))]
            if mechanic.node_receipt().mounted_instance() == arriving));
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(&output)
            .unwrap();
        drop(frame);
    }
    let frame = prepare(&mut session);
    let accepted = frame.lower_unpublished_appearance_for_test();
    let UiMountedAppearanceMechanic::Surface(mechanic) =
        &accepted.fragments()[0].work().successor().mechanics()[0]
    else {
        panic!("the newly mounted copy has surface appearance");
    };
    let arriving_receipt = mechanic.node_receipt();
    fixture::publish(&mut session, &host, frame, 2);
    let mut order = session
        .inspect_mounted_identity()
        .mounted_instances()
        .iter()
        .map(|instance| instance.identity())
        .collect::<Vec<_>>();
    order.reverse();
    session.reorder_mounted_instances(&order).unwrap();
    session.advance_mounted_identity_frame().unwrap();
    let unchanged = prepare(&mut session);
    assert_eq!(
        unchanged
            .appearance_selection_cost_report()
            .selected_instance_count(),
        0
    );
    drop(unchanged);
    // The identity-only capsule must still distinguish an actual successor
    // incarnation from the retained entries in the same pending order journal.
    session.unmount_instance(arriving).unwrap();
    let successor = session.mount_instance(handle, neighbor).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        neighbor,
    );
    let frame = prepare(&mut session);
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        1
    );
    let output = frame.lower_unpublished_appearance_for_test();
    let insertions = output
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().changes())
        .filter_map(|change| match change {
            UiMountedAppearanceMechanicChange::Insert(UiMountedAppearanceMechanic::Surface(
                mechanic,
            )) => Some(mechanic.node_receipt().mounted_instance()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(insertions, [successor]);
    let removals = output
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().changes())
        .filter_map(|change| match change {
            UiMountedAppearanceMechanicChange::Remove(
                UiMountedAppearanceMechanicIdentity::Surface(instance),
            ) => Some(*instance),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(removals, [arriving]);
    frame.verify_unpublished_appearance_retirement_denial_and_retry(arriving_receipt);
    drop(frame);
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
