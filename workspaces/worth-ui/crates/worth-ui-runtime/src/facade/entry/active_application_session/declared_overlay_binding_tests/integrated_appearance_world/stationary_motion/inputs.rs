use worth_ui_host_contract::*;

pub(super) fn role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    use worth_ui_dsl::*;
    let base = super::super::authored::role(0);
    let mut foreground = UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
        UiAppearanceStateAxis::Pressed,
    )]);
    for (class, slot) in [
        (
            UiAppearanceAxisClass::PressedIdle,
            "overlay.content.foreground",
        ),
        (
            UiAppearanceAxisClass::PressedArmedInside,
            "overlay.content.foreground",
        ),
        (
            UiAppearanceAxisClass::PressedCapturedOutside,
            "overlay.content.hovered",
        ),
    ] {
        foreground = foreground.with_cell(
            UiAppearanceCell::when([UiAppearanceAxisPredicate::exact(class)]).uses_slot(
                UiThemeSlotIdentity::new(slot).unwrap(),
                UiThemeValueKind::Color,
            ),
        );
    }
    let foreground = foreground.compile(UiAppearanceAspect::Foreground).unwrap();
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new("overlay.content").unwrap(),
        base.revision(),
        base.applicability().clone(),
        base.aspect_contract(),
        base.partitions().iter().map(|(aspect, partition)| {
            (
                *aspect,
                if *aspect == UiAppearanceAspect::Foreground {
                    foreground.clone()
                } else {
                    partition.clone()
                },
            )
        }),
    )
    .unwrap()
}

pub(super) fn source() -> String {
    super::super::authored::source().replacen(
        "foreground use token(overlay.content.foreground)",
        r#"foreground over [pressed] {
            cell idle when pressed = idle use token(overlay.content.foreground)
            cell armed when pressed = armed-inside use token(overlay.content.foreground)
            cell outside when pressed = captured-outside use token(overlay.content.hovered)
        }"#,
        1,
    )
}

pub(super) fn presence(
    session: &crate::facade::WorthUiActiveApplicationSession,
) -> crate::runtime::interaction::UiPointerPresenceAppearanceOwnerSnapshot {
    session
        .interaction
        .pointer_presence_appearance_snapshot()
        .unwrap()
}

pub(super) fn presentation(
    session: &crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
) -> UiHostObservationPresentationBasis {
    session
        .mounted
        .current_publication()
        .unwrap()
        .presentation_for_surface(surface)
        .unwrap()
}

pub(super) fn admit(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
    pointer: UiHostPointerIdentity,
    position: UiHostSurfacePosition,
    press: bool,
) -> crate::runtime::interaction::UiInteractionBatchReceipt {
    let receipt = ingest(session, presentation, sequence, pointer, position, press);
    assert!(receipt.pointer_presence_denials().is_empty());
    assert_eq!(receipt.pointer_presence_transitions().len(), 1);
    receipt
}

pub(super) fn ingest(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
    pointer: UiHostPointerIdentity,
    position: UiHostSurfacePosition,
    press: bool,
) -> crate::runtime::interaction::UiInteractionBatchReceipt {
    let batch = pointer_batch(
        session.host_session.identity().as_u64(),
        presentation,
        sequence,
        pointer,
        position,
        press.then_some(UiHostPointerButtonTransition::Pressed),
        press,
    );
    let ingress = session.admit_host_interaction_batch(batch);
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = ingress
    else {
        panic!("pointer owner proof must reach interaction admission: {ingress:?}");
    };
    receipt
}

pub(super) fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    now: u64,
) {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("pointer proof frame prepares"));
    for _ in frame.surfaces() {
        if now <= 2 {
            host.push_native_display_presented();
        } else {
            host.push_native_display_settled_without_effects();
        }
    }
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        UiPresentationDeadline::at_tick(100),
        now,
    );
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(_)
        | crate::mounting::UiMountedFrameOutcome::Unchanged(_) => {}
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(rejection) => {
            panic!("turn {now} retention: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            panic!("turn {now} admission: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) => {
            panic!("turn {now} completion: {denial:?}")
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejection) => {
            panic!("turn {now} host rejection: {:?}", rejection.rejections())
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(_) => panic!("turn {now} in flight"),
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("turn {now} indeterminate")
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => panic!("turn {now} superseded"),
        crate::mounting::UiMountedFrameOutcome::Reconciled(_) => panic!("turn {now} reconciled"),
    }
}

pub(in super::super) fn pointer_batch(
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
    pointer: UiHostPointerIdentity,
    position: UiHostSurfacePosition,
    transition: Option<UiHostPointerButtonTransition>,
    held: bool,
) -> UiHostObservationBatch {
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("current protocol")
    };
    let capture_epoch = UiHostPointerCaptureEpoch::new(1);
    let payload = match transition {
        Some(transition) => UiHostObservationPayload::PointerButton {
            pointer,
            capture_epoch,
            button: UiHostPointerButton::Primary,
            transition,
            position,
        },
        None => UiHostObservationPayload::PointerMotion {
            pointer,
            capture_epoch,
            position,
            pressed_buttons: if held {
                UiHostPressedPointerButtons::from_buttons([UiHostPointerButton::Primary])
            } else {
                UiHostPressedPointerButtons::NONE
            },
        },
    };
    let sequence = UiHostObservationSequence::new(sequence);
    UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session,
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            payload,
        )
        .with_pointer_device_kind(UiHostPointerDeviceKind::Mouse)
        .unwrap()],
    })
    .unwrap()
}
