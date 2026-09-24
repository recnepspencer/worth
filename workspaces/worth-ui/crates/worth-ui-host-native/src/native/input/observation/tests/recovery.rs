use super::*;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase};
use worth_ui_host_contract::UiHostObservationSequence;

fn overflow(state: &mut UiNativeInputObservationState) {
    for _ in 0..=worth_ui_host_contract::UI_HOST_OBSERVATION_DRAIN_BATCH_LIMIT {
        state.observe_window_event_at_with_pointer_witness(
            &WindowEvent::MouseWheel {
                device_id: DeviceId::dummy(),
                delta: MouseScrollDelta::LineDelta(0.0, -1.0),
                phase: TouchPhase::Moved,
            },
            1,
            pointer::UiNativePointerPositionWitness::EventTime(PhysicalPosition::new(10.0, 20.0)),
        );
    }
    assert!(state
        .report()
        .terminal_stop()
        .unwrap()
        .permits_retention_recovery());
}

#[test]
fn recovery_requires_drained_prefix_and_acknowledgement_and_preserves_sequence() {
    let mut state = presented_state_without_recipient();
    state.observe_window_event_at_with_pointer_witness(
        &WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        },
        1,
        pointer::UiNativePointerPositionWitness::EventTime(PhysicalPosition::new(10.0, 20.0)),
    );
    assert!(state.pointer.has_pressed_buttons());
    let capture = state.pointer.capture_epoch();
    let presentation = state.report().last_completed_presentation();
    overflow(&mut state);
    assert!(state.begin_retention_recovery().is_none());
    assert_eq!(state.drain(HOST_SESSION).into_batches().len(), 16);
    let grant = state.begin_retention_recovery().unwrap();
    assert_eq!(grant.host_session(), HOST_SESSION);
    assert!(
        state.report().terminal_stop().is_some(),
        "issuance alone cannot resume input"
    );
    assert!(state.complete_retention_recovery(grant.acknowledge_cancellation()));
    assert!(!state.pointer.has_pressed_buttons());
    assert_eq!(state.pointer.capture_epoch(), capture + 1);
    assert_eq!(state.report().terminal_stop(), None);
    assert_eq!(state.report().last_completed_presentation(), presentation);
    assert!(
        !state.report().stops().is_empty(),
        "recovery must preserve loss evidence"
    );
    state.observe_window_event(&WindowEvent::Focused(true));
    let batches = state.drain(HOST_SESSION).into_batches();
    assert_eq!(batches.len(), 1);
    assert_eq!(
        batches[0].reports()[0].sequence(),
        UiHostObservationSequence::new(17)
    );
}

#[test]
fn stale_acknowledgement_cannot_resume_a_later_exhaustion() {
    let mut state = presented_state_without_recipient();
    overflow(&mut state);
    state.drain(HOST_SESSION);
    let stale = state
        .begin_retention_recovery()
        .unwrap()
        .acknowledge_cancellation();
    let current = state
        .begin_retention_recovery()
        .unwrap()
        .acknowledge_cancellation();
    assert!(state.complete_retention_recovery(current));
    overflow(&mut state);
    state.drain(HOST_SESSION);
    assert!(!state.complete_retention_recovery(stale));
    assert!(state.report().terminal_stop().is_some());
}

#[test]
fn fatal_completion_error_dominates_prior_capacity_stop() {
    let mut state = presented_state_without_recipient();
    overflow(&mut state);
    state.drain(HOST_SESSION);
    let acknowledgement = state
        .begin_retention_recovery()
        .unwrap()
        .acknowledge_cancellation();
    assert!(!state.complete_pending_presentation(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        UiHostPresentationEpoch::issued_by_host(2),
        999,
    ));
    assert_eq!(
        state.report().terminal_stop(),
        Some(UiNativeInputObservationStop::MissingPendingPresentationContext)
    );
    assert!(state.begin_retention_recovery().is_none());
    assert!(!state.complete_retention_recovery(acknowledgement));
}

#[test]
fn recovery_cannot_wrap_capture_epoch() {
    let mut state = presented_state_without_recipient();
    overflow(&mut state);
    state.drain(HOST_SESSION);
    state.pointer.set_capture_epoch_for_test(u64::MAX);
    let acknowledgement = state
        .begin_retention_recovery()
        .unwrap()
        .acknowledge_cancellation();
    assert!(!state.complete_retention_recovery(acknowledgement));
    assert_eq!(
        state.report().terminal_stop(),
        Some(UiNativeInputObservationStop::PointerCaptureEpochExhausted)
    );
}

#[test]
fn recovery_keeps_latest_modifiers_and_only_emits_completed_viewport() {
    for completed in [false, true] {
        let mut state = presented_state_without_recipient();
        state.observe_window_event(&WindowEvent::ModifiersChanged(
            winit::keyboard::ModifiersState::SHIFT.into(),
        ));
        overflow(&mut state);
        state.observe_window_event(&WindowEvent::ModifiersChanged(
            winit::keyboard::ModifiersState::empty().into(),
        ));
        state.observe_profile_transition_at(2.0, [1200, 800], 42);
        if completed {
            assert!(state.record_completed_presentation(protocol(), HOST_SESSION, basis(2)));
        }
        assert_eq!(state.drain(HOST_SESSION).into_batches().len(), 16);
        let acknowledgement = state
            .begin_retention_recovery()
            .unwrap()
            .acknowledge_cancellation();
        assert!(state.complete_retention_recovery(acknowledgement));
        assert_eq!(state.modifiers, UiHostKeyboardModifiers::default());
        if !completed {
            assert!(state.drain(HOST_SESSION).into_batches().is_empty());
            assert!(state.profile_requires_completion);
            assert!(state.record_completed_presentation(protocol(), HOST_SESSION, basis(2)));
        }
        let batches = state.drain(HOST_SESSION).into_batches();
        assert_eq!(batches.len(), 1);
        assert!(matches!(
            batches[0].reports()[0].payload(),
            UiHostObservationPayload::Viewport {
                width_subpixels: 600_000,
                height_subpixels: 400_000
            }
        ));
        assert!(matches!(
            batches[0].reports()[1].payload(),
            UiHostObservationPayload::DeviceScale { micros: 2_000_000 }
        ));
        assert_eq!(
            batches[0].reports()[0].sequence(),
            UiHostObservationSequence::new(17)
        );
        assert!(!state.profile_requires_completion);
    }
}
