use worth_ui_dsl::UiAppearanceStateAxis;

#[test]
fn occurrence_geometry_update_is_local_and_identical_geometry_suppresses_work() {
    let role = super::support::validation_background_role_with_axis(
        super::support::APPEARANCE_TOKEN,
        UiAppearanceStateAxis::Validation,
    );
    let (mut session, host) = super::theme_session(&role);
    let (surface, graph_node) = super::mounting_fixture::mount(&mut session, 1_000);
    let handle = session.mounted_graph_node(graph_node).unwrap();
    let root = session.mounted_instances_for(handle).unwrap()[0];
    let child = session.mount_instance(handle, surface).unwrap();
    let other_root = session.mount_instance(handle, surface).unwrap();
    let other_child = session.mount_instance(handle, surface).unwrap();
    let hierarchy = [
        (root, None),
        (child, Some(root)),
        (other_root, None),
        (other_child, Some(other_root)),
    ];
    let initial = crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(
        &mut session,
        surface,
        2,
        &hierarchy,
    );
    let source = super::support::attached_appearance_candidate_submission(
        &session,
        "occurrence-geometry-initial",
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
        .unwrap_or_else(|_| panic!("initial occurrence frame prepares"));
    crate::facade::entry::mounted_occurrence_geometry_test_support::assert_prepared_surface_geometry(
        &session, &frame, surface, &initial,
    );
    let initial_projection = frame.projection_rc_for_test();
    let retained_allocations = [other_root, other_child].map(|instance| {
        (
            instance,
            initial_projection
                .appearance_allocation_for_test(instance)
                .unwrap(),
        )
    });
    assert!(matches!(
        session.present_prepared_mounted_frame_internal(
            frame,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            1,
        ),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));

    let stale = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("stale occurrence candidate prepares"));
    let calls_before = host.presentation_calls();
    crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(
        &mut session,
        surface,
        3,
        &hierarchy,
    );
    drop(stale);
    let equal = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("equal occurrence geometry prepares without invalidation"));
    assert_eq!(equal.cost_report().changed_mounted_instances(), 0);
    assert!(equal.appearance_invalidation_batch().is_none());
    drop(equal);
    assert_eq!(host.presentation_calls(), calls_before);

    let shifted = crate::facade::entry::mounted_occurrence_geometry_test_support::install_shifted_surface_geometry(
        &mut session,
        surface,
        4,
        &hierarchy,
        root,
        4.0,
    );
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("shifted occurrence frame prepares"));
    assert_eq!(frame.cost_report().changed_mounted_instances(), 2);
    let invalidation = frame
        .appearance_invalidation_batch()
        .expect("geometry changes select exact mounted appearance consumers");
    assert_eq!(
        invalidation.mounted_consumers(),
        &[(graph_node, root), (graph_node, child)]
    );
    crate::facade::entry::mounted_occurrence_geometry_test_support::assert_prepared_surface_geometry(
        &session, &frame, surface, &shifted,
    );
    let output = frame.lower_unpublished_appearance_for_test();
    assert_eq!(output.fragments().len(), 2);
    for instance in [root, child] {
        let fragment = output
            .fragments()
            .iter()
            .find(|fragment| {
                matches!(
                    fragment.identity(),
                    worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                        successor: Some(receipt),
                        ..
                    } if receipt.mounted_instance() == instance
                )
            })
            .expect("only the shifted subtree emits appearance work");
        let before = initial.iter().find(|row| row.0 == instance).unwrap().1;
        let after = shifted.iter().find(|row| row.0 == instance).unwrap().1;
        assert_eq!(fragment.work().damage().len(), 1);
        let damage = fragment.work().damage()[0];
        assert_eq!(damage.x(), (before.x() * 1_000.0) as i32);
        assert_eq!(damage.y(), (before.y() * 1_000.0) as i32);
        assert_eq!(
            damage.width(),
            ((after.x() + after.width() - before.x()) * 1_000.0) as u32
        );
        assert_eq!(damage.height(), (before.height() * 1_000.0) as u32);
    }
    let unchanged = initial
        .iter()
        .filter(|(instance, _)| *instance != root && *instance != child)
        .collect::<Vec<_>>();
    for (instance, bounds) in unchanged {
        assert_eq!(
            shifted.iter().find(|row| row.0 == *instance).unwrap().1,
            *bounds
        );
    }
    let shifted_projection = frame.projection_rc_for_test();
    for (instance, allocation) in retained_allocations {
        assert_eq!(
            shifted_projection
                .appearance_allocation_for_test(instance)
                .unwrap(),
            allocation
        );
    }
    let _ = session.shutdown();
}

