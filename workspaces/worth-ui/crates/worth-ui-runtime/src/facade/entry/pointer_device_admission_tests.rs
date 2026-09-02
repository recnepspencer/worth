use crate::certification_support::ScriptedPresentationHost;
use crate::mounting::{UiMountedFrameOutcome, UiMountedFramePublicationReceipt};
use crate::runtime::tests::native_pointer_observation_test_support::source_backed_hover_consumer_app_with_host;
use worth_ui_host_contract::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationFamily,
    UiHostObservationLoss, UiHostObservationPayload, UiHostObservationPresentationBasis,
    UiHostObservationReport, UiHostObservationSequence, UiHostObservationSequenceRange,
    UiHostObservationTimeBasis, UiHostPointerButton, UiHostPointerButtonTransition,
    UiHostPointerCaptureEpoch, UiHostPointerDeviceKind, UiHostPointerIdentity,
    UiHostPresentationEpoch, UiHostPressedPointerButtons, UiHostProtocolContract,
    UiHostProtocolNegotiation, UiHostSurfacePosition, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

#[test]
fn native_absent_pointer_kind_is_denied_without_pointer_effects() {
    let mut world = pointer_world();
    let motion_batch = pointer_batch(&world, 1, None, None);
    let motion = admit(&mut world, motion_batch);
    assert_missing_denial(&motion, UiHostObservationFamily::PointerMotion);
    assert!(motion.transitions().is_empty());
    assert_eq!(motion.state().active_gestures(), 0);
    assert_eq!(motion.state().pointer_presence_records(), 0);

    let press_batch = pointer_batch(
        &world,
        2,
        None,
        Some(UiHostPointerButtonTransition::Pressed),
    );
    let press = admit(&mut world, press_batch);
    assert_missing_denial(&press, UiHostObservationFamily::PointerButton);
    assert!(press.transitions().is_empty());
    assert_eq!(press.state().counters().button_reports(), 0);

    let release_batch = pointer_batch(
        &world,
        3,
        None,
        Some(UiHostPointerButtonTransition::Released),
    );
    let release = admit(&mut world, release_batch);
    assert_missing_denial(&release, UiHostObservationFamily::PointerButton);
    assert!(release.transitions().is_empty());
    assert_eq!(release.state().active_gestures(), 0);
    assert_eq!(release.state().counters().semantic_interactions(), 0);
    let _ = world.shell.shutdown();
}

#[test]
fn native_explicit_touch_can_settle_posture_but_cannot_activate() {
    let mut world = pointer_world();
    let press_batch = pointer_batch(
        &world,
        1,
        Some(UiHostPointerDeviceKind::Touch),
        Some(UiHostPointerButtonTransition::Pressed),
    );
    let press = admit(&mut world, press_batch);
    assert!(press.pointer_presence_denials().is_empty());
    assert!(press.transitions().iter().any(|transition| {
        matches!(
            transition,
            crate::facade::interaction::UiInteractionTransition::PointerPressed(_)
        )
    }));

    let release_batch = pointer_batch(
        &world,
        2,
        Some(UiHostPointerDeviceKind::Touch),
        Some(UiHostPointerButtonTransition::Released),
    );
    let release = admit(&mut world, release_batch);
    assert!(!has_activation(&release));
    assert_eq!(release.state().counters().semantic_interactions(), 0);
    let presence = world
        .shell
        .session
        .interaction
        .pointer_presence_appearance_snapshot()
        .expect("the hover consumer installs pointer presence");
    assert!(presence.postures().is_empty());
    let _ = world.shell.shutdown();
}

#[test]
fn native_mid_stream_kind_change_denies_and_stops_without_activation() {
    let mut world = pointer_world();
    let motion_batch = pointer_batch(&world, 1, Some(UiHostPointerDeviceKind::Mouse), None);
    let motion = admit(&mut world, motion_batch);
    assert!(motion.pointer_presence_denials().is_empty());

    let press_batch = pointer_batch(
        &world,
        2,
        Some(UiHostPointerDeviceKind::Mouse),
        Some(UiHostPointerButtonTransition::Pressed),
    );
    let press = admit(&mut world, press_batch);
    assert_eq!(press.state().active_gestures(), 1);

    let changed_batch = pointer_batch(&world, 3, Some(UiHostPointerDeviceKind::Stylus), None);
    let changed = admit(&mut world, changed_batch);
    assert!(matches!(
        changed.pointer_presence_denials(),
        [
            crate::runtime::interaction::UiPointerPresenceAdmissionDenial::PointerKindChanged {
                pointer: _,
                prior: UiHostPointerDeviceKind::Mouse,
                observed: UiHostPointerDeviceKind::Stylus,
            }
        ]
    ));
    assert!(changed.transitions().iter().any(|transition| {
        matches!(
            transition,
            crate::facade::interaction::UiInteractionTransition::Stopped(
                crate::facade::interaction::UiInteractionStop::PointerGesture(stop)
            ) if stop.reason()
                == crate::runtime::interaction::UiPointerGestureStopReason::PointerDeviceKindChanged {
                    expected: UiHostPointerDeviceKind::Mouse,
                    observed: UiHostPointerDeviceKind::Stylus,
                }
        )
    }));
    assert!(!has_press_or_dismiss(&changed));
    assert!(!has_activation(&changed));
    assert_eq!(changed.state().active_gestures(), 0);

    let release_batch = pointer_batch(
        &world,
        4,
        Some(UiHostPointerDeviceKind::Stylus),
        Some(UiHostPointerButtonTransition::Released),
    );
    let release = admit(&mut world, release_batch);
    assert!(!has_press_or_dismiss(&release));
    assert!(!has_activation(&release));
    assert_eq!(release.state().active_gestures(), 0);
    let _ = world.shell.shutdown();
}

