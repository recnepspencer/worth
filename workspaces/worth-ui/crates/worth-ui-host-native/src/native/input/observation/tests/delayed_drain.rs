//! Diagnostic boundary reproduction. The consumer's delayed drain is scripted;
//! native event translation, retention and completion are production code. This
//! does not establish that the live application's frozen run exhausted retention.
use super::{presented_state_without_recipient, protocol, HOST_SESSION};
use crate::native::{
    UiNativeInputObservationDisposition as Disposition, UiNativeInputObservationStop,
    UiNativePointerPositionWitness,
};
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceId, ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
};
use worth_ui_host_contract::{
    UiHostObservationDrainDenial, UiHostObservationRetentionDenial, UiHostPresentationEpoch,
    UiMountedFrameIdentity, UI_HOST_OBSERVATION_DRAIN_BATCH_LIMIT,
};

#[test]
fn scroll_burst_past_delayed_drain_capacity_disables_fresh_input_after_completion() {
    // The at-capacity control distinguishes pending presentation from overflow.
    for overflow in [false, true] {
        let mut input = presented_state_without_recipient();
        let predecessor = input.report().last_completed_presentation().unwrap();
        let completion = 91;
        assert!(input.remember_pending_presentation(
            protocol(),
            HOST_SESSION,
            predecessor.host_surface(),
            predecessor.binding(),
            completion,
        ));
        let count = UI_HOST_OBSERVATION_DRAIN_BATCH_LIMIT + usize::from(overflow);
        for index in 0..count {
            let disposition = input.observe_window_event_at_with_pointer_witness(
                &WindowEvent::MouseWheel {
                    device_id: DeviceId::dummy(),
                    delta: MouseScrollDelta::LineDelta(0.0, -1.0),
                    phase: TouchPhase::Moved,
                },
                index as u64,
                UiNativePointerPositionWitness::EventTime(PhysicalPosition::new(100.0, 100.0)),
            );
            assert_eq!(
                disposition,
                if index < UI_HOST_OBSERVATION_DRAIN_BATCH_LIMIT {
                    Disposition::Retained
                } else {
                    Disposition::Stopped
                },
            );
        }
        assert_eq!(
            input.report().terminal_stop(),
            overflow.then_some(UiNativeInputObservationStop::Retention(
                UiHostObservationRetentionDenial::Capacity(
                    UiHostObservationDrainDenial::BatchCapacityExceeded,
                ),
            )),
        );

        // Remove both possible blockers: complete the exact pending presentation
        // and consume every retained event. The overflowed session stays dead.
        assert!(input.complete_pending_presentation(
            UiMountedFrameIdentity::mint_unbound().unwrap(),
            predecessor.binding(),
            UiHostPresentationEpoch::issued_by_host(2),
            completion,
        ));
        assert_eq!(
            input.drain(HOST_SESSION).into_batches().len(),
            UI_HOST_OBSERVATION_DRAIN_BATCH_LIMIT,
        );
        let click = input.observe_window_event_at_with_pointer_witness(
            &WindowEvent::MouseInput {
                device_id: DeviceId::dummy(),
                state: ElementState::Pressed,
                button: MouseButton::Left,
            },
            100,
            UiNativePointerPositionWitness::EventTime(PhysicalPosition::new(100.0, 100.0)),
        );
        let escape = input.observe_keyboard_components_at(
            &Key::Named(NamedKey::Escape),
            PhysicalKey::Code(KeyCode::Escape),
            ElementState::Pressed,
            false,
            None,
            101,
        );
        let expected = if overflow {
            Disposition::Stopped
        } else {
            Disposition::Retained
        };
        assert_eq!(click, expected);
        assert_eq!(escape, expected);
        assert_eq!(
            input.drain(HOST_SESSION).into_batches().len(),
            if overflow { 0 } else { 2 }
        );
    }
}
