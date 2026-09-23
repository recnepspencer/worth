use super::*;
use winit::event::{ElementState, MouseButton};
use worth_ui_host_contract::{UiHostPointerButton, UiHostPointerButtonTransition};

#[test]
fn held_pointer_capture_survives_client_exit_until_outside_release() {
    let mut state = presented_state();
    button(&mut state, ElementState::Pressed, 10.0);
    let epoch = state.pointer.capture_epoch();
    state.observe_window_event(&WindowEvent::CursorLeft {
        device_id: DeviceId::dummy(),
    });
    assert_eq!(state.pointer.capture_epoch(), epoch);
    motion(&mut state, 620.0);
    button(&mut state, ElementState::Released, 620.0);
    motion(&mut state, 20.0);
    let batches = state.drain(HOST_SESSION).into_batches();
    let reports: Vec<_> = batches.iter().flat_map(|batch| batch.reports()).collect();
    assert_eq!(reports.len(), 4);
    assert!(
        matches!(reports[0].payload(), UiHostObservationPayload::PointerButton {
        capture_epoch, transition: UiHostPointerButtonTransition::Pressed, ..
    } if capture_epoch.value() == epoch)
    );
    assert!(
        matches!(reports[1].payload(), UiHostObservationPayload::PointerMotion {
        capture_epoch, pressed_buttons, position, ..
    } if capture_epoch.value() == epoch
        && pressed_buttons.contains(UiHostPointerButton::Primary)
        && position.y_subpixels() == 620_000)
    );
    assert!(
        matches!(reports[2].payload(), UiHostObservationPayload::PointerButton {
        capture_epoch, transition: UiHostPointerButtonTransition::Released, position, ..
    } if capture_epoch.value() == epoch && position.y_subpixels() == 620_000)
    );
    assert!(
        matches!(reports[3].payload(), UiHostObservationPayload::PointerMotion {
        pressed_buttons, ..
    } if pressed_buttons.bits() == 0)
    );
    state.observe_window_event(&WindowEvent::CursorLeft {
        device_id: DeviceId::dummy(),
    });
    assert_eq!(state.pointer.capture_epoch(), epoch + 1);
    assert_eq!(state.report().terminal_stop(), None);
}

#[test]
fn held_pointer_capture_is_still_cancelled_by_focus_loss() {
    let mut state = presented_state();
    button(&mut state, ElementState::Pressed, 10.0);
    let epoch = state.pointer.capture_epoch();
    state.observe_window_event(&WindowEvent::Focused(false));
    assert_eq!(state.pointer.capture_epoch(), epoch + 1);
    assert!(!state.pointer.has_pressed_buttons());
    assert_eq!(state.report().terminal_stop(), None);
}

fn button(state: &mut UiNativeInputObservationState, transition: ElementState, y: f64) {
    state.observe_window_event_at_with_pointer_witness(
        &WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state: transition,
            button: MouseButton::Left,
        },
        1,
        pointer::UiNativePointerPositionWitness::EventTime(PhysicalPosition::new(10.0, y)),
    );
}

fn motion(state: &mut UiNativeInputObservationState, y: f64) {
    state.observe_window_event(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(10.0, y),
    });
}
