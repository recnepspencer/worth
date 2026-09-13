use super::*;

#[test]
fn source_replacement_clears_pressed_before_first_successor_paint() {
    replacement_journey(false, false);
}

#[test]
fn rejected_pressed_replacement_preserves_owner_and_paint_then_retries_prepared_results() {
    replacement_journey(true, false);
}

#[test]
fn retained_source_successor_preserves_pressed_owner_and_next_public_frame() {
    replacement_journey(false, true);
}

fn replacement_journey(reject_first: bool, retain_generation: bool) {
    let role = fixture::role();
    let successor = fixture::successor_role(&role);
    let (mut session, host) = fixture::session(&role);
    let (surface, graph_node) = mounting_fixture::mount(&mut session, 1_000);
    close_source_turn(&mut session, &role, "pressed-replacement-initial");
    session.advance_mounted_identity_frame().unwrap();
    publish(&mut session, &host, 1, 10, None);
    let presentation = session
        .mounted
        .current_publication()
        .unwrap()
        .presentation_for_surface(surface)
        .unwrap();
    let instance = session
        .inspect_mounted_identity()
        .mounted_instances()
        .iter()
        .find(|row| row.graph_node_identity() == graph_node)
        .unwrap()
        .identity();
    let batch = pointer_batch(
        session.host_session.identity().as_u64(),
        presentation,
        1,
        UiHostPointerIdentity::new(1),
        inside_position(&session, presentation, instance),
        Some(UiHostPointerButtonTransition::Pressed),
        true,
    );
    assert!(matches!(
        session.admit_host_interaction_batch(batch),
        crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(_)
    ));
    assert_eq!(
        session
            .interaction
            .pressed_appearance_snapshot()
            .postures()
            .len(),
        1
    );
    assert_eq!(
        session.interaction.pressed_appearance_snapshot().postures()[0].target(),
        instance
    );
    close_source_turn(&mut session, &role, "pressed-replacement-armed");
    publish(&mut session, &host, 2, 30, Some((surface, instance, true)));

    if retain_generation {
        let pressed = session.interaction.pressed_appearance_snapshot();
        let previous = session.active_generation_identity();
        let source =
            support::appearance_candidate_submission(&session, "pressed-equal-source", Some(&role));
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(source).unwrap();
        let observations = turn.seal().unwrap();
        let crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) =
            session.classify_observations(observations).unwrap()
        else {
            panic!("unchanged role must retain its owner lifecycle");
        };
        let plan = session
            .compile_preservation_rebind(
                evidence,
                crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
            )
            .unwrap();
        let calls = host.presentation_calls();
        let outcome = session
            .prepare_rebind(
                plan,
                crate::runtime::rebind::UiRebindExecutionRequest::new(3),
            )
            .unwrap()
            .execute(3);
        assert!(matches!(
            outcome,
            crate::runtime::rebind::UiRebindOutcome::Published(_)
        ));
        drop(outcome);
        assert_eq!(
            host.presentation_calls(),
            calls,
            "equivalent owner exports require no paint"
        );
        assert_ne!(session.active_generation_identity(), previous);
        assert_eq!(session.interaction.pressed_appearance_snapshot(), pressed);
        let owner = session.appearance_owner_snapshot.as_ref().unwrap();
        assert_eq!(owner.generation(), &session.active_generation_identity());
        assert_eq!(owner.pressed(), Some(&pressed));
        host.push_native_display_settled_without_effects();
        let request = session.mounted_frame_request();
        let outcome = session
            .execute_mounted_frame(
                request,
                worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
                4,
                |_| {},
            )
            .unwrap_or_else(|_| panic!("retained Pressed successor must prepare publicly"));
        assert!(matches!(
            outcome,
            crate::mounting::UiMountedFrameOutcome::Published(_)
                | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        ));
        drop(outcome);
        let current = session.interaction.pressed_appearance_snapshot();
        assert_eq!(current.owner_revision(), pressed.owner_revision());
        assert_eq!(current.postures().len(), 1);
        let posture = current.postures()[0];
        assert!(posture.appearance_dependency_eq(pressed.postures()[0]));
        assert_eq!(
            posture.presentation(),
            session
                .mounted
                .current_publication()
                .unwrap()
                .presentation_for_surface(surface)
                .unwrap()
        );
        let release = pointer_batch(
            session.host_session.identity().as_u64(),
            posture.presentation(),
            2,
            posture.pointer(),
            inside_position(&session, posture.presentation(), instance),
            Some(UiHostPointerButtonTransition::Released),
            false,
        );
        assert!(matches!(
            session.admit_host_interaction_batch(release),
            crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(_)
        ));
        close_source_turn(&mut session, &role, "retained-successor-release");
        assert_eq!(
            session
                .appearance_owner_snapshot
                .as_ref()
                .unwrap()
                .generation(),
            &session.active_generation_identity()
        );
        host.push_native_display_settled_without_effects();
        let request = session.mounted_frame_request();
        let outcome = session
            .execute_mounted_frame(request, UiPresentationDeadline::at_tick(100), 5, |_| {})
            .unwrap_or_else(|_| panic!("successor axis change must resolve publicly"));
        assert!(matches!(
            outcome,
            crate::mounting::UiMountedFrameOutcome::Published(_)
        ));
        drop(outcome);
        assert!(session
            .interaction
            .pressed_appearance_snapshot()
            .postures()
            .is_empty());
        // Released while inside selects Hover (20), not retained ArmedInside (30).
        test_support::assert_unpublished_surface_with_pointer(
            session
                .mounted
                .current_unpublished_appearance()
                .unwrap()
                .unwrap(),
            [20, 0, 0, 255],
            None,
        );
        let _ = session.shutdown();
        return;
    }

    let source =
        support::appearance_candidate_submission(&session, "pressed-role-edit", Some(&successor));
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let observations = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::Changed(changed) =
        session.classify_observations(observations).unwrap()
    else {
        panic!("real role attachment edit must change meaning");
    };
    let lifecycle = session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = session
        .compile_rebind_plan(
            lifecycle,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let predecessor = session.mounted.current_publication().unwrap().clone();
    let pressed = session.interaction.pressed_appearance_snapshot();
    if reject_first {
        host.push_rejected();
    } else {
        host.push_native_display_settled_without_effects();
    }
    let retry = {
        let outcome = session
            .prepare_rebind(
                plan,
                crate::runtime::rebind::UiRebindExecutionRequest::new(3),
            )
            .unwrap()
            .execute(3);
        match outcome {
            crate::runtime::rebind::UiRebindOutcome::Published(_) if !reject_first => None,
            crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial)
                if reject_first =>
            {
                assert_eq!(
                    denial.cause(),
                    crate::runtime::rebind::UiRebindDenialCause::HostRejectedBeforeEffects
                );
                Some(
                    denial
                        .detach_retry_for_native()
                        .unwrap_or_else(|_| panic!("host rejection retains exact prepared retry")),
                )
            }
            crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) => {
                panic!("stateful replacement denied: {:?}", denial.cause())
            }
            _ => panic!("stateful source replacement must reach the requested host outcome"),
        }
    };
    if let Some(retry) = retry {
        assert_eq!(session.mounted.current_publication(), Some(&predecessor));
        assert_eq!(session.interaction.pressed_appearance_snapshot(), pressed);
        test_support::assert_unpublished_surface_with_pointer(
            session
                .mounted
                .current_unpublished_appearance()
                .unwrap()
                .unwrap(),
            [30, 0, 0, 255],
            Some((surface, instance, true)),
        );
        host.push_native_display_settled_without_effects();
        assert!(matches!(
            retry.rebase_content_and_retry(&mut session, 4).unwrap(),
            crate::runtime::rebind::UiRebindOutcome::Published(_)
        ));
    }
    // Replacement cancels both pointer owners: Outside + Idle selects red=10;
    // carrying ArmedInside would select red=30, and retaining Hover selects 20.
    assert!(session
        .interaction
        .pressed_appearance_snapshot()
        .postures()
        .is_empty());
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    test_support::assert_unpublished_surface_with_pointer(
        output,
        [10, 0, 0, 255],
        Some((surface, instance, false)),
    );
    let _ = session.shutdown();
}

