use super::{fixture, prepare, set_text, text_contract};
use worth_ui_host_contract::*;

#[test]
fn text_publication_on_one_surface_keeps_other_copy_pending() {
    let role = fixture::foreground_role();
    let (mut session, host) = fixture::session_with_text(&role, 65_537, Some(text_contract()));
    let (first, _) = super::super::super::mounting_fixture::mount(&mut session, 1_000);
    let authored = format!(
        "component:{}",
        super::super::super::support::APPEARANCE_NODE_A
    );
    let graph_node = session
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.declaration_identity().authored_semantic_name() == authored)
        .unwrap()
        .graph_node_identity();
    let second = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            second,
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
    let second_instance = session.mount_instance(node, second).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        second,
    );
    set_text(&mut session, 0, "AB");
    fixture::close_source(&mut session, &role, "text-publication-surfaces");
    let initial = prepare(&mut session);
    let output = initial.lower_unpublished_appearance_for_test();
    assert_eq!(foreground_count(&output), 2);
    fixture::publish(&mut session, &host, initial, 1);

    set_text(&mut session, 1, "");
    let partial = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::exact_surfaces(vec![first]),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("single-surface content prepares"));
    super::assert_foreground_removed(&partial);
    fixture::publish(&mut session, &host, partial, 2);

    let remaining = prepare(&mut session);
    let output = remaining.lower_unpublished_appearance_for_test();
    assert_eq!(foreground_count(&output), 0);
    assert!(output.fragments().iter().any(|fragment| {
        fragment.work().changes().iter().any(|change| {
            matches!(change,
                UiMountedAppearanceMechanicChange::Remove(
                    UiMountedAppearanceMechanicIdentity::TextForeground { target, .. }
                ) if *target == second_instance
            )
        })
    }));
    fixture::publish(&mut session, &host, remaining, 3);
    let unchanged = prepare(&mut session);
    unchanged.assert_no_unpublished_appearance_for_test();
    drop(unchanged);
    let _ = session.shutdown();
}

fn foreground_count(output: &UiUnpublishedAppearanceFrameProjection) -> usize {
    output
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().successor().mechanics())
        .filter(|mechanic| matches!(mechanic, UiMountedAppearanceMechanic::TextForeground(_)))
        .count()
}
