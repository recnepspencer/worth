#[path = "appearance_motion_acceptance_tests/no_paint.rs"]
mod no_paint;
#[path = "appearance_motion_acceptance_tests/pending_mount.rs"]
mod pending_mount;

#[path = "appearance_motion_acceptance_fixture.rs"]
mod fixture;
use crate::certification_support::{ScriptedPresentationHost, ScriptedSurfaceCompletion};
use crate::facade::WorthUiActiveApplicationSession;
use crate::mounting::{UiMountedFrameOutcome, UiMountedMotionSampleSettlement};
use crate::runtime::motion::UiMotionTargetIdentity;
use fixture::*;
use worth_ui_host_contract::*;

#[test]
fn appearance_refresh_composes_the_exact_physically_accepted_command_sample() {
    let (mut session, host, surface, command) = mounted();
    let original = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let target =
        UiMotionTargetIdentity::from_family_owner(surface, command.mounted_instance(), 801);
    install(&mut session, target, original, 801, false, None);
    present(&mut session, &host, surface, 1, 2);
    present(&mut session, &host, surface, 56, 3);
    let current = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(current, command)
            .unwrap()
            .unwrap()
            .opacity_units(),
        8_192
    );

    refresh_appearance(&mut session, &host, 2);
    let projection = session
        .mounted
        .current_unpublished_appearance()
        .expect("appearance transport remains current")
        .expect("accepted Motion changes the retained appearance mechanic");
    let mechanic = projection
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().successor().mechanics())
        .find_map(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::Surface(surface) => Some(surface),
            _ => None,
        })
        .expect("the exact mounted surface remains present");
    assert_eq!(
        mechanic.node_receipt().mounted_instance(),
        command.mounted_instance()
    );
    assert_eq!(mechanic.opacity().units(), 8_192);
    let _ = session.shutdown();
}

// Only semantic track issuance is a fixture. Command identities, preparation,
// host acceptance, physical epoch admission, and retained ownership are real.
#[test]
fn geometry_free_motion_evidence_requires_acceptance_and_survives_retarget_and_retirement() {
    let (mut session, host, surface, command) = mounted();
    let reserved = session
        .mounted
        .retention_snapshot()
        .current
        .retained_structural_bytes;
    let original = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let target =
        UiMotionTargetIdentity::from_family_owner(surface, command.mounted_instance(), 811);
    install(&mut session, target, original, 811, false, None);
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(original, command)
            .unwrap(),
        None
    );
    let prepared = session.mounted.prepare_motion_tick(1, original).unwrap();
    assert!(prepared.receipt().samples()[0].geometry().is_none());
    host.push_rejected();
    assert!(matches!(
        session
            .mounted
            .present_prepared_motion_tick(&session.host_session, prepared, original),
        UiMountedMotionSampleSettlement::Discarded
    ));
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(original, command)
            .unwrap(),
        None
    );
    present(&mut session, &host, surface, 1, 2);
    let accepted = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let first = session
        .mounted
        .accepted_motion_for_command(accepted, command)
        .unwrap()
        .unwrap();
    assert_eq!(first.opacity_units(), 65_535);
    assert_eq!(first.presentation_basis(), accepted);
    assert!(session
        .mounted
        .accepted_motion_for_command(original, command)
        .is_err());

    host.push_in_flight(
        vec![
            ScriptedSurfaceCompletion::Pending,
            ScriptedSurfaceCompletion::Presented(completion(3, true)),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let prepared = session.mounted.prepare_motion_tick(56, accepted).unwrap();
    assert_eq!(prepared.receipt().samples()[0].opacity_units(), 8_192);
    assert!(matches!(
        session
            .mounted
            .present_prepared_motion_tick(&session.host_session, prepared, accepted),
        UiMountedMotionSampleSettlement::Deferred
    ));
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(accepted, command)
            .unwrap(),
        Some(first)
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
            .accepted_motion_for_command(accepted, command)
            .unwrap(),
        Some(first)
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
    let middle = session
        .mounted
        .accepted_motion_for_command(current, command)
        .unwrap()
        .unwrap();
    assert_eq!(middle.opacity_units(), 8_192);
    assert_eq!(middle.tick(), 56);
    assert_eq!(middle.presentation_basis(), current);
    install(
        &mut session,
        target,
        current,
        812,
        true,
        Some(
            crate::runtime::motion::UiMotionRetargetDisposition::Install {
                predecessor:
                    crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
            },
        ),
    );
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(current, command)
            .unwrap(),
        Some(middle)
    );
    present(&mut session, &host, surface, 57, 4);
    present(&mut session, &host, surface, 127, 5);
    let current = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(current, command)
            .unwrap()
            .unwrap()
            .opacity_units(),
        58_367
    );
    present(&mut session, &host, surface, 197, 6);
    let current = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let terminal = session
        .mounted
        .accepted_motion_for_command(current, command)
        .unwrap()
        .unwrap();
    assert_eq!(terminal.opacity_units(), 65_535);
    assert!(session
        .mounted
        .retire_terminal_motion_sample(terminal.track()));
    assert!(!session.mounted.contains_motion_track(terminal.track()));
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(current, command)
            .unwrap(),
        Some(terminal)
    );
    assert_eq!(
        session
            .mounted
            .retention_snapshot()
            .current
            .retained_structural_bytes,
        reserved,
        "accepted samples consume command slots reserved at mounted admission"
    );
    let _ = session.shutdown();
}

#[test]
fn semantic_candidate_keeps_later_accepted_motion_on_unchanged_commands() {
    let (mut session, host, surface, command) = mounted();
    let original = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let target =
        UiMotionTargetIdentity::from_family_owner(surface, command.mounted_instance(), 821);
    install(&mut session, target, original, 821, false, None);
    let graph = session
        .mounted
        .current_mounted_identity_basis(command.mounted_instance())
        .unwrap()
        .graph_node_identity();
    let node = session.mounted_graph_node(graph).unwrap();
    session.mount_instance(node, surface).unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut session,
        surface,
    );
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("semantic successor prepares"));
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Presented(completion(3, true))],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(100),
        1,
    );
    if let UiMountedFrameOutcome::RejectedBeforeEffects(ref rejection) = outcome {
        panic!("semantic rejection: {:?}", rejection.rejections());
    }
    let UiMountedFrameOutcome::InFlight(pending) = outcome else {
        panic!(
            "semantic candidate stays pending: {:?}",
            std::mem::discriminant(&outcome)
        )
    };
    present(&mut session, &host, surface, 1, 2);
    let accepted = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let sample = session
        .mounted
        .accepted_motion_for_command(accepted, command)
        .unwrap()
        .unwrap();
    assert!(matches!(
        session.complete_mounted_presentation(pending, 2),
        UiMountedFrameOutcome::Published(_)
    ));
    let current = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    assert_ne!(current.frame(), accepted.frame());
    assert_eq!(
        session
            .mounted
            .accepted_motion_for_command(current, command)
            .unwrap(),
        Some(sample)
    );
    assert_eq!(
        sample.presentation_basis(),
        accepted,
        "receipt preserves its actual acceptance epoch"
    );
    assert!(session
        .mounted
        .accepted_motion_for_command(accepted, command)
        .is_err());
    let _ = session.shutdown();
}
