//! Scripted effect boundary: no-paint completion may accept raw Motion only at zero physical cost.
use super::*;

#[test]
fn pending_no_paint_motion_accepts_only_after_zero_cost_completion() {
    let (mut session, host, surface, command) = mounted();
    let original = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let target =
        UiMotionTargetIdentity::from_family_owner(surface, command.mounted_instance(), 819);
    install(&mut session, target, original, 819, false, None);
    present(&mut session, &host, surface, 1, 2);
    let original = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let previous = session
        .mounted
        .accepted_motion_for_command(original, command)
        .unwrap()
        .unwrap();
    let prepared = session.mounted.prepare_motion_tick(56, original).unwrap();
    assert_eq!(prepared.receipt().samples()[0].opacity_units(), 8_192);
    host.push_in_flight(
        vec![
            ScriptedSurfaceCompletion::Pending,
            ScriptedSurfaceCompletion::Presented(completion(3, false)),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    assert!(matches!(
        session
            .mounted
            .present_prepared_motion_tick(&session.host_session, prepared, original),
        UiMountedMotionSampleSettlement::Deferred
    ));
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(original, command)
            .unwrap(),
        Some(previous)
    );
    assert!(matches!(
        session
            .mounted
            .complete_motion_sample_presentation(&session.host_session),
        Some(UiMountedMotionSampleSettlement::Deferred)
    ));
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(original, command)
            .unwrap(),
        Some(previous)
    );
    assert!(matches!(
        session
            .mounted
            .complete_motion_sample_presentation(&session.host_session),
        Some(UiMountedMotionSampleSettlement::Committed(_))
    ));
    let current = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let accepted = session
        .mounted
        .accepted_motion_for_command(current, command)
        .unwrap()
        .unwrap();
    assert_eq!(accepted.opacity_units(), 8_192);
    assert_eq!(accepted.presentation_basis(), current);
    assert_ne!(current, original);
    assert!(session
        .mounted
        .accepted_motion_for_command(original, command)
        .is_err());

    // A port cannot omit NativePaint while reporting physical submission.
    host.push_presentation(UiHostSurfacePresentationOutcome::Presented(
        UiMountedSurfacePresentationCompletion::new(
            UiHostSurfacePresentationMode::NativeDisplay,
            UiHostPresentationEpoch::issued_by_host(4),
            UiMountedCompletedEffects::new(vec![]),
            UiHostPresentationCostReport::from_adapter(UiHostPresentationCostInput {
                queue_submissions: 1,
                ..Default::default()
            }),
        ),
    ));
    let prepared = session.mounted.prepare_motion_tick(70, current).unwrap();
    assert!(matches!(
        session
            .mounted
            .present_prepared_motion_tick(&session.host_session, prepared, current),
        UiMountedMotionSampleSettlement::PresentationIndeterminate
    ));
    assert!(
        session
            .mounted
            .accepted_motion_for_command(current, command)
            .is_err(),
        "inconsistent physical truth cannot authorize composition"
    );
    let _ = session.shutdown();
}
