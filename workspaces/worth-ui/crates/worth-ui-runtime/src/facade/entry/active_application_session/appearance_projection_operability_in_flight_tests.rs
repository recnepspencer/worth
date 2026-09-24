use super::*;
use crate::certification_support::ScriptedPresentationAcknowledgement;

#[test]
fn changed_operability_supersedes_in_flight_pixels_before_mounted_commit() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role, 1);
    let (surface, _) = super::super::mounting_fixture::mount(&mut session, 1_000);
    close(&mut session, &role, 1, "operability-in-flight-initial");
    session.advance_mounted_identity_frame().unwrap();
    let initial_frame = prepare(&mut session);
    publish(&mut session, &host, initial_frame, 1);

    let (target, initial) = activate(&mut session, surface, 1);
    let pending_route = activation_route(&mut session, surface, 3);
    let pending_candidate = session.prepare_intent_payload(pending_route).unwrap();
    close(&mut session, &role, 1, "operability-in-flight-ready");
    let frame = project(
        &mut session,
        &[(target, 10)],
        &[(
            surface,
            Some((target, UiPointerAffordanceFamily::Activation)),
        )],
    );
    host.push_in_flight(
        vec![
            crate::certification_support::ScriptedSurfaceCompletion::Presented(
                ScriptedPresentationAcknowledgement::new(
                    crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
                    crate::certification_support::scripted_presentation_epoch(),
                    UiMountedCompletedEffects::new(Vec::new()),
                    UiHostPresentationCostReport::default(),
                ),
            ),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let predecessor = session.current_mounted_publication().unwrap().frame();
    let in_flight = match session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(100),
        2,
    ) {
        crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("operability publication must remain in flight"),
    };

    session
        .update_intent_boolean_fact(&fixture::fact(fixture::MUTABLE), false)
        .unwrap();
    let changed = match session.evaluate_intent_operability(pending_candidate) {
        UiIntentOperabilityOutcome::Operable(proof) => proof.decision().clone(),
        UiIntentOperabilityOutcome::Inoperable(candidate) => candidate.decision().clone(),
    };
    assert_ne!(changed, initial);

    assert!(matches!(
        session.complete_mounted_presentation(in_flight, 4),
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
    ));
    assert_eq!(
        session.current_mounted_publication().unwrap().frame(),
        predecessor,
        "stale pixels must not advance mounted publication"
    );

    close(&mut session, &role, 1, "operability-in-flight-recovery");
    let frame = project(
        &mut session,
        &[(target, 60)],
        &[(surface, Some((target, UiPointerAffordanceFamily::Default)))],
    );
    publish(&mut session, &host, frame, 5);
    assert_ne!(
        session.current_mounted_publication().unwrap().frame(),
        predecessor
    );
    let _ = session.shutdown();
}