#[test]
fn required_owner_snapshot_cannot_be_deferred_into_first_publication() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role);
    let _ = mounting_fixture::mount(&mut session, 1_000);
    close_source_turn(&mut session, &role, "required-owner-preparation");
    let snapshot = session.appearance_owner_snapshot.take().unwrap();
    let pending = session
        .presentation
        .appearance_invalidation_batch()
        .unwrap();
    let denied = session.prepare_mounted_frame_with_application_presentation(
        crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
        |_| {},
    );
    match denied {
        Err(crate::facade::entry::WorthUiMountedFrameExecutionStop::Preparation(denial)) => {
            assert_eq!(*denial, crate::mounting::UiMountedFramePreparationDenial::AppearanceOwnerSnapshotUnavailable);
        }
        _ => panic!("missing required owner preparation must deny"),
    }
    assert_eq!(host.presentation_calls(), 0);
    assert!(session.mounted.current_publication().is_none());
    let still_pending = session
        .presentation
        .appearance_invalidation_batch()
        .unwrap();
    assert_eq!(still_pending.basis(), pending.basis());
    assert_eq!(still_pending.revision(), pending.revision());
    assert_eq!(still_pending.selected_count(), pending.selected_count());
    session.appearance_owner_snapshot = Some(snapshot);
    publish(&mut session, &host, 1, 10, None);
    let unchanged = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("validated dependency reuse prepares"));
    assert_eq!(
        unchanged
            .appearance_selection_cost_report()
            .selected_instance_count(),
        0
    );
    unchanged.assert_no_unpublished_appearance_for_test();
    let _ = session.shutdown();
}
