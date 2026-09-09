use worth_ui_host_contract::*;

#[test]
fn mounted_pointer_rows_follow_real_no_role_observation_and_primary_handoff() {
    let (mut session, host, surfaces) = super::mounted_world();
    let main = surfaces[0];
    super::motion(
        &mut session,
        main,
        1,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let frame = prepare(&mut session);
    let initial = frame.lower_unpublished_appearance_for_test();
    let first = pointer(&initial, main);
    assert_eq!(first.pointer(), UiHostPointerIdentity::new(1));
    assert_eq!(first.family(), UiPointerAffordanceFamily::Activation);
    assert_eq!(initial.fragments().len(), 1);
    assert_eq!(frame.pointer_affordance_work().targets_examined, 1);
    assert!(frame.pointer_affordance_work().membership_key_probes > 0);
    assert_transport(&initial, UiMountedAppearanceWorkPosture::Initial, 1);
    super::super::publish(&mut session, &host, frame, 2);
    let unchanged = prepare(&mut session);
    unchanged.assert_no_unpublished_appearance_for_test();
    drop(unchanged);

    super::motion(
        &mut session,
        surfaces[1],
        2,
        2,
        UiHostPointerDeviceKind::Stylus,
        false,
    );
    let frame = prepare(&mut session);
    let neighbor = frame.lower_unpublished_appearance_for_test();
    assert_eq!(neighbor.fragments().len(), 1);
    assert_eq!(
        pointer(&neighbor, surfaces[1]).pointer(),
        UiHostPointerIdentity::new(2)
    );
    super::super::publish(&mut session, &host, frame, 3);

    // Existing pointer 2 becomes primary on main. The original pointer is
    // removed there, and pointer 2 departs its previous surface in the same work.
    super::motion(
        &mut session,
        main,
        3,
        2,
        UiHostPointerDeviceKind::Stylus,
        false,
    );
    let frame = prepare(&mut session);
    let handoff = frame.lower_unpublished_appearance_for_test();
    assert_eq!(handoff.fragments().len(), 2);
    assert_eq!(
        pointer(&handoff, main).pointer(),
        UiHostPointerIdentity::new(2)
    );
    let main_work = handoff
        .fragments()
        .iter()
        .find(|fragment| fragment.work().successor().semantic_surface() == main)
        .unwrap()
        .work();
    assert_eq!(main_work.predecessor(), Some(initial.frame()));
    assert_eq!(main_work.changes().len(), 2);
    assert!(
        matches!(&main_work.changes()[0], UiMountedAppearanceMechanicChange::Remove(
        UiMountedAppearanceMechanicIdentity::Pointer { pointer, .. }
    ) if *pointer == UiHostPointerIdentity::new(1))
    );
    let departed = handoff
        .fragments()
        .iter()
        .find(|fragment| fragment.work().successor().semantic_surface() == surfaces[1])
        .unwrap()
        .work();
    assert!(departed.successor().mechanics().is_empty());
    assert_eq!(departed.changes().len(), 1);
    assert_headless(&handoff);
    super::super::publish(&mut session, &host, frame, 4);

    session
        .update_intent_boolean_fact(
            &super::super::fixture::fact(super::super::fixture::MUTABLE),
            false,
        )
        .unwrap();
    super::close_source(&mut session, "mounted-pointer-readonly");
    let frame = prepare(&mut session);
    let denied = frame.lower_unpublished_appearance_for_test();
    assert_eq!(denied.fragments().len(), 1);
    assert_eq!(
        pointer(&denied, main).family(),
        UiPointerAffordanceFamily::Default
    );
    assert_transport(&denied, UiMountedAppearanceWorkPosture::Delta, 1);
    super::super::publish(&mut session, &host, frame, 5);

    super::motion(
        &mut session,
        main,
        4,
        2,
        UiHostPointerDeviceKind::Stylus,
        false,
    );
    let frame = prepare(&mut session);
    frame.assert_no_unpublished_appearance_for_test();
    super::super::publish(&mut session, &host, frame, 6);

    // No new input/close: candidate membership owns the target's departure.
    session.unmount_instance(first.target()).unwrap();
    let frame = prepare(&mut session);
    let removed = frame.lower_unpublished_appearance_for_test();
    assert_eq!(
        removed.fragments()[0].work().predecessor(),
        Some(denied.frame())
    );
    assert_eq!(removed.fragments().len(), 1);
    assert!(removed.fragments()[0]
        .work()
        .successor()
        .mechanics()
        .is_empty());
    assert_transport(&removed, UiMountedAppearanceWorkPosture::Delta, 1);
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
        .unwrap_or_else(|_| panic!("pointer frame preparation must succeed"))
}

#[test]
fn mounted_pointer_partial_publication_preserves_other_surface_rows() {
    let (mut session, host, surfaces) = super::mounted_world();
    super::motion(
        &mut session,
        surfaces[0],
        1,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let initial = prepare(&mut session);
    let output = initial.lower_unpublished_appearance_for_test();
    let first = pointer(&output, surfaces[0]);
    super::super::publish(&mut session, &host, initial, 2);
    super::motion(
        &mut session,
        surfaces[1],
        2,
        2,
        UiHostPointerDeviceKind::Stylus,
        false,
    );
    let partial = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::exact_surfaces(vec![surfaces[1]]),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("neighbor pointer must prepare independently"));
    let neighbor = partial.lower_unpublished_appearance_for_test();
    assert_eq!(neighbor.fragments().len(), 1);
    assert_eq!(
        pointer(&neighbor, surfaces[1]).pointer(),
        UiHostPointerIdentity::new(2)
    );
    super::super::publish(&mut session, &host, partial, 3);
    let combined = prepare(&mut session);
    combined.assert_no_unpublished_appearance_for_test();
    let reconstructed = combined.lower_reconstructed_pointer_for_test();
    assert_eq!(reconstructed.fragments().len(), 2);
    assert_eq!(pointer(&reconstructed, surfaces[0]), first);
    let _ = session.shutdown();
}

#[test]
fn mounted_pointer_candidates_preserve_retry_and_reject_expired_snapshots() {
    let (mut session, host, surfaces) = super::mounted_world();
    super::motion(
        &mut session,
        surfaces[0],
        1,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let stale = session.pointer_affordance_snapshot.clone().unwrap();
    let abandoned = prepare(&mut session);
    abandoned.verify_pointer_output_retry();
    let target = pointer(
        &abandoned.lower_unpublished_appearance_for_test(),
        surfaces[0],
    )
    .target();
    drop(abandoned);
    assert!(session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .is_none());

    let rejected = prepare(&mut session);
    assert_transport(
        &rejected.lower_unpublished_appearance_for_test(),
        UiMountedAppearanceWorkPosture::Initial,
        1,
    );
    for _ in rejected.surfaces() {
        host.push_rejected();
    }
    assert!(matches!(
        session.present_prepared_mounted_frame_internal(
            rejected,
            UiPresentationDeadline::at_tick(100),
            2,
        ),
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    assert!(session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .is_none());
    super::close_source(&mut session, "pointer-retry-fresh-observation");
    let retry = prepare(&mut session);
    assert_transport(
        &retry.lower_unpublished_appearance_for_test(),
        UiMountedAppearanceWorkPosture::Initial,
        1,
    );
    super::super::publish(&mut session, &host, retry, 3);

    super::motion(
        &mut session,
        surfaces[0],
        2,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let mut fresh = prepare(&mut session);
    assert_eq!(
        fresh.stage_pointer_affordance(Some(&stale), &session.mounted),
        Err(
            crate::mounting::UiMountedFramePreparationDenial::PointerSnapshotTargetUnavailable(
                target
            )
        )
    );
    fresh.assert_no_unpublished_appearance_for_test();
    let reconstruction = fresh.lower_reconstructed_pointer_for_test();
    assert_transport(
        &reconstruction,
        UiMountedAppearanceWorkPosture::Reconstruction,
        1,
    );
    assert_eq!(pointer(&reconstruction, surfaces[0]).target(), target);
    // This is the same mounted clearing operation used by generation cutover.
    fresh.clear_pointer_affordance();
    let removal = fresh.lower_unpublished_appearance_for_test();
    assert_transport(&removal, UiMountedAppearanceWorkPosture::Delta, 1);
    assert!(removal.fragments()[0]
        .work()
        .successor()
        .mechanics()
        .is_empty());
    let _ = session.shutdown();
}

fn pointer(
    output: &UiUnpublishedAppearanceFrameProjection,
    surface: UiSemanticSurfaceIdentity,
) -> UiMountedPointerAffordanceMechanic {
    let fragment = output
        .fragments()
        .iter()
        .find(|fragment| fragment.work().successor().semantic_surface() == surface)
        .unwrap();
    assert!(matches!(
        fragment.identity(),
        UiUnpublishedAppearanceFragmentIdentity::SurfacePointer { .. }
    ));
    let [UiMountedAppearanceMechanic::Pointer(pointer)] = fragment.work().successor().mechanics()
    else {
        panic!("one independent pointer mechanic")
    };
    *pointer
}

fn assert_transport(
    output: &UiUnpublishedAppearanceFrameProjection,
    posture: UiMountedAppearanceWorkPosture,
    changes: usize,
) {
    let work = output.fragments()[0].work();
    assert_eq!(work.posture(), posture);
    assert_eq!(work.changes().len(), changes);
    assert_headless(output);
}

fn assert_headless(output: &UiUnpublishedAppearanceFrameProjection) {
    let transcript =
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    assert_eq!(transcript.fragments().len(), output.fragments().len());
    for fragment in transcript.fragments() {
        assert!(fragment.work().damage().is_empty());
        assert!(fragment.text_candidates().is_empty());
        assert!(fragment
            .presentation_affinity()
            .receipt_affinity()
            .is_none());
    }
}
