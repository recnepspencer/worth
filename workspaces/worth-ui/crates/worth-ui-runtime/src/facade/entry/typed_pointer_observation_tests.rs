use crate::certification_support::ScriptedPresentationHost;
use crate::mounting::{UiMountedFrameOutcome, UiMountedFramePublicationReceipt};
use crate::runtime::tests::native_pointer_observation_test_support::source_backed_hover_consumer_app_with_host;
use worth_ui_host_contract::{
    UiHostObservationBatch, UiHostObservationBatchInput, UiHostObservationLoss,
    UiHostObservationPayload, UiHostObservationPresentationBasis, UiHostObservationReport,
    UiHostObservationSequence, UiHostObservationSequenceRange, UiHostObservationTimeBasis,
    UiHostPointerButton, UiHostPointerButtonTransition, UiHostPointerCaptureEpoch,
    UiHostPointerDeviceKind, UiHostPointerIdentity, UiHostPresentationEpoch,
    UiHostPressedPointerButtons, UiHostProtocolContract, UiHostProtocolNegotiation,
    UiHostSurfacePosition, UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

#[test]
fn host_pointer_kinds_reach_owners_and_touch_cannot_activate() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_hover_consumer_app_with_host(host)
        .launch_native_surface()
        .expect("typed pointer fixture should launch");
    super::native_application_identity_trace_test_support::install_bound_surface_geometry(
        &mut shell,
    );
    let frame = published(shell.present_frame(100, 1));
    let binding = *frame.bindings().first().expect("native binding");
    let host_surface = shell.session.mounted.view().surface_bindings()[0].host_surface_identity();
    let presentation = UiHostObservationPresentationBasis::new(
        host_surface,
        frame.frame(),
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let point = interior_point(&shell, presentation);
    let host_session = shell.session.host_session.identity().as_u64();

    for (sequence, pointer, kind) in [
        (1, 1, UiHostPointerDeviceKind::Mouse),
        (2, 2, UiHostPointerDeviceKind::Stylus),
        (3, 3, UiHostPointerDeviceKind::Touch),
    ] {
        let receipt = admit(
            &mut shell,
            pointer_batch(
                host_session,
                presentation,
                sequence,
                UiHostPointerIdentity::new(pointer),
                kind,
                point,
                None,
            ),
        );
        assert_eq!(receipt.pointer_presence_denials(), &[]);
        assert_eq!(receipt.pointer_presence_transitions().len(), 1);
    }

    let surface = shell.session.mounted.view().surface_bindings()[0].semantic_surface_identity();
    let presence = shell
        .session
        .interaction
        .pointer_presence_appearance_snapshot()
        .expect("hover demand should install pointer presence");
    assert_eq!(
        presence.primary_pointer(surface),
        Some(UiHostPointerIdentity::new(2))
    );
    assert_eq!(presence.postures().len(), 3);
    assert!(presence.postures().iter().any(|posture| {
        posture.pointer() == UiHostPointerIdentity::new(1)
            && posture.kind() == crate::runtime::interaction::UiPrimaryPointerKind::Mouse
    }));
    assert!(presence.postures().iter().any(|posture| {
        posture.pointer() == UiHostPointerIdentity::new(2)
            && posture.kind() == crate::runtime::interaction::UiPrimaryPointerKind::Stylus
    }));
    assert!(presence.postures().iter().any(|posture| {
        posture.pointer() == UiHostPointerIdentity::new(3)
            && posture.kind() == crate::runtime::interaction::UiPrimaryPointerKind::Touch
    }));

    let mouse_press = admit(
        &mut shell,
        pointer_batch(
            host_session,
            presentation,
            4,
            UiHostPointerIdentity::new(1),
            UiHostPointerDeviceKind::Mouse,
            point,
            Some(UiHostPointerButtonTransition::Pressed),
        ),
    );
    assert_pressed_kind(&mouse_press, UiHostPointerDeviceKind::Mouse);
    let mouse_release = admit(
        &mut shell,
        pointer_batch(
            host_session,
            presentation,
            5,
            UiHostPointerIdentity::new(1),
            UiHostPointerDeviceKind::Mouse,
            point,
            Some(UiHostPointerButtonTransition::Released),
        ),
    );
    assert_activation_kind(&mouse_release, UiHostPointerDeviceKind::Mouse);

    let stylus_press = admit(
        &mut shell,
        pointer_batch(
            host_session,
            presentation,
            6,
            UiHostPointerIdentity::new(2),
            UiHostPointerDeviceKind::Stylus,
            point,
            Some(UiHostPointerButtonTransition::Pressed),
        ),
    );
    assert_pressed_kind(&stylus_press, UiHostPointerDeviceKind::Stylus);
    let stylus_release = admit(
        &mut shell,
        pointer_batch(
            host_session,
            presentation,
            7,
            UiHostPointerIdentity::new(2),
            UiHostPointerDeviceKind::Stylus,
            point,
            Some(UiHostPointerButtonTransition::Released),
        ),
    );
    assert_activation_kind(&stylus_release, UiHostPointerDeviceKind::Stylus);

    let touch_press = admit(
        &mut shell,
        pointer_batch(
            host_session,
            presentation,
            8,
            UiHostPointerIdentity::new(3),
            UiHostPointerDeviceKind::Touch,
            point,
            Some(UiHostPointerButtonTransition::Pressed),
        ),
    );
    assert_pressed_kind(&touch_press, UiHostPointerDeviceKind::Touch);
    let touch_release = admit(
        &mut shell,
        pointer_batch(
            host_session,
            presentation,
            9,
            UiHostPointerIdentity::new(3),
            UiHostPointerDeviceKind::Touch,
            point,
            Some(UiHostPointerButtonTransition::Released),
        ),
    );
    assert!(!touch_release.transitions().iter().any(|transition| {
        matches!(
            transition,
            crate::facade::interaction::UiInteractionTransition::Semantic(
                crate::facade::interaction::UiSemanticInteraction::Activate(_)
            )
        )
    }));
    let shutdown = shell.shutdown();
    assert!(shutdown.host_session_released());
}

fn admit(
    shell: &mut crate::facade::entry::WorthUiNativeApplicationShell,
    batch: UiHostObservationBatch,
) -> crate::facade::interaction::UiInteractionBatchReceipt {
    match shell.session.admit_host_interaction_batch(batch) {
        crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) => receipt,
        _ => panic!("typed pointer report should be admitted"),
    }
}

