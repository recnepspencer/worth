use worth_ui_host_contract::*;

#[test]
fn evidence_only_source_successor_retains_live_activation_and_next_frame() {
    retained_pointer_journey(false);
}

#[test]
fn evidence_source_with_changed_operability_publishes_pointer_before_committing() {
    retained_pointer_journey(true);
}

fn retained_pointer_journey(change_operability: bool) {
    let (mut session, host, surfaces) = super::mounted_world();
    super::motion(
        &mut session,
        surfaces[0],
        1,
        1,
        UiHostPointerDeviceKind::Mouse,
        false,
    );
    let initial = super::super::prepare(&mut session);
    super::super::publish(&mut session, &host, initial, 2);
    assert_eq!(
        session
            .mounted
            .current_pointer_affordance_for_test(surfaces[0])
            .unwrap()
            .family(),
        UiPointerAffordanceFamily::Activation
    );
    let candidate =
        crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
            crate::runtime::WorthUiSourceProvider::rust_authored("pointer-successor")
                .with_rust_authored_input(super::super::fixture::source_with_role(None, 2)),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(
                "pointer-successor",
            )],
            session.capabilities(),
        );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let set = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) =
        session.classify_observations(set).unwrap()
    else {
        panic!("same authored meaning preserves identity")
    };
    let plan = session
        .compile_preservation_rebind(
            evidence,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let predecessor = session.generation_identity().clone();
    let paint = session.mounted.current_publication().unwrap().clone();
    if change_operability {
        session
            .update_intent_boolean_fact(
                &super::super::fixture::fact(super::super::fixture::MUTABLE),
                false,
            )
            .unwrap();
        host.push_rejected();
        host.push_rejected();
        host.push_rejected();
    }
    let calls = host.presentation_calls();
    let inspection_world = session.appearance_inspection_world(surfaces[0]);
    let target = super::inspection_tests::current_target(&session, surfaces[0]);
    let inspection_before = session.why_pointer_affordance(inspection_world, target);
    assert!(matches!(
        &inspection_before,
        worth_ui_inspection::UiPointerAffordanceInspectionOutcome::Found(_)
    ));
    let prepared = session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(3),
        )
        .unwrap();
    if change_operability {
        let frame = prepared
            .prepared_frame()
            .expect("fresh owner output requires a mounted publication");
        let output = frame.lower_unpublished_appearance_for_test();
        let pointer = output
            .fragments()
            .iter()
            .flat_map(|fragment| fragment.work().successor().mechanics())
            .find_map(|mechanic| match mechanic {
                UiMountedAppearanceMechanic::Pointer(pointer) => Some(pointer),
                _ => None,
            })
            .unwrap();
        assert_eq!(pointer.family(), UiPointerAffordanceFamily::Default);
    } else {
        assert!(
            prepared.prepared_frame().is_none(),
            "validated same mechanics need no host work"
        );
    }
    let retry = match prepared.execute(3) {
        crate::runtime::rebind::UiRebindOutcome::Published(_) if !change_operability => None,
        crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial)
            if change_operability =>
        {
            assert_eq!(
                denial.cause(),
                crate::runtime::rebind::UiRebindDenialCause::HostRejectedBeforeEffects
            );
            Some(
                denial
                    .detach_retry_for_native()
                    .unwrap_or_else(|_| panic!("content retry detaches")),
            )
        }
        _ => panic!("successor must reach the specified publication outcome"),
    };
    if let Some(retry) = retry {
        assert_eq!(session.generation_identity(), &predecessor);
        assert_eq!(
            session.why_pointer_affordance(inspection_world, target),
            inspection_before,
            "rejected successor cannot replace the sealed inspection owner"
        );
        assert_eq!(session.mounted.current_publication(), Some(&paint));
        assert_eq!(
            session
                .mounted
                .current_pointer_affordance_for_test(surfaces[0])
                .unwrap()
                .family(),
            UiPointerAffordanceFamily::Activation
        );
        for _ in &surfaces {
            host.push_native_display_settled_without_effects();
        }
        assert!(matches!(
            retry.rebase_content_and_retry(&mut session, 4).unwrap(),
            crate::runtime::rebind::UiRebindOutcome::Published(_)
        ));
    } else {
        assert_eq!(host.presentation_calls(), calls);
    }
    assert_ne!(session.generation_identity(), &predecessor);
    assert_eq!(
        session.why_pointer_affordance(inspection_world, target),
        worth_ui_inspection::UiPointerAffordanceInspectionOutcome::Expired(
            worth_ui_inspection::UiPointerAffordanceInspectionExpiry::GenerationChanged
        )
    );
    let target = super::inspection_tests::current_target(&session, surfaces[0]);
    let worth_ui_inspection::UiPointerAffordanceInspectionOutcome::Found(explanation) =
        session.why_pointer_affordance(session.appearance_inspection_world(surfaces[0]), target)
    else {
        panic!("accepted successor must immediately explain the committed pointer")
    };
    assert_eq!(
        explanation.presentation,
        worth_ui_inspection::UiPointerAffordanceInspectionPresentation::Current
    );
    assert_eq!(
        explanation.family,
        if change_operability {
            worth_ui_inspection::UiPointerAffordanceInspectionFamily::Default
        } else {
            worth_ui_inspection::UiPointerAffordanceInspectionFamily::Activation
        }
    );
    assert_eq!(
        session
            .pointer_affordance_snapshot
            .as_ref()
            .unwrap()
            .generation(),
        &session.active_generation_identity()
    );
    assert_eq!(
        session
            .mounted
            .current_pointer_affordance_for_test(surfaces[0])
            .unwrap()
            .family(),
        if change_operability {
            UiPointerAffordanceFamily::Default
        } else {
            UiPointerAffordanceFamily::Activation
        }
    );
    let next = super::super::prepare(&mut session);
    next.assert_no_unpublished_appearance_for_test();
    let _ = session.shutdown();
}