#[test]
fn surface_rebind_preserves_occurrence_bounds_and_rebases_coordinate_ownership() {
    let role = super::support::validation_background_role_with_axis(
        super::support::APPEARANCE_TOKEN,
        UiAppearanceStateAxis::Validation,
    );
    let (mut session, host) = super::theme_session(&role);
    let (surface, graph_node) = super::mounting_fixture::mount(&mut session, 1_000);
    let handle = session.mounted_graph_node(graph_node).unwrap();
    let root = session.mounted_instances_for(handle).unwrap()[0];
    let child = session.mount_instance(handle, surface).unwrap();
    let hierarchy = [(root, None), (child, Some(root))];
    let expected = crate::facade::entry::mounted_occurrence_geometry_test_support::install_nonoverlapping_surface_geometry(
        &mut session,
        surface,
        2,
        &hierarchy,
    );
    let source = super::support::attached_appearance_candidate_submission(
        &session,
        "occurrence-geometry-rebind",
        "workspace.component.active_session_current",
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    session.classify_observations(admitted).unwrap();
    session.advance_mounted_identity_frame().unwrap();
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("initial rebind frame prepares"));
    host.push_native_display_presented();
    assert!(matches!(
        session.present_prepared_mounted_frame_internal(
            frame,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            1,
        ),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));

    let prior = session
        .inspect_mounted_identity()
        .surface_bindings()
        .iter()
        .find(|binding| binding.semantic_surface_identity() == surface)
        .copied()
        .unwrap();
    let rebound = session
        .rebind_host_surface(
            prior.binding_generation(),
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            prior.profile(),
        )
        .unwrap();
    assert_ne!(rebound.binding_generation(), prior.binding_generation());
    let replacements = [crate::mounting::UiMountedSurfaceReconciliationBinding::new(
        prior.binding_generation(),
        rebound.binding_generation(),
    )];
    let frame = session
        .prepare_mounted_reconstruction_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            &replacements,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("rebound occurrence frame prepares"));
    crate::facade::entry::mounted_occurrence_geometry_test_support::assert_prepared_surface_geometry(
        &session, &frame, surface, &expected,
    );
    host.push_native_display_presented();
    let outcome = session
        .present_prepared_mounted_frame_for_reconciliation(
            frame,
            &replacements,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(200),
            2,
        )
        .unwrap();
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {}
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            panic!(
                "rebound occurrence admission denied: {:?}",
                rejection.denial()
            )
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("rebound occurrence host rejected")
        }
        _ => panic!("rebound occurrence frame did not publish"),
    }
    let frame = session.current_mounted_projection_rc_for_test().unwrap();
    for (instance, bounds) in &expected {
        let worth_ui_host_contract::UiMountedAllocationProjection::Known {
            bounds: projected,
            basis,
        } = frame.appearance_allocation_for_test(*instance).unwrap()
        else {
            panic!("completed occurrence geometry remains available after rebind")
        };
        assert_eq!(projected, *bounds);
        let incarnation = session
            .mounted
            .current_mounted_identity_basis(*instance)
            .unwrap()
            .mount_incarnation();
        assert_eq!(
            basis.coordinate_ownership(),
            rebound.binding_generation().diagnostic_value() ^ incarnation.diagnostic_value()
        );
    }
    let _ = session.shutdown();
}
