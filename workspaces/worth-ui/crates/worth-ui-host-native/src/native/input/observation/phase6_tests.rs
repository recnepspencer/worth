use super::*;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, ElementState, MouseButton, WindowEvent};
use worth_ui_host_contract::{
    UiHostObservationPayload, UiHostObservationTimeBasis, UiHostPointerDeviceKind,
    UiHostPresentationEpoch, UiHostProtocolContract, UiHostProtocolNegotiation,
    UiMountedFrameIdentity, UiSurfaceBindingGeneration,
};

const HOST_SESSION: u64 = 73;

#[test]
fn presentation_requires_the_explicit_session_open_edge() {
    let mut state = UiNativeInputObservationState::new();
    state.install_initial_profile(1.0, [800, 600]);

    assert!(!state.record_completed_presentation(protocol(), HOST_SESSION, basis(1)));
    assert_eq!(state.report().last_completed_presentation(), None);
    assert_eq!(
        state.report().terminal_stop(),
        Some(UiNativeInputObservationStop::Retention(
            worth_ui_host_contract::UiHostObservationRetentionDenial::InactiveSession,
        ))
    );
}

#[test]
fn event_time_is_independent_from_observation_sequence() {
    let mut state = presented_state();
    state.observe_window_event_at(
        &WindowEvent::CursorMoved {
            device_id: DeviceId::dummy(),
            position: PhysicalPosition::new(10.0, 20.0),
        },
        77,
    );

    assert!(state.has_retained_observations());
    let batches = state.drain(HOST_SESSION).into_batches();
    let report = &batches[0].reports()[0];
    assert_eq!(report.sequence().value(), 1);
    assert_eq!(
        report.time_basis(),
        UiHostObservationTimeBasis::HostMonotonicMillis(77)
    );
    assert_eq!(
        report.pointer_device_kind(),
        Some(UiHostPointerDeviceKind::Mouse)
    );
    assert_eq!(state.report().retained_batch_count(), 1);
    assert_eq!(state.report().retained_event_count(), 1);
    assert_eq!(state.report().first_retained_sequence(), Some(1));
    assert_eq!(state.report().last_retained_sequence(), Some(1));
    assert!(!state.has_retained_observations());
}

#[test]
fn resize_observation_keeps_the_resize_event_tick_after_completion() {
    let mut state = presented_state();
    state.observe_profile_transition_at(1.5, [1200, 800], 41);
    assert!(state.record_completed_presentation(protocol(), HOST_SESSION, basis(2)));

    let batches = state.drain(HOST_SESSION).into_batches();
    assert_eq!(batches.len(), 1);
    assert!(batches[0]
        .reports()
        .iter()
        .all(|report| report.time_basis() == UiHostObservationTimeBasis::HostMonotonicMillis(41)));
    assert_eq!(state.report().profile_transition_count(), 1);
}

#[test]
fn pending_successor_contexts_are_not_dropped_by_another_completion() {
    let mut state = UiNativeInputObservationState::new();
    state.install_initial_profile(1.0, [800, 600]);
    state.register_session(HOST_SESSION).unwrap();
    let first_binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let second_binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let host_surface = worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap();
    assert!(state.remember_pending_presentation(
        protocol(),
        HOST_SESSION,
        host_surface,
        first_binding,
        91,
    ));
    assert!(state.remember_pending_presentation(
        protocol(),
        HOST_SESSION,
        host_surface,
        second_binding,
        92,
    ));

    assert!(state.complete_pending_presentation(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        first_binding,
        UiHostPresentationEpoch::issued_by_host(2),
        91,
    ));
    assert!(state.complete_pending_presentation(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        second_binding,
        UiHostPresentationEpoch::issued_by_host(3),
        92,
    ));
    assert_eq!(state.report().terminal_stop(), None);
    assert_eq!(state.report().completed_presentation_count(), 2);
}

