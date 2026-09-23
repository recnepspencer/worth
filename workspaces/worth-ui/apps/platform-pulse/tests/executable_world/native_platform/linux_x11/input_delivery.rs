//! Synthesised input through XTEST, with the same effect boundary as the
//! Windows observer: every qualification happens before the first fake
//! event, every failure after it is `InputDeliveryIndeterminate` carrying
//! how many events the server had already accepted.
use std::thread;
use std::time::Duration;

use x11rb::protocol::xproto::Window;
use x11rb::protocol::xtest::ConnectionExt as _;

use crate::external_observation::{
    NativeClientPixelPoint, NativeInputDeliveryObservation, NativeInputProbeKind,
    NativeKeyboardCommand, ProcessBoundNativeClientAreaObservation,
};
use crate::native_platform::{pointer_visual_settlement, NativePlatformFailure};

use super::connection::X11Observation;
use super::input_environment::{
    self, LinuxX11InputEnvironmentDenial, BUTTON_PRESS, BUTTON_RELEASE, KEY_PRESS, KEY_RELEASE,
};
use super::keyboard_map::{KeyboardMap, NativeKeyboardInput, XK_A};
use super::{client_capture, pointer_target};

const LEFT_BUTTON: u8 = 1;
/// X core buttons 4/5 scroll vertically and 6/7 horizontally; winit maps a
/// press to one `MouseWheel` line delta. One press+release per axis.
const WHEEL_UP: u8 = 4;
const WHEEL_RIGHT: u8 = 7;
const CHORD_STEP: Duration = Duration::from_millis(10);

pub(super) fn deliver(
    x11: &X11Observation,
    window: Window,
    observed: ProcessBoundNativeClientAreaObservation,
    kind: NativeInputProbeKind,
) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
    deliver_at(
        x11,
        window,
        observed,
        kind,
        client_center(observed)?,
        None,
        NativeKeyboardInput::Single(XK_A),
    )
}

pub(super) fn deliver_keyboard_command(
    x11: &X11Observation,
    window: Window,
    observed: ProcessBoundNativeClientAreaObservation,
    command: NativeKeyboardCommand,
) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
    deliver_at(
        x11,
        window,
        observed,
        NativeInputProbeKind::Keyboard,
        client_center(observed)?,
        None,
        NativeKeyboardInput::for_command(command),
    )
}

pub(super) fn deliver_pointer(
    x11: &X11Observation,
    window: Window,
    observed: ProcessBoundNativeClientAreaObservation,
    point: NativeClientPixelPoint,
) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
    let bounds = observed.bounds();
    if point.capture_extent() != (bounds.width(), bounds.height()) {
        return Err(NativePlatformFailure::InputDelivery(
            "pointer point was adjudicated from a different client capture extent".to_owned(),
        ));
    }
    let (client_x, client_y) = point.coordinates();
    let screen_point = (
        bounds
            .left()
            .checked_add_unsigned(client_x)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?,
        bounds
            .top()
            .checked_add_unsigned(client_y)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?,
    );
    deliver_at(
        x11,
        window,
        observed,
        NativeInputProbeKind::Pointer,
        screen_point,
        Some(point.landing_tolerance()),
        NativeKeyboardInput::Single(XK_A),
    )
}

pub(super) fn deliver_wheel_deltas(
    x11: &X11Observation,
    window: Window,
    observed: ProcessBoundNativeClientAreaObservation,
) -> Result<(), NativePlatformFailure> {
    let screen_point = client_center(observed)?;
    input_environment::qualify_keyboard_focus(x11, window)
        .map_err(NativePlatformFailure::InputEnvironment)?;
    input_environment::actuate_and_observe_cursor(x11, screen_point)
        .map_err(NativePlatformFailure::InputEnvironment)?;
    pointer_target::require_before_effect(x11, window)?;
    let mut delivered = 0;
    for button in [WHEEL_UP, WHEEL_RIGHT] {
        delivered = fake_button(x11, NativeInputProbeKind::Pointer, button, delivered)?;
    }
    if delivered != 4 {
        return Err(NativePlatformFailure::InputDelivery(format!(
            "XTEST accepted {delivered} of 4 wheel events"
        )));
    }
    Ok(())
}