struct PointerWorld {
    shell: crate::facade::entry::WorthUiNativeApplicationShell,
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
    point: UiHostSurfacePosition,
}

fn pointer_world() -> PointerWorld {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_hover_consumer_app_with_host(host)
        .launch_native_surface()
        .expect("pointer shell should launch");
    let frame = published(&mut shell);
    let binding = *frame.bindings().first().expect("native binding");
    let host_surface = shell.session.mounted.view().surface_bindings()[0].host_surface_identity();
    let presentation = UiHostObservationPresentationBasis::new(
        host_surface,
        frame.frame(),
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let hit_test = shell
        .session
        .mounted
        .interaction_hit_test_basis(presentation)
        .expect("published frame should expose hit-test geometry");
    let row = hit_test
        .rows()
        .first()
        .expect("fixture has one hit-test row");
    let bounds = row.bounds();
    let clip = row.clip_bounds();
    let point = UiHostSurfacePosition::viewport_logical(
        (((bounds.x().max(clip.x()) + (bounds.x() + bounds.width()).min(clip.x() + clip.width()))
            / 2.0)
            * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32) as i64,
        (((bounds.y().max(clip.y())
            + (bounds.y() + bounds.height()).min(clip.y() + clip.height()))
            / 2.0)
            * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32) as i64,
    );
    PointerWorld {
        host_session: shell.session.host_session.identity().as_u64(),
        shell,
        presentation,
        point,
    }
}

fn pointer_batch(
    world: &PointerWorld,
    sequence: u64,
    kind: Option<UiHostPointerDeviceKind>,
    transition: Option<UiHostPointerButtonTransition>,
) -> UiHostObservationBatch {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let sequence = UiHostObservationSequence::new(sequence);
    let pointer = UiHostPointerIdentity::new(1);
    let payload = match transition {
        Some(transition) => UiHostObservationPayload::PointerButton {
            pointer,
            capture_epoch: UiHostPointerCaptureEpoch::new(1),
            button: UiHostPointerButton::Primary,
            transition,
            position: world.point,
        },
        None => UiHostObservationPayload::PointerMotion {
            pointer,
            capture_epoch: UiHostPointerCaptureEpoch::new(1),
            pressed_buttons: UiHostPressedPointerButtons::NONE,
            position: world.point,
        },
    };
    let report = UiHostObservationReport::new(
        sequence,
        UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
        payload,
    );
    let report = match kind {
        Some(kind) => report
            .with_pointer_device_kind(kind)
            .expect("pointer kind must target a pointer payload"),
        None => report,
    };
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: world.host_session,
        presentation: world.presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![report],
    })
    .expect("pointer shell batch should satisfy host shape")
}

fn admit(
    world: &mut PointerWorld,
    batch: UiHostObservationBatch,
) -> crate::facade::interaction::UiInteractionBatchReceipt {
    match world.shell.session.admit_host_interaction_batch(batch) {
        crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) => receipt,
        _ => panic!("pointer batch should reach the runtime interaction owner"),
    }
}

fn assert_missing_denial(
    receipt: &crate::facade::interaction::UiInteractionBatchReceipt,
    family: UiHostObservationFamily,
) {
    assert!(matches!(
        receipt.pointer_presence_denials(),
        [crate::runtime::interaction::UiPointerPresenceAdmissionDenial::MissingDeviceKind {
            pointer,
            family: observed,
        }] if pointer.value() == 1 && *observed == family
    ));
}

fn has_activation(receipt: &crate::facade::interaction::UiInteractionBatchReceipt) -> bool {
    receipt.transitions().iter().any(|transition| {
        matches!(
            transition,
            crate::facade::interaction::UiInteractionTransition::Semantic(
                crate::facade::interaction::UiSemanticInteraction::Activate(_)
            )
        )
    })
}

fn has_press_or_dismiss(receipt: &crate::facade::interaction::UiInteractionBatchReceipt) -> bool {
    receipt.transitions().iter().any(|transition| {
        matches!(
            transition,
            crate::facade::interaction::UiInteractionTransition::PointerPressed(_)
                | crate::facade::interaction::UiInteractionTransition::DismissRequested(_)
        )
    })
}

fn published(
    shell: &mut crate::facade::entry::WorthUiNativeApplicationShell,
) -> UiMountedFramePublicationReceipt {
    let outcome = match shell.present_frame(100, 1) {
        Ok(outcome) => outcome,
        Err(_) => panic!("frame should execute"),
    };
    match outcome {
        UiMountedFrameOutcome::Published(receipt)
        | UiMountedFrameOutcome::Unchanged(receipt)
        | UiMountedFrameOutcome::Reconciled(receipt) => receipt,
        UiMountedFrameOutcome::RejectedBeforeEffects(_)
        | UiMountedFrameOutcome::InFlight(_)
        | UiMountedFrameOutcome::Superseded(_)
        | UiMountedFrameOutcome::PresentationIndeterminate(_)
        | UiMountedFrameOutcome::RetentionDenied(_)
        | UiMountedFrameOutcome::AdmissionDenied(_)
        | UiMountedFrameOutcome::CompletionDenied(_) => panic!("frame should publish"),
    }
}