#[test]
fn button_event_uses_the_event_time_position_witness() {
    let mut state = presented_state();
    state.observe_window_event_at_with_pointer_witness(
        &WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        },
        19,
        super::pointer::UiNativePointerPositionWitness::EventTime(PhysicalPosition::new(
            12.0, 24.0,
        )),
    );

    let batches = state.drain(HOST_SESSION).into_batches();
    assert!(matches!(
        batches[0].reports()[0].payload(),
        UiHostObservationPayload::PointerButton { position, .. }
            if position.x_subpixels() == 12_000 && position.y_subpixels() == 24_000
    ));
    assert_eq!(
        batches[0].reports()[0].pointer_device_kind(),
        Some(UiHostPointerDeviceKind::Mouse)
    );
    assert_eq!(
        batches[0].reports()[0].time_basis(),
        UiHostObservationTimeBasis::HostMonotonicMillis(19)
    );
    let button = state
        .report()
        .last_pointer_button()
        .expect("retained button evidence");
    assert_eq!(button.sequence(), 1);
    assert_eq!(button.event_tick(), 19);
    assert_eq!(button.x_subpixels(), 12_000);
    assert_eq!(button.y_subpixels(), 24_000);
}

#[test]
fn failed_retention_does_not_create_a_sequence_gap() {
    let mut state = presented_state();
    for index in 0..17 {
        state.observe_window_event(&WindowEvent::CursorMoved {
            device_id: DeviceId::dummy(),
            position: PhysicalPosition::new(index as f64, 0.0),
        });
    }

    let reports = state
        .drain(HOST_SESSION)
        .into_batches()
        .into_vec()
        .into_iter()
        .flat_map(|batch| batch.reports().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(reports.len(), 16);
    assert_eq!(state.report().retained_event_count(), 16);
    assert_eq!(state.report().last_retained_sequence(), Some(16));
    assert!(reports
        .iter()
        .enumerate()
        .all(|(index, report)| report.sequence().value() == index as u64 + 1));
}

#[test]
fn cursor_left_before_first_presentation_denies_before_capture_mutation() {
    let mut state = UiNativeInputObservationState::new();
    state.install_initial_profile(1.0, [800, 600]);
    let before = state.pointer.capture_epoch();
    state.observe_window_event(&WindowEvent::CursorLeft {
        device_id: DeviceId::dummy(),
    });

    assert_eq!(state.pointer.capture_epoch(), before);
    assert_eq!(
        state.report().stops().last().copied(),
        Some(UiNativeInputObservationStop::NoPresentationBasis)
    );
    assert_eq!(state.report().terminal_stop(), None);
}

#[test]
fn cursor_left_epoch_exhaustion_cannot_mask_missing_presentation_basis() {
    let mut state = UiNativeInputObservationState::new();
    state.install_initial_profile(1.0, [800, 600]);
    state.pointer.set_capture_epoch_for_test(u64::MAX);
    state.observe_window_event(&WindowEvent::CursorLeft {
        device_id: DeviceId::dummy(),
    });

    assert_eq!(
        state.report().stops().last().copied(),
        Some(UiNativeInputObservationStop::NoPresentationBasis)
    );
    assert_eq!(state.report().terminal_stop(), None);
    assert_eq!(state.pointer.capture_epoch(), u64::MAX);
}

fn presented_state() -> UiNativeInputObservationState {
    let mut state = UiNativeInputObservationState::new();
    state.install_initial_profile(1.0, [800, 600]);
    state.register_session(HOST_SESSION).unwrap();
    assert!(state.record_completed_presentation(protocol(), HOST_SESSION, basis(1)));
    state
}

fn protocol() -> worth_ui_host_contract::UiHostProtocolAgreement {
    match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(agreement) => agreement,
        UiHostProtocolNegotiation::Incompatible(_) => unreachable!(),
    }
}

fn basis(epoch: u64) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
    worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        UiHostPresentationEpoch::issued_by_host(epoch),
    )
}