#[test]
fn detached_posture_completion_retains_recovery_when_stale_pixels_may_have_effected() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role, 1);
    let (surface, _) = super::super::mounting_fixture::mount(&mut session, 1_000);
    close(&mut session, &role, 1, "detached-posture-initial");
    session.advance_mounted_identity_frame().unwrap();
    let initial_frame = prepare(&mut session);
    publish(&mut session, &host, initial_frame, 1);

    let (target, initial_decision) = activate(&mut session, surface, 1);
    let stale_route = activation_route(&mut session, surface, 3);
    let stale_candidate = session.prepare_intent_payload(stale_route).unwrap();
    close(&mut session, &role, 1, "detached-posture-ready");
    let owner_frame = project(
        &mut session,
        &[(target, 10)],
        &[(
            surface,
            Some((target, UiPointerAffordanceFamily::Activation)),
        )],
    );
    publish(&mut session, &host, owner_frame, 2);

    let route = activation_route(&mut session, surface, 5);
    let posture = session
        .intent_postures
        .prepare(
            route.graph_node(),
            route.target(),
            crate::fact_contract::UiIntentPostureReference::Route(route.definition_id()),
            crate::fact_contract::UiIntentPostureKind::Denied,
        )
        .unwrap();
    host.push_in_flight(
        vec![
            crate::certification_support::ScriptedSurfaceCompletion::Presented(
                ScriptedPresentationAcknowledgement::new(
                    crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
                    crate::certification_support::scripted_presentation_epoch(),
                    UiMountedCompletedEffects::new(Vec::new()),
                    UiHostPresentationCostReport::default(),
                ),
            ),
        ],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    let predecessor = session.current_mounted_publication().unwrap().frame();
    let outcome = session.publish_native_intent_posture(
        crate::facade::entry::WorthUiNativeIntentPosture::new(
            posture.0,
            posture.1,
            crate::fact_contract::UiIntentPostureKind::Denied,
        ),
        crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        crate::runtime::rebind::UiRebindExecutionRequest::new(3),
    );
    let pending = match crate::facade::entry::native_managed_rebind::normalize_managed_intent_posture(outcome) {
        crate::facade::entry::native_managed_rebind::ManagedIntentPostureNormalization::Pending(pending) => pending,
        _ => panic!("posture presentation must detach while host work is in flight"),
    };

    session
        .update_intent_boolean_fact(&fixture::fact(fixture::MUTABLE), false)
        .unwrap();
    let changed = match session.evaluate_intent_operability(stale_candidate) {
        UiIntentOperabilityOutcome::Operable(proof) => proof.decision().clone(),
        UiIntentOperabilityOutcome::Inoperable(candidate) => candidate.decision().clone(),
    };
    assert_ne!(changed, initial_decision);

    use crate::facade::entry::native_intent_posture::WorthUiNativeIntentPosturePublicationOutcome as Outcome;
    let recovery = match pending.complete(&mut session, 4) {
        Outcome::Indeterminate(recovery) => recovery,
        _ => panic!("stale detached pixels with possible effects must retain recovery"),
    };
    assert!(recovery.frame().report().awaits_physical_recovery());
    {
        let returned = recovery.into_session_for_shutdown();
        assert_eq!(
            returned.current_mounted_publication().unwrap().frame(),
            predecessor,
            "stale detached pixels must not advance mounted publication"
        );
    }
    let _ = session.shutdown();
}

/// An observation turn that closes while a publication is in flight cannot
/// decide the hovered target's operability. That is not a decision that the
/// target is inoperable: the pointer keeps the affordance it already shows for
/// the same target, instead of flickering to the default and back.
#[test]
fn withheld_operability_keeps_the_published_affordance_of_the_same_target() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role, 1);
    let (surface, _) = super::super::mounting_fixture::mount(&mut session, 1_000);
    close(&mut session, &role, 1, "withheld-initial");
    session.advance_mounted_identity_frame().unwrap();
    let initial_frame = prepare(&mut session);
    publish(&mut session, &host, initial_frame, 1);

    let (target, _) = activate(&mut session, surface, 1);
    close(&mut session, &role, 1, "withheld-ready");
    let frame = project(
        &mut session,
        &[(target, 10)],
        &[(
            surface,
            Some((target, UiPointerAffordanceFamily::Activation)),
        )],
    );
    host.push_in_flight(
        vec![
            crate::certification_support::ScriptedSurfaceCompletion::Presented(
                ScriptedPresentationAcknowledgement::new(
                    crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
                    crate::certification_support::scripted_presentation_epoch(),
                    UiMountedCompletedEffects::new(Vec::new()),
                    UiHostPresentationCostReport::default(),
                ),
            ),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let in_flight = match session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(100),
        2,
    ) {
        crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("the pointer publication must remain in flight"),
    };

    close(&mut session, &role, 1, "withheld-during-flight");
    let withheld = session
        .pointer_affordance_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.projections().first())
        .and_then(|projection| projection.operability())
        .is_some_and(|operability| {
            operability.is_err_and(|unavailable| unavailable.is_withheld_by_publication())
        });
    assert!(
        withheld,
        "the turn closed while the publication was in flight"
    );
    assert!(matches!(
        session.complete_mounted_presentation(in_flight, 3),
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));

    // The next frame lowers the pointer from the withheld observation: the
    // published Activation stands, so the pointer receives no work at all.
    prepare(&mut session).assert_no_unpublished_appearance_for_test();
    let _ = session.shutdown();
}
