//! An admitted click can wait for Motion without becoming invalid raw input.
use super::*;

#[test]
fn admitted_activation_survives_motion_epoch_but_raw_old_epoch_stays_invalid() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role, 1);
    let (surface, _) = super::super::mounting_fixture::mount(&mut session, 1_000);
    close(&mut session, &role, 1, "admitted-motion-input");
    session.advance_mounted_identity_frame().unwrap();
    let frame = prepare(&mut session);
    publish(&mut session, &host, frame, 1);
    // Operability is owner-issued, not inferred from mounting. Evaluate it
    // through an initial real activation so the button has painted appearance
    // mechanics for the subsequent Motion sample.
    activate(&mut session, surface, 1);
    close(&mut session, &role, 2, "admitted-motion-ready");
    let frame = prepare(&mut session);
    publish(&mut session, &host, frame, 2);
    let observed = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let hit = session
        .mounted
        .interaction_hit_test_basis(observed)
        .unwrap();
    assert_eq!(hit.rows().len(), 1);
    let row = hit.rows()[0];
    let instance = row.mounted_instance();
    let bounds = row.bounds();
    let position = UiHostSurfacePosition::viewport_logical(
        ((bounds.x() + bounds.width() / 2.0) * 1_000.0) as i64,
        ((bounds.y() + bounds.height() / 2.0) * 1_000.0) as i64,
    );
    let mut activation = None;
    for (sequence, transition, held) in [
        (3, UiHostPointerButtonTransition::Pressed, true),
        (4, UiHostPointerButtonTransition::Released, false),
    ] {
        let batch = super::super::pointer_tests::pointer_batch(
            session.host_session.identity().as_u64(),
            observed,
            sequence,
            UiHostPointerIdentity::new(1),
            position,
            Some(transition),
            held,
        );
        let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) =
            session.admit_host_interaction_batch(batch)
        else {
            panic!("pointer input must be admitted before the animation completes")
        };
        for transition in receipt.into_transitions().into_vec() {
            if let crate::facade::interaction::UiInteractionTransition::Semantic(value) = transition
            {
                activation = Some(value);
            }
        }
    }
    let activation = activation.expect("real press/release emits an activation");
    assert_eq!(activation.target().mounted_instance(), instance);
    use crate::runtime::motion::*;
    // Only semantic track issuance is scripted; the accepted physical sample,
    // mounted epoch change, and sealed input routing use production owners.
    let target = UiMotionTargetIdentity::from_mounted_owner(surface, instance, 821);
    let geometry = Some([bounds.x(), bounds.y(), bounds.width(), bounds.height()]);
    let committed = UiMotionCommitReceipt::for_sampling_test_transition(
        821,
        target,
        observed,
        geometry,
        true,
        geometry,
        true,
        UiMotionDeclaration::portal_entrance(),
        None,
    );
    session.mounted.install_motion_commit(committed).unwrap();
    let prepared = session.mounted.prepare_motion_tick(1, observed).unwrap();
    host.push_presentation(UiHostSurfacePresentationOutcome::Presented(
        UiMountedSurfacePresentationCompletion::new(
            UiHostSurfacePresentationMode::NativeDisplay,
            UiHostPresentationEpoch::issued_by_host(2),
            UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
            UiHostPresentationCostReport::default(),
        ),
    ));
    match session
        .mounted
        .present_prepared_motion_tick(&session.host_session, prepared, observed)
    {
        crate::mounting::UiMountedMotionSampleSettlement::Committed(_) => {}
        crate::mounting::UiMountedMotionSampleSettlement::Deferred => {
            panic!("scripted sample unexpectedly pending")
        }
        crate::mounting::UiMountedMotionSampleSettlement::Discarded => {
            panic!("scripted sample rejected before effect")
        }
        crate::mounting::UiMountedMotionSampleSettlement::PresentationIndeterminate => {
            panic!("scripted sample completion did not match its physical effect contract")
        }
    }
    let current = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    assert_eq!(current.frame(), observed.frame());
    assert!(current.epoch() > observed.epoch());
    assert_eq!(
        session.mounted.classify_interaction_presentation(observed),
        Err(crate::mounting::UiPresentedFrameBasisDenial::PresentationEpochMismatch),
        "raw observations must still be checked against exact presented pixels",
    );
    let UiIntentRouteResolution::Product(route) = session
        .resolve_intent_route(
            crate::facade::interaction::UiIntentRouteSource::mounted_interaction(activation),
        )
        .expect("already-admitted activation survives a presentation-only epoch advance")
    else {
        panic!("authored product route")
    };
    assert_eq!(
        route.target().mounted_instance(),
        instance,
        "routing cannot select a new control"
    );
    let _ = session.shutdown();
}
