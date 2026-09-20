use super::{presented_state, HOST_SESSION};
use crate::native::UiNativePointerPositionWitness;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, MouseScrollDelta, TouchPhase, WindowEvent};
use worth_ui_host_contract::{
    UiHostObservationPayload, UiHostScrollDeltaPhase, UiHostScrollDeltaPrecision,
    UiHostScrollDeltaSource, UiHostScrollLineCountBasis, UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
};

#[test]
fn event_time_pointer_witness_targets_the_exact_presented_coordinate() {
    let mut state = presented_state();
    state.observe_window_event_at_with_pointer_witness(
        &WindowEvent::MouseWheel {
            device_id: DeviceId::dummy(),
            delta: MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, 3.25)),
            phase: TouchPhase::Moved,
        },
        7,
        UiNativePointerPositionWitness::EventTime(PhysicalPosition::new(12.5, 24.25)),
    );
    let batches = state.drain(HOST_SESSION).into_batches();
    let UiHostObservationPayload::ScrollDelta { target, .. } = batches[0].reports()[0].payload()
    else {
        panic!("wheel event must remain a scroll payload");
    };
    let position = target.position().expect("event-time coordinate target");
    assert_eq!(
        (position.x_subpixels(), position.y_subpixels()),
        (12_500, 24_250)
    );
    assert!(!target.is_surface_fallback());
}

#[test]
fn wheel_delta_follows_the_count_the_observation_states_and_does_not_suppress_later_input() {
    let mut state = presented_state();
    state.observe_window_event(&WindowEvent::MouseWheel {
        device_id: DeviceId::dummy(),
        delta: MouseScrollDelta::LineDelta(1.0, -2.0),
        phase: TouchPhase::Moved,
    });
    state.observe_window_event(&WindowEvent::Focused(true));
    let batches = state.drain(HOST_SESSION).into_batches();
    assert_eq!(batches.len(), 2);
    let UiHostObservationPayload::ScrollDelta {
        source,
        phase,
        precision,
        target,
        x_subpixels,
        y_subpixels,
    } = batches[0].reports()[0].payload()
    else {
        panic!("first batch must retain the qualified wheel event");
    };
    assert_eq!(*source, UiHostScrollDeltaSource::PointerWheel);
    assert_eq!(*phase, UiHostScrollDeltaPhase::Updated);
    // The reader's own platform setting decides the count here, so this test
    // pins the coherence between what the observation states and what it
    // carries, never a particular machine's answer. The mapping from each
    // platform answer to a stated count lives in the wheel notch and event
    // profile unit tests, which need no platform at all.
    let lines_per_notch = match *precision {
        UiHostScrollDeltaPrecision::Line {
            platform_lines_per_notch,
            basis,
        } => {
            match basis {
                UiHostScrollLineCountBasis::PlatformReported => assert!(
                    platform_lines_per_notch >= 1,
                    "a count the platform reported is a count of at least one line"
                ),
                UiHostScrollLineCountBasis::DefaultedAfterMissing
                | UiHostScrollLineCountBasis::DefaultedAfterInvalid => assert_eq!(
                    platform_lines_per_notch, UI_HOST_SCROLL_DEFAULT_LINES_PER_NOTCH,
                    "a defaulted count is the one default this contract names"
                ),
            }
            i64::from(platform_lines_per_notch)
        }
        UiHostScrollDeltaPrecision::Page => 1,
        UiHostScrollDeltaPrecision::Pixel => {
            panic!("a notch-denominated wheel event never carries pixel precision")
        }
    };
    assert!(target.is_surface_fallback());
    assert_eq!(
        target.presentation(),
        batches[0].canonical_core().presentation()
    );
    // One notch right and two notches up, each notch worth exactly the unit
    // count the observation itself states, carried at 1000 per unit.
    assert_eq!(
        (*x_subpixels, *y_subpixels),
        (1_000 * lines_per_notch, -2_000 * lines_per_notch)
    );
    assert!(matches!(
        batches[1].reports()[0].payload(),
        UiHostObservationPayload::WindowFocus { focused: true, .. }
    ));
    assert_eq!(batches[0].reports()[0].sequence().value(), 1);
    assert_eq!(batches[1].reports()[0].sequence().value(), 2);
    let report = state.report();
    assert_eq!(report.terminal_stop(), None);
    assert_eq!(
        report.last_vertical_scroll().map(|scroll| (
            scroll.sequence(),
            scroll.x_subpixels(),
            scroll.y_subpixels()
        )),
        Some((1, 1_000 * lines_per_notch, -2_000 * lines_per_notch))
    );
    assert_eq!(
        report.last_horizontal_scroll().map(|scroll| (
            scroll.sequence(),
            scroll.x_subpixels(),
            scroll.y_subpixels()
        )),
        Some((1, 1_000 * lines_per_notch, -2_000 * lines_per_notch))
    );
}

#[test]
fn pixel_precision_and_every_native_scroll_phase_survive_adapter_drain() {
    for (native, expected) in [
        (TouchPhase::Started, UiHostScrollDeltaPhase::Started),
        (TouchPhase::Moved, UiHostScrollDeltaPhase::Updated),
        (TouchPhase::Ended, UiHostScrollDeltaPhase::Ended),
        (TouchPhase::Cancelled, UiHostScrollDeltaPhase::Cancelled),
    ] {
        let mut state = presented_state();
        state.observe_window_event(&WindowEvent::MouseWheel {
            device_id: DeviceId::dummy(),
            delta: MouseScrollDelta::PixelDelta(PhysicalPosition::new(1.5, -2.5)),
            phase: native,
        });
        let batches = state.drain(HOST_SESSION).into_batches();
        let UiHostObservationPayload::ScrollDelta {
            phase,
            precision,
            target,
            x_subpixels,
            y_subpixels,
            ..
        } = batches[0].reports()[0].payload()
        else {
            panic!("pixel wheel event must remain a scroll payload");
        };
        assert_eq!(*phase, expected);
        assert_eq!(*precision, UiHostScrollDeltaPrecision::Pixel);
        assert_eq!((*x_subpixels, *y_subpixels), (1_500, -2_500));
        assert_eq!(
            target.presentation(),
            batches[0].canonical_core().presentation()
        );
    }
}
