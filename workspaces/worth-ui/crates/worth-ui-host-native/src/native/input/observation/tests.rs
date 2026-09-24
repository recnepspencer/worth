use super::*;
use winit::dpi::PhysicalPosition;
use winit::event::DeviceId;
use winit::event::{Ime, WindowEvent};
use worth_ui_host_contract::{
    UiHostImeCompositionPhase, UiHostImePreeditSelection, UiHostObservationDrainDenial,
    UiHostObservationPayload, UiHostObservationPresentationBasis, UiHostPresentationEpoch,
    UiHostProtocolContract, UiHostProtocolNegotiation, UiMountedFrameIdentity,
    UiSurfaceBindingGeneration,
};

mod delayed_drain;
mod pointer_capture;
mod recipient_absence;
mod recovery;
mod scroll;

const HOST_SESSION: u64 = 73;

#[test]
fn input_before_first_completed_presentation_is_typed_without_basis() {
    let mut state = UiNativeInputObservationState::new();
    state.install_initial_profile(1.0, [800, 600]);
    state.observe_window_event(&winit::event::WindowEvent::Focused(true));
    assert_eq!(state.report().terminal_stop(), None);
    assert!(state
        .report()
        .stops()
        .contains(&UiNativeInputObservationStop::NoPresentationBasis));
}

#[test]
fn completed_affinity_and_event_profile_are_carried_in_order() {
    let mut state = presented_state();
    let expected_presentation = state
        .report()
        .last_completed_presentation()
        .expect("completed presentation");
    state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(10.0, 20.0),
    });
    state.observe_profile_transition(1.5, [1200, 800]);
    state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(20.0, 30.0),
    });
    let predecessor_batches = state.drain(HOST_SESSION).into_batches();
    assert_eq!(predecessor_batches.len(), 1);
    assert_eq!(
        predecessor_batches[0].canonical_core().presentation(),
        expected_presentation
    );
    assert!(matches!(
        predecessor_batches[0].reports()[0].payload(),
        UiHostObservationPayload::PointerMotion { .. }
    ));
    assert!(state
        .report()
        .stops()
        .contains(&UiNativeInputObservationStop::StalePresentationAffinity));

    let successor = basis(2);
    state.record_completed_presentation(protocol(), HOST_SESSION, successor);
    let transition_batches = state.drain(HOST_SESSION).into_batches();
    assert_eq!(transition_batches.len(), 1);
    assert!(matches!(
        transition_batches[0].reports()[0].payload(),
        UiHostObservationPayload::Viewport {
            width_subpixels: 800_000,
            height_subpixels: 533_333,
        }
    ));
    assert!(matches!(
        transition_batches[0].reports()[1].payload(),
        UiHostObservationPayload::DeviceScale { micros: 1_500_000 }
    ));
}

#[test]
fn successor_presentation_takes_affinity_only_after_completion() {
    let mut state = presented_state();
    let predecessor = state
        .report()
        .last_completed_presentation()
        .expect("predecessor presentation");
    state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(10.0, 20.0),
    });

    let successor = basis(2);
    state.record_completed_presentation(protocol(), HOST_SESSION, successor);
    state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(30.0, 40.0),
    });

    let batches = state.drain(HOST_SESSION).into_batches();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].canonical_core().presentation(), predecessor);
    assert_eq!(batches[1].canonical_core().presentation(), successor);
}

#[test]
fn pending_completion_identity_is_the_only_successor_affinity_witness() {
    let mut state = UiNativeInputObservationState::new();
    state.install_initial_profile(1.0, [800, 600]);
    state.register_session(HOST_SESSION).unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let completion_identity = 91;
    assert!(state.remember_pending_presentation(
        protocol(),
        HOST_SESSION,
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        binding,
        completion_identity,
    ));

    state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(10.0, 20.0),
    });
    assert!(state.drain(HOST_SESSION).into_batches().is_empty());
    assert!(state.complete_pending_presentation(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        binding,
        UiHostPresentationEpoch::issued_by_host(2),
        completion_identity,
    ));

    state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(30.0, 40.0),
    });
    let batches = state.drain(HOST_SESSION).into_batches();
    assert_eq!(batches.len(), 1);
    assert_eq!(
        batches[0].canonical_core().presentation(),
        state
            .report()
            .last_completed_presentation()
            .expect("pending completion establishes the only affinity")
    );
}