fn deliver_at(
    x11: &X11Observation,
    window: Window,
    observed: ProcessBoundNativeClientAreaObservation,
    kind: NativeInputProbeKind,
    screen_point: (i32, i32),
    pointer_tolerance: Option<u32>,
    keyboard_input: NativeKeyboardInput,
) -> Result<NativeInputDeliveryObservation, NativePlatformFailure> {
    let bounds = observed.bounds();
    input_environment::qualify_keyboard_focus(x11, window)
        .map_err(NativePlatformFailure::InputEnvironment)?;
    let (delivered_event_count, qualified_point) = match kind {
        NativeInputProbeKind::Pointer => {
            prime_pointer_motion(x11, window, screen_point)?;
            pointer_visual_settlement::await_client_stability(|| {
                client_capture::window_rgba(x11, window, bounds)
            })?;
            pointer_target::require_before_effect(x11, window)?;
            let qualified_point = input_environment::observe_cursor(x11)
                .map_err(NativePlatformFailure::InputDelivery)?;
            if pointer_tolerance.is_some_and(|tolerance| {
                qualified_point.0.abs_diff(screen_point.0) > tolerance
                    || qualified_point.1.abs_diff(screen_point.1) > tolerance
            }) {
                return Err(NativePlatformFailure::InputDelivery(format!(
                    "cursor moved away from the qualified control before button delivery: expected={screen_point:?}; observed={qualified_point:?}; tolerance={pointer_tolerance:?}"
                )));
            }
            let delivered = fake_button(x11, kind, LEFT_BUTTON, 0)?;
            require_complete_delivery(kind, delivered, 2, "pointer")?;
            (delivered, qualified_point)
        }
        NativeInputProbeKind::Keyboard => {
            let focus = input_environment::focused_window(x11)
                .map_err(LinuxX11InputEnvironmentDenial::DisplayUnavailable)
                .map_err(NativePlatformFailure::InputEnvironment)?;
            if focus != window {
                return Err(NativePlatformFailure::InputEnvironment(
                    LinuxX11InputEnvironmentDenial::KeyboardFocusUnavailable {
                        target_window: window,
                    },
                ));
            }
            let sequence = keyboard_input.keycode_sequence(&KeyboardMap::read(x11)?)?;
            let delivered = deliver_keyboard_events(x11, kind, &sequence)?;
            require_complete_delivery(
                kind,
                delivered,
                keyboard_input.expected_event_count(),
                "keyboard",
            )?;
            let retained = input_environment::focused_window(x11).map_err(|error| {
                post_effect_failure(
                    kind,
                    delivered,
                    format!("observe keyboard focus after delivery: {error}"),
                )
            })?;
            if retained != window {
                return Err(post_effect_failure(
                    kind,
                    delivered,
                    format!("the bound window lost keyboard focus to {retained:#x}"),
                ));
            }
            (delivered, screen_point)
        }
    };
    if kind == NativeInputProbeKind::Pointer {
        pointer_target::require_after_effect(x11, window, kind, delivered_event_count)?;
    }
    if qualified_point.0 < bounds.left()
        || qualified_point.0 >= bounds.right()
        || qualified_point.1 < bounds.top()
        || qualified_point.1 >= bounds.bottom()
    {
        return Err(post_effect_failure(
            kind,
            delivered_event_count,
            "native input target was outside the process-bound client area",
        ));
    }
    Ok(NativeInputDeliveryObservation::for_client(
        kind,
        observed,
        qualified_point,
        delivered_event_count,
    ))
}

fn client_center(
    observed: ProcessBoundNativeClientAreaObservation,
) -> Result<(i32, i32), NativePlatformFailure> {
    let bounds = observed.bounds();
    Ok((
        bounds
            .left()
            .checked_add_unsigned(bounds.width() / 2)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?,
        bounds
            .top()
            .checked_add_unsigned(bounds.height() / 2)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?,
    ))
}

/// Move next to the point and then onto it, so the product sees motion into
/// the control rather than a cursor that was already there.
fn prime_pointer_motion(
    x11: &X11Observation,
    window: Window,
    screen_point: (i32, i32),
) -> Result<(), NativePlatformFailure> {
    let adjacent = (
        screen_point
            .0
            .checked_sub(1)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?,
        screen_point.1,
    );
    input_environment::actuate_and_observe_cursor(x11, adjacent)
        .map_err(NativePlatformFailure::InputEnvironment)?;
    pointer_target::require_before_effect(x11, window)?;
    input_environment::actuate_and_observe_cursor(x11, screen_point)
        .map_err(NativePlatformFailure::InputEnvironment)
}

/// Press and release one button at the cursor's current position. Returns
/// the running count of events the server accepted.
fn fake_button(
    x11: &X11Observation,
    kind: NativeInputProbeKind,
    button: u8,
    delivered: u32,
) -> Result<u32, NativePlatformFailure> {
    let delivered = fake_input(x11, kind, BUTTON_PRESS, button, delivered)?;
    fake_input(x11, kind, BUTTON_RELEASE, button, delivered)
}

fn deliver_keyboard_events(
    x11: &X11Observation,
    kind: NativeInputProbeKind,
    sequence: &[(u8, bool)],
) -> Result<u32, NativePlatformFailure> {
    let mut delivered = 0;
    for (index, (keycode, pressed)) in sequence.iter().enumerate() {
        if index > 0 {
            thread::sleep(CHORD_STEP);
        }
        let type_ = if *pressed { KEY_PRESS } else { KEY_RELEASE };
        delivered = fake_input(x11, kind, type_, *keycode, delivered)?;
    }
    Ok(delivered)
}

/// One checked XTEST request. The server's acceptance of the request is the
/// delivery count; a refusal after earlier acceptances is indeterminate,
/// not a clean denial, because those earlier events already happened.
fn fake_input(
    x11: &X11Observation,
    kind: NativeInputProbeKind,
    type_: u8,
    detail: u8,
    delivered: u32,
) -> Result<u32, NativePlatformFailure> {
    x11.connection()
        .xtest_fake_input(type_, detail, x11rb::CURRENT_TIME, x11rb::NONE, 0, 0, 0)
        .map_err(|error| post_effect_failure(kind, delivered, error.to_string()))?
        .check()
        .map_err(|error| post_effect_failure(kind, delivered, error.to_string()))?;
    Ok(delivered + 1)
}

fn require_complete_delivery(
    kind: NativeInputProbeKind,
    delivered_event_count: u32,
    expected_event_count: u32,
    family: &'static str,
) -> Result<(), NativePlatformFailure> {
    if delivered_event_count != expected_event_count {
        return Err(post_effect_failure(
            kind,
            delivered_event_count,
            format!(
                "XTEST accepted {delivered_event_count} of {expected_event_count} {family} events"
            ),
        ));
    }
    Ok(())
}

pub(super) fn post_effect_failure(
    kind: NativeInputProbeKind,
    delivered_event_count: u32,
    detail: impl Into<String>,
) -> NativePlatformFailure {
    NativePlatformFailure::InputDeliveryIndeterminate {
        kind,
        delivered_event_count,
        detail: detail.into(),
    }
}
