use worth_ui_dsl::UiAppearanceStateAxis;

#[test]
fn scroll_pose_is_local_idempotent_and_rejects_a_later_invalid_owner_before_effects() {
    let role = super::support::validation_background_role_with_axis(
        super::support::APPEARANCE_TOKEN,
        UiAppearanceStateAxis::Validation,
    );
    let (mut session, host) = super::theme_session(&role);
    let (surface, node) = super::mounting_fixture::mount(&mut session, 1_000);
    let handle = session.mounted_graph_node(node).unwrap();
    let root = session.mounted_instances_for(handle).unwrap()[0];
    let child = session.mount_instance(handle, surface).unwrap();
    let neighbor = session.mount_instance(handle, surface).unwrap();
    let neighbor_child = session.mount_instance(handle, surface).unwrap();
    let retired = session.mount_instance(handle, surface).unwrap();
    session.unmount_instance(retired).unwrap();
    let expected = crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(&mut session, surface, 2, &[(root, None), (child, Some(root)), (neighbor, None), (neighbor_child, Some(neighbor))]);
    let source = super::support::attached_appearance_candidate_submission(
        &session,
        "scroll-geometry",
        "workspace.component.active_session_current",
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    session.classify_observations(admitted).unwrap();
    session.advance_mounted_identity_frame().unwrap();
    host.push_native_display_presented();
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("initial frame"));
    assert!(matches!(
        session.present_prepared_mounted_frame_internal(
            frame,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            1
        ),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    let offset = crate::runtime::scroll::UiScrollOffset::new(
        0,
        8 * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
    )
    .unwrap();
    assert!(!session.mounted.projection_changes_pending());
    assert!(matches!(
        session
            .mounted
            .apply_scroll_geometries(&[(surface, root, offset), (surface, retired, offset)]),
        Err(crate::mounting::UiMountedOccurrenceGeometryDenial::UnknownMountedInstance)
    ));
    assert!(!session.mounted.projection_changes_pending());
    let unchanged = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("denial leaves predecessor"));
    assert_eq!(unchanged.cost_report().changed_mounted_instances(), 0);
    crate::facade::entry::mounted_occurrence_geometry_test_support::assert_prepared_surface_geometry(&session, &unchanged, surface, &expected);
    drop(unchanged);
    session
        .mounted
        .apply_scroll_geometries(&[(surface, root, offset)])
        .unwrap();
    assert!(
        session.mounted.projection_changes_pending(),
        "Scroll must wake native presentation without another input"
    );
    session
        .mounted
        .apply_scroll_geometries(&[(surface, root, offset)])
        .unwrap();
    let shifted = expected
        .into_iter()
        .map(|(instance, bounds)| {
            let next = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
                worth_ui_host_contract::UiMountedCanonicalBoxInput {
                    x: bounds.x(),
                    y: bounds.y() - if instance == child { 8.0 } else { 0.0 },
                    width: bounds.width(),
                    height: bounds.height(),
                    coordinate_space: bounds.coordinate_space(),
                },
            )
            .unwrap();
            (instance, next)
        })
        .collect::<Vec<_>>();
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("scrolled frame"));
    assert_eq!(
        frame.cost_report().changed_mounted_instances(),
        1,
        "only the owner's child changes; the owner and other neighborhood stay fixed"
    );
    crate::facade::entry::mounted_occurrence_geometry_test_support::assert_prepared_surface_geometry(&session, &frame, surface, &shifted);
    drop(frame);
    let _ = session.shutdown();
}