#[test]
fn abandoned_pending_identity_cannot_complete_later() {
    let mut state = UiNativeInputObservationState::new();
    state.install_initial_profile(1.0, [800, 600]);
    state.register_session(HOST_SESSION).unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let completion_identity = 92;
    assert!(state.remember_pending_presentation(
        protocol(),
        HOST_SESSION,
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        binding,
        completion_identity,
    ));
    state.abandon_pending_presentation(binding, Some(completion_identity));

    assert!(!state.complete_pending_presentation(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        binding,
        UiHostPresentationEpoch::issued_by_host(2),
        completion_identity,
    ));
    assert_eq!(
        state.report().terminal_stop(),
        Some(UiNativeInputObservationStop::MissingPendingPresentationContext)
    );
}

#[test]
fn ime_keeps_preedit_commit_and_cancel_distinct_and_converts_bytes() {
    let mut state = presented_state();
    state.observe_window_event(&WindowEvent::Ime(Ime::Preedit("aé🦀".into(), Some((1, 3)))));
    state.observe_window_event(&WindowEvent::Ime(Ime::Preedit("".into(), None)));
    state.observe_window_event(&WindowEvent::Ime(Ime::Commit("done".into())));
    let batches = state.drain(HOST_SESSION).into_batches();
    assert_eq!(batches.len(), 3);
    assert!(matches!(
        batches[0].reports()[0].payload(),
        UiHostObservationPayload::ImeComposition {
            phase: UiHostImeCompositionPhase::Preedit(preedit),
            ..
        } if matches!(preedit.selection(), UiHostImePreeditSelection::Converted(receipt)
            if receipt.source().start() == 1
                && receipt.source().end() == 2
                && receipt.canonical().start() == 1
                && receipt.canonical().end() == 3)
    ));
    assert!(matches!(
        batches[1].reports()[0].payload(),
        UiHostObservationPayload::ImeComposition {
            phase: UiHostImeCompositionPhase::Cancel,
            ..
        }
    ));
    assert!(matches!(
        batches[2].reports()[0].payload(),
        UiHostObservationPayload::ImeComposition {
            phase: UiHostImeCompositionPhase::Commit(text),
            ..
        } if text.as_ref() == "done"
    ));
    assert!(batches
        .iter()
        .flat_map(|batch| batch.reports())
        .all(|report| !matches!(report.payload(), UiHostObservationPayload::TextInput { .. })));
}

#[test]
fn unprovable_ime_byte_range_stops_before_retention() {
    let mut state = presented_state();
    state.observe_window_event(&WindowEvent::Ime(Ime::Preedit("é".into(), Some((1, 1)))));
    assert_eq!(
        state.report().stops().last(),
        Some(&UiNativeInputObservationStop::ImeRangeNotScalarBoundary)
    );
    assert_eq!(state.report().terminal_stop(), None);
    assert!(state.drain(HOST_SESSION).into_batches().is_empty());
}

#[test]
fn button_without_a_cursor_witness_is_a_typed_stop() {
    let mut state = presented_state();
    state.observe_window_event(&WindowEvent::MouseInput {
        device_id: DeviceId::dummy(),
        state: winit::event::ElementState::Pressed,
        button: winit::event::MouseButton::Left,
    });
    assert_eq!(
        state.report().terminal_stop(),
        Some(UiNativeInputObservationStop::PointerPositionUnavailable)
    );
}

#[test]
fn retention_over_capacity_is_terminal_and_does_not_overwrite() {
    let mut state = presented_state();
    for index in 0..17 {
        state.observe_window_event(&WindowEvent::Focused(index % 2 == 0));
    }
    assert_eq!(
        state.report().terminal_stop(),
        Some(UiNativeInputObservationStop::Retention(
            worth_ui_host_contract::UiHostObservationRetentionDenial::Capacity(
                UiHostObservationDrainDenial::BatchCapacityExceeded,
            ),
        ))
    );
    assert_eq!(state.drain(HOST_SESSION).into_batches().len(), 16);
}