fn assert_pressed_kind(
    receipt: &crate::facade::interaction::UiInteractionBatchReceipt,
    kind: UiHostPointerDeviceKind,
) {
    let press = receipt
        .transitions()
        .iter()
        .find_map(|transition| match transition {
            crate::facade::interaction::UiInteractionTransition::PointerPressed(press) => {
                Some(press)
            }
            _ => None,
        });
    assert_eq!(press.map(|press| press.pointer_device_kind()), Some(kind));
}

fn assert_activation_kind(
    receipt: &crate::facade::interaction::UiInteractionBatchReceipt,
    kind: UiHostPointerDeviceKind,
) {
    let activation = receipt
        .transitions()
        .iter()
        .find_map(|transition| match transition {
            crate::facade::interaction::UiInteractionTransition::Semantic(
                crate::facade::interaction::UiSemanticInteraction::Activate(activation),
            ) => Some(activation),
            _ => None,
        });
    let Some(activation) = activation else {
        panic!("mouse and stylus release should activate");
    };
    let crate::facade::interaction::UiActivateInteractionSource::Pointer(gesture) =
        activation.source()
    else {
        panic!("pointer release should retain a pointer gesture source");
    };
    assert_eq!(gesture.pointer_device_kind(), kind);
}

fn interior_point(
    shell: &crate::facade::entry::WorthUiNativeApplicationShell,
    presentation: UiHostObservationPresentationBasis,
) -> UiHostSurfacePosition {
    let hit_test = shell
        .session
        .mounted
        .interaction_hit_test_basis(presentation)
        .expect("the published frame should expose hit-test geometry");
    let row = hit_test
        .rows()
        .first()
        .expect("the fixture has one hit-test row");
    let bounds = row.bounds();
    let clip = row.clip_bounds();
    let x = (bounds.x().max(clip.x()) + (bounds.x() + bounds.width()).min(clip.x() + clip.width()))
        / 2.0;
    let y = (bounds.y().max(clip.y())
        + (bounds.y() + bounds.height()).min(clip.y() + clip.height()))
        / 2.0;
    UiHostSurfacePosition::viewport_logical(
        (x * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32) as i64,
        (y * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f32) as i64,
    )
}

fn pointer_batch(
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
    pointer: UiHostPointerIdentity,
    kind: UiHostPointerDeviceKind,
    position: UiHostSurfacePosition,
    transition: Option<UiHostPointerButtonTransition>,
) -> UiHostObservationBatch {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    };
    let sequence = UiHostObservationSequence::new(sequence);
    let payload = match transition {
        Some(transition) => UiHostObservationPayload::PointerButton {
            pointer,
            capture_epoch: UiHostPointerCaptureEpoch::new(1),
            button: UiHostPointerButton::Primary,
            transition,
            position,
        },
        None => UiHostObservationPayload::PointerMotion {
            pointer,
            capture_epoch: UiHostPointerCaptureEpoch::new(1),
            pressed_buttons: UiHostPressedPointerButtons::NONE,
            position,
        },
    };
    let report = UiHostObservationReport::new(
        sequence,
        UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
        payload,
    )
    .with_pointer_device_kind(kind)
    .expect("pointer payload should carry its host-originated device kind");
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session,
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![report],
    })
    .expect("typed pointer report should satisfy the host contract")
}

fn published(
    outcome: Result<
        UiMountedFrameOutcome,
        crate::facade::entry::WorthUiMountedFrameExecutionStop<'_>,
    >,
) -> UiMountedFramePublicationReceipt {
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(_) => panic!("typed pointer frame should execute"),
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
        | UiMountedFrameOutcome::CompletionDenied(_) => {
            panic!("typed pointer frame should publish")
        }
    }
}
