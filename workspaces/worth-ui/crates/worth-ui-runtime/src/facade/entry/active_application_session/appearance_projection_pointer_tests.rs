use super::{mounting_fixture, support, test_support};
use worth_ui_host_contract::*;

#[path = "appearance_projection_pointer_fixture.rs"]
mod fixture;
#[path = "appearance_owner_locality_tests.rs"]
mod locality_tests;
#[path = "appearance_pointer_presentation_owner_tests.rs"]
mod presentation_owner_tests;

#[test]
fn pointer_owner_state_reaches_unpublished_appearance_on_observation_close() {
    let role = fixture::role();
    let (mut session, host) = fixture::session(&role);
    let (surface, graph_node) = mounting_fixture::mount(&mut session, 1_000);
    close_source_turn(&mut session, &role, "pointer-appearance-initial");
    session.advance_mounted_identity_frame().unwrap();
    publish(&mut session, &host, 1, 10, None);
    let initial = session
        .mounted
        .current_publication()
        .unwrap()
        .presentation_for_surface(surface)
        .unwrap();
    let hit_test = session.mounted.interaction_hit_test_basis(initial).unwrap();
    let identity = session.inspect_mounted_identity();
    let instance = identity
        .mounted_instances()
        .iter()
        .find(|row| row.graph_node_identity() == graph_node)
        .unwrap()
        .identity();
    let row = hit_test
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == instance)
        .expect("appearance target is hit-testable");
    let bounds = row.bounds();
    let clip = row.clip_bounds();
    let captured_incarnation = crate::mounting::UiMountedIncarnationAffinityInput {
        surface,
        binding: initial.binding(),
        mounted_instance: instance,
    };
    assert_eq!(
        session
            .mounted
            .current_presented_incarnation_receipt(captured_incarnation, initial),
        Ok(row.node_receipt()),
    );
    let inside = UiHostSurfacePosition::viewport_logical(
        ((bounds.x().max(clip.x()) + (bounds.x() + bounds.width()).min(clip.x() + clip.width()))
            * 500.0) as i64,
        ((bounds.y().max(clip.y()) + (bounds.y() + bounds.height()).min(clip.y() + clip.height()))
            * 500.0) as i64,
    );
    let outside = UiHostSurfacePosition::viewport_logical(-1_000, -1_000);
    for (sequence, position, button, held, expected_red, expected_pressed, pointer_output) in [
        (1, inside, None, false, 20, None, Some(true)),
        (
            2,
            inside,
            Some(UiHostPointerButtonTransition::Pressed),
            true,
            30,
            Some(crate::runtime::interaction::gesture::UiPressedAppearanceClass::ArmedInside),
            None,
        ),
        (
            3,
            outside,
            None,
            true,
            40,
            Some(crate::runtime::interaction::gesture::UiPressedAppearanceClass::CapturedOutside),
            Some(false),
        ),
        (
            4,
            inside,
            None,
            true,
            30,
            Some(crate::runtime::interaction::gesture::UiPressedAppearanceClass::ArmedInside),
            Some(true),
        ),
        (
            5,
            outside,
            Some(UiHostPointerButtonTransition::Released),
            false,
            10,
            None,
            Some(false),
        ),
    ] {
        let presentation = session
            .mounted
            .current_publication()
            .unwrap()
            .presentation_for_surface(surface)
            .unwrap();
        let batch = pointer_batch(
            session.host_session.identity().as_u64(),
            presentation,
            sequence,
            UiHostPointerIdentity::new(1),
            position,
            button,
            held,
        );
        if sequence > 1 {
            assert_eq!(
                session
                    .mounted
                    .current_presented_incarnation_receipt(captured_incarnation, initial),
                Err(crate::mounting::UiCurrentHitTargetAffinityDenial::PresentationNotCurrent),
            );
        }
        let ingress = session.admit_host_interaction_batch(batch);
        let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = ingress
        else {
            panic!("pointer sequence {sequence} must reach the interaction owner: {ingress:?}");
        };
        assert!(receipt.pointer_presence_denials().is_empty());
        let pressed = session.interaction.pressed_appearance_snapshot();
        assert_eq!(
            pressed.postures().first().map(|posture| posture.class()),
            expected_pressed
        );
        if let Some(posture) = pressed.postures().first() {
            assert_eq!(posture.presentation(), presentation);
            assert_eq!(posture.target(), instance);
            assert_eq!(posture.press_sequence().value(), 2);
        }
        // A real source observation closes the coherent snapshot after host ingress.
        // This proves state carriage, not native event-loop scheduling (Gate 6).
        close_source_turn(
            &mut session,
            &role,
            &format!("pointer-appearance-{sequence}"),
        );
        publish(
            &mut session,
            &host,
            sequence + 1,
            expected_red,
            pointer_output.map(|present| (surface, instance, present)),
        );
        let output = session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .unwrap();
        let UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            successor: Some(receipt),
            ..
        } = output.fragments()[0].identity()
        else {
            panic!("pointer appearance must retain mounted attribution");
        };
        assert_eq!(receipt.mounted_instance(), instance);
        assert!(host
            .last_filled_rect_colors()
            .iter()
            .all(|color| color.channels() == [17, 34, 51, 255]));
    }
    let _ = session.shutdown();
}

fn close_source_turn(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    source_name: &str,
) {
    let source = support::appearance_candidate_submission(session, source_name, Some(role));
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    session.classify_observations(admitted).unwrap();
}

fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    now: u64,
    expected_red: u8,
    pointer: Option<(UiSemanticSurfaceIdentity, UiMountedInstanceIdentity, bool)>,
) {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("pointer appearance frame must prepare"));
    if now == 1 {
        host.push_native_display_presented();
    } else {
        host.push_native_display_settled_without_effects();
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
            panic!("turn {now} retention denial: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            panic!("turn {now} admission denial: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) => {
            panic!("turn {now} completion denial: {denial:?}")
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("turn {now} rejected before effects")
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(_) => panic!("turn {now} in flight"),
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("turn {now} indeterminate")
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => panic!("turn {now} superseded"),
        crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
            panic!("turn {now} unexpectedly reconciled")
        }
    }
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap_or_else(|denial| panic!("turn {now} pointer owner basis must admit: {denial:?}"))
        .expect("pointer state must have appearance output");
    test_support::assert_unpublished_surface_with_pointer(
        output,
        [expected_red, 0, 0, 255],
        pointer,
    );
}

pub(super) fn pointer_batch(
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