#[test]
fn releasing_a_session_resets_sequence_revision_and_pointer_witness() {
    let mut state = presented_state();
    state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(10.0, 20.0),
    });
    let _ = state.drain(HOST_SESSION);
    state.release_session(HOST_SESSION);
    state.register_session(HOST_SESSION + 1).unwrap();
    state.install_initial_profile(1.0, [800, 600]);
    state.record_completed_presentation(protocol(), HOST_SESSION + 1, basis(2));
    state.observe_window_event(&WindowEvent::MouseInput {
        device_id: DeviceId::dummy(),
        state: winit::event::ElementState::Pressed,
        button: winit::event::MouseButton::Left,
    });
    assert_eq!(
        state.report().terminal_stop(),
        Some(UiNativeInputObservationStop::PointerPositionUnavailable)
    );

    let mut sequence_state = presented_state();
    sequence_state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(10.0, 20.0),
    });
    let _ = sequence_state.drain(HOST_SESSION);
    sequence_state.release_session(HOST_SESSION);
    sequence_state.register_session(HOST_SESSION + 1).unwrap();
    sequence_state.install_initial_profile(1.0, [800, 600]);
    sequence_state.record_completed_presentation(protocol(), HOST_SESSION + 1, basis(2));
    sequence_state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(3.0, 4.0),
    });
    let retained = sequence_state
        .drain(HOST_SESSION + 1)
        .into_batches()
        .into_vec();
    let batch = retained
        .into_iter()
        .next()
        .expect("successor session retains its first observation");
    assert_eq!(batch.reports()[0].sequence().value(), 1);
}

#[test]
fn close_releases_retained_input_before_clearing_affinity() {
    let mut state = presented_state();
    state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(10.0, 20.0),
    });
    state.close();

    assert!(state.drain(HOST_SESSION).into_batches().is_empty());
    assert_eq!(state.report().last_completed_presentation(), None);
}

#[test]
fn ime_disable_cancels_active_preedit_without_text_payload() {
    let mut state = presented_state();
    state.observe_window_event(&WindowEvent::Ime(Ime::Preedit("draft".into(), None)));
    state.observe_window_event(&WindowEvent::Ime(Ime::Disabled));
    let batches = state.drain(HOST_SESSION).into_batches();
    assert_eq!(batches.len(), 2);
    assert!(matches!(
        batches[1].reports()[0].payload(),
        UiHostObservationPayload::ImeComposition {
            phase: UiHostImeCompositionPhase::Cancel,
            ..
        }
    ));
}

fn presented_state() -> UiNativeInputObservationState {
    let mut state = presented_state_without_recipient();
    let presentation = state
        .report()
        .last_completed_presentation()
        .expect("completed presentation");
    assert!(state.install_input_recipient(draft_binding(presentation)));
    state
}

fn presented_state_without_recipient() -> UiNativeInputObservationState {
    let mut state = UiNativeInputObservationState::new();
    state.install_initial_profile(1.0, [800, 600]);
    state.register_session(HOST_SESSION).unwrap();
    let presentation = basis(1);
    state.record_completed_presentation(protocol(), HOST_SESSION, presentation);
    state
}

fn draft_binding(
    presentation: UiHostObservationPresentationBasis,
) -> worth_ui_host_contract::UiHostInputRecipientBindingReceipt {
    let instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let node_receipt =
        worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(presentation.frame())
            .unwrap()
            .receipt_for(instance);
    worth_ui_host_contract::UiHostInputRecipientBindingReceipt::new(
        worth_ui_host_contract::UiHostInputRecipientBindingInput {
            host_session: HOST_SESSION,
            application_generation: worth_ui_host_contract::UiHostApplicationGeneration::new(1)
                .unwrap(),
            recipient_generation: worth_ui_host_contract::UiHostInputRecipientGeneration::new(1)
                .unwrap(),
            family: worth_ui_host_contract::UiHostInputRecipientFamily::Draft,
            draft_session: Some(
                worth_ui_host_contract::UiHostInputDraftSessionIdentity::new(1).unwrap(),
            ),
            surface: worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            binding: presentation.binding(),
            mounted_instance: instance,
            node_receipt,
            text_profile: Some(worth_ui_host_contract::UiTextProfileGeneration::new(1).unwrap()),
        },
    )
}

fn protocol() -> worth_ui_host_contract::UiHostProtocolAgreement {
    match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    }
}

fn basis(epoch: u64) -> UiHostObservationPresentationBasis {
    UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        UiHostPresentationEpoch::issued_by_host(epoch),
    )
}

#[path = "tests/focus.rs"]
mod focus;
