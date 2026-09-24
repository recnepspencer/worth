//! Vary only host completion timing; input goes through the real host drain.
use super::{authored, session::World, stationary_motion::inputs::pointer_batch};
use crate::certification_support::ScriptedPresentationAcknowledgement;
use crate::certification_support::ScriptedSurfaceCompletion;
use crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome as Outcome;
use crate::facade::entry::WorthUiAdmittedPortalDismissal;
use worth_ui_host_contract::*;

#[test]
fn dismissal_after_completed_motion_accepts_fresh_input() {
    exercise_completion_order(false);
}

#[test]
fn dismissal_admitted_before_motion_completion_survives_dispatch_after_completion() {
    exercise_completion_order(true);
}

#[test]
fn delayed_dismissal_completion_reopens_input_and_frame_gates() {
    let source = authored::source().replace(
        "dismiss escape anchor_gone",
        "dismiss escape outside_press anchor_gone",
    );
    let mut world = World::launch_with_source(false, false, source);
    // Pulse has one native surface. The shared world normally has a second
    // surface; remove its binding so it cannot supersede this pending request
    // through the shared asynchronous presentation owner.
    let secondary_binding = world
        .session
        .inspect_mounted_identity()
        .surface_bindings()
        .iter()
        .find(|binding| binding.semantic_surface_identity() == world.surfaces[1])
        .unwrap()
        .binding_generation();
    world
        .session
        .deregister_host_surface(secondary_binding)
        .unwrap();
    let initial = world.prepare();
    world.publish(initial, 1, true);
    world.open(0, "overlay.menu", None, 10);
    let surface = world.surfaces[0];
    let basis = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let outside = UiHostSurfacePosition::viewport_logical(790_000, 590_000);
    let dismissal = drain_outside_press(&mut world, basis.basis(), outside, 1);
    world.host.push_in_flight(
        vec![
            ScriptedSurfaceCompletion::Pending,
            ScriptedSurfaceCompletion::Presented(ScriptedPresentationAcknowledgement::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                UiHostPresentationEpoch::issued_by_host(32),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                Default::default(),
            )),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let pending = match world
        .session
        .publish_admitted_portal_dismissal(dismissal, 11)
    {
        Outcome::InFlight(pending) => pending.detach_for_native(),
        Outcome::Stopped(stop) => panic!("dismissal setup stopped: {stop:?}"),
        _ => panic!("dismissal must own a real pending presentation"),
    };
    assert!(world.session.mounted.has_active_presentation_attempt());
    assert!(!world.session.mounted.observation_basis_admission_ready());
    world.host.enqueue_observation_for_next_drain(pointer_batch(
        world.session.host_session.identity().as_u64(),
        basis.basis(),
        2,
        UiHostPointerIdentity::new(1),
        outside,
        Some(UiHostPointerButtonTransition::Released),
        false,
    ));
    let pending = match pending.complete(&mut world.session, 12) {
        Outcome::InFlight(pending) => pending.detach_for_native(),
        _ => panic!("the first host completion poll must stay pending"),
    };
    assert!(world.session.mounted.has_active_presentation_attempt());
    assert!(!world.session.mounted.observation_basis_admission_ready());
    assert_eq!(world.host.pending_observation_batch_count(), 1);
    match pending.complete(&mut world.session, 13) {
        Outcome::Published(_) => {}
        Outcome::Stopped(stop) => panic!("released dismissal stopped: {stop:?}"),
        Outcome::InFlight(_) => panic!("released dismissal still in flight"),
        Outcome::Indeterminate(_) => panic!("released dismissal became indeterminate"),
        _ => panic!("released dismissal was ignored"),
    }
    assert!(!world.session.mounted.has_active_presentation_attempt());
    assert!(world.session.mounted.observation_basis_admission_ready());
    let settlement = world
        .session
        .drain_and_admit_host_observation_batches(Default::default());
    assert_eq!(settlement.drain_denial(), None);
    let counts = settlement.counts();
    assert_eq!(counts, (1, 0, 0, 0), "{:?}", settlement.into_outcomes());
    assert_eq!(world.host.pending_observation_batch_count(), 0);
    let shutdown = world.session.shutdown();
    assert!(shutdown.runtime_service_resource_census().is_empty());
}

fn exercise_completion_order(input_before_completion: bool) {
    let source = authored::source().replace(
        "dismiss escape anchor_gone",
        "dismiss escape outside_press anchor_gone",
    );
    let mut world = World::launch_with_source(false, false, source);
    let initial = world.prepare();
    world.publish(initial, 1, true);
    let portal = world.open(0, "overlay.menu", None, 10);
    let surface = world.surfaces[0];
    let observed = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let body = world
        .session
        .portal
        .as_ref()
        .unwrap()
        .placement(portal)
        .unwrap()
        .prepared()
        .bounds()
        .mounted_box();
    let outside = UiHostSurfacePosition::viewport_logical(
        ((body.x() + body.width() + 30.0) * 1_000.0) as i64,
        ((body.y() + body.height() + 30.0) * 1_000.0) as i64,
    );
    let sample = world.session.prepare_motion_tick(1, observed).unwrap();
    world.host.push_in_flight(
        vec![
            ScriptedSurfaceCompletion::Pending,
            ScriptedSurfaceCompletion::Presented(ScriptedPresentationAcknowledgement::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                UiHostPresentationEpoch::issued_by_host(31),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                UiHostPresentationCostReport::default(),
            )),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    world.session.present_prepared_motion_tick(sample, observed);
    assert!(world.session.mounted.motion_sample_presentation_pending());
    world.session.complete_motion_sample_presentation();
    assert!(world.session.mounted.motion_sample_presentation_pending());
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(surface),
        Some(observed)
    );

    if !input_before_completion {
        world.session.complete_motion_sample_presentation();
    }
    let input_basis = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    assert!(
        world.session.mounted.observation_basis_admission_ready(),
        "the native input gate must permit this drain while a motion sample is pending"
    );
    let dismissal = drain_outside_press(&mut world, input_basis.basis(), outside, 1);
    // The production drain admits input first, then consumes the completion.
    assert!(!world.session.mounted.motion_sample_presentation_pending());
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    assert_eq!(current.frame(), observed.frame());
    assert_ne!(current.basis().epoch(), observed.basis().epoch());
    assert_eq!(world.host.pending_observation_batch_count(), 0);

    for _ in world.surfaces {
        world.host.push_native_display_presented();
    }
    let (published, result) = {
        let outcome = world
            .session
            .publish_admitted_portal_dismissal(dismissal, 101);
        let published = matches!(outcome, Outcome::Published(_));
        let result = match outcome {
            Outcome::Stopped(stop) => format!("stopped: {stop:?}"),
            Outcome::Published(_) => "published".to_owned(),
            Outcome::IgnoredInsideTopmostPortal => "ignored inside".to_owned(),
            Outcome::IgnoredNoMatchingPortal => "ignored no portal".to_owned(),
            Outcome::InFlight(_) => "in flight".to_owned(),
            Outcome::Indeterminate(_) => "indeterminate".to_owned(),
        };
        (published, result)
    };
    // Probe liveness separately from the admitted dismissal. A lost click is
    // not evidence that the whole runtime is permanently frozen.
    if !published {
        world.host.enqueue_observation_for_next_drain(pointer_batch(
            world.session.host_session.identity().as_u64(),
            current.basis(),
            2,
            UiHostPointerIdentity::new(1),
            outside,
            Some(UiHostPointerButtonTransition::Released),
            false,
        ));
        let released = world
            .session
            .drain_and_admit_host_observation_batches(Default::default());
        assert_eq!(released.drain_denial(), None);
        assert_eq!(released.counts(), (1, 0, 0, 0));
        let fresh = drain_outside_press(&mut world, current.basis(), outside, 3);
        assert!(
            matches!(
                world.session.publish_admitted_portal_dismissal(fresh, 103),
                Outcome::Published(_)
            ),
            "fresh input must still dismiss after the earlier click fails"
        );
    }
    let _ = world.session.shutdown();
    assert!(published, "admitted dismissal must survive completion ordering; input_before_completion={input_before_completion}, {result}; fresh input recovered");
}

pub(super) fn drain_outside_press(
    world: &mut World,
    basis: UiHostObservationPresentationBasis,
    position: UiHostSurfacePosition,
    sequence: u64,
) -> WorthUiAdmittedPortalDismissal {
    world.host.enqueue_observation_for_next_drain(pointer_batch(
        world.session.host_session.identity().as_u64(),
        basis,
        sequence,
        UiHostPointerIdentity::new(1),
        position,
        Some(UiHostPointerButtonTransition::Pressed),
        true,
    ));
    let settlement = world
        .session
        .drain_and_admit_host_observation_batches(Default::default());
    assert_eq!(settlement.drain_denial(), None);
    let counts = settlement.counts();
    let (outcomes, dismissals) = settlement.into_routing_parts();
    assert_eq!(counts, (1, 0, 0, 0), "{outcomes:?}");
    assert_eq!(
        dismissals.len(),
        1,
        "real input must seal exactly one dismissal"
    );
    dismissals.into_vec().into_iter().next().unwrap()
}
