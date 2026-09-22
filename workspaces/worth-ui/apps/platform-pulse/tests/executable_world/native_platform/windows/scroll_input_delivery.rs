use std::thread;
use std::time::{Duration, Instant};

use uiautomation::types::Handle;
use uiautomation::UIAutomation;
use winsafe::{co, HwKbMouse, HWND, MOUSEINPUT};

use crate::external_observation::{
    NativeClientPixelPoint, NativeInputProbeKind, ProcessBoundNativeClientAreaObservation,
};

use super::input_delivery::{
    input_failure, move_pointer_to, post_effect_failure, prime_pointer_motion,
};
use super::NativePlatformFailure;

/// One wheel notch in the platform's own `WHEEL_DELTA` units.
const WHEEL_DELTA: i32 = 120;
/// Intermediate pointer positions reported between button press and release,
/// so the product sees a real drag rather than a teleporting pointer.
const DRAG_STEP_COUNT: i32 = 4;
const DRAG_STEP_PAUSE: Duration = Duration::from_millis(16);

/// The screen point a process-bound client pixel occupies right now.
pub(super) fn screen_point_of(
    observed: ProcessBoundNativeClientAreaObservation,
    point: NativeClientPixelPoint,
) -> Result<(i32, i32), NativePlatformFailure> {
    let bounds = observed.bounds();
    if point.capture_extent() != (bounds.width(), bounds.height()) {
        return Err(NativePlatformFailure::InputDelivery(
            "pointer point was adjudicated from a different client capture extent".to_owned(),
        ));
    }
    let (client_x, client_y) = point.coordinates();
    Ok((
        bounds
            .left()
            .checked_add_unsigned(client_x)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?,
        bounds
            .top()
            .checked_add_unsigned(client_y)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?,
    ))
}

/// Turn the wheel `notches` times over `point`.
///
/// Returns the instant the events left for the product, so a caller timing the
/// product can start its clock there rather than at the harness preparation --
/// focus, pointer qualification and hit testing -- that has to happen first.
///
/// Positive `notches` roll the wheel toward the user, which the platform
/// spells as a negative `WHEEL_DELTA` and every application reads as "scroll
/// the content down". Each notch is one OS event, exactly as a real wheel
/// produces them, so the product's per-notch arithmetic is what gets tested.
pub(super) fn deliver_wheel_notches(
    window: &HWND,
    observed: ProcessBoundNativeClientAreaObservation,
    point: NativeClientPixelPoint,
    notches: i32,
) -> Result<Instant, NativePlatformFailure> {
    if notches == 0 {
        return Err(NativePlatformFailure::InputDelivery(
            "a wheel delivery needs at least one notch".to_owned(),
        ));
    }
    prepare_wheel_target(window, observed, point)?;
    // Rolling toward the user is a negative platform delta.
    let per_notch = notches.signum().wrapping_neg().wrapping_mul(WHEEL_DELTA);
    let events: Vec<HwKbMouse> = (0..notches.unsigned_abs())
        .map(|_| {
            HwKbMouse::Mouse(MOUSEINPUT {
                mouseData: per_notch as u32,
                dwFlags: co::MOUSEEVENTF::WHEEL,
                ..Default::default()
            })
        })
        .collect();
    let expected = notches.unsigned_abs();
    let issued = Instant::now();
    let delivered = winsafe::SendInput(&events)
        .map_err(|error| NativePlatformFailure::InputDelivery(error.to_string()))?;
    if delivered != expected {
        return Err(NativePlatformFailure::InputDelivery(format!(
            "SendInput delivered {delivered} of {expected} wheel notch events"
        )));
    }
    Ok(issued)
}

pub(super) fn prepare_wheel_target(
    window: &HWND,
    observed: ProcessBoundNativeClientAreaObservation,
    point: NativeClientPixelPoint,
) -> Result<(i32, i32), NativePlatformFailure> {
    let screen_point = screen_point_of(observed, point)?;
    focus_process_window(window)?;
    super::input_environment::qualify_pointer_world(window, observed.process_id(), screen_point)
        .map_err(NativePlatformFailure::InputEnvironment)?;
    move_pointer_to(screen_point)?;
    super::pointer_target::require_before_effect(window, screen_point)?;
    Ok(screen_point)
}

/// Press the primary button at `from`, move the pointer in steps to `to`, and
/// release it there.
///
/// The button stays down for the whole motion, so whatever the product
/// captured on press is what receives the motion and the release.
pub(super) fn deliver_pointer_drag(
    window: &HWND,
    observed: ProcessBoundNativeClientAreaObservation,
    from: NativeClientPixelPoint,
    to: NativeClientPixelPoint,
) -> Result<(), NativePlatformFailure> {
    let to_screen = screen_point_of(observed, to)?;
    deliver_captured_drag(window, observed, from, to_screen)?;
    super::pointer_target::require_after_effect(window, to_screen, NativeInputProbeKind::Pointer, 2)
}

/// A captured drag may end outside the client. Qualify the press, retain the
/// exact process/window, and always release the held button even on failure.
pub(super) fn deliver_captured_drag(
    window: &HWND,
    observed: ProcessBoundNativeClientAreaObservation,
    from: NativeClientPixelPoint,
    to_screen: (i32, i32),
) -> Result<(), NativePlatformFailure> {
    let from_screen = screen_point_of(observed, from)?;
    focus_process_window(window)?;
    super::input_environment::qualify_pointer_world(window, observed.process_id(), from_screen)
        .map_err(NativePlatformFailure::InputEnvironment)?;
    prime_pointer_motion(window, from_screen)?;
    super::pointer_visual_settlement::await_client_stability(observed)?;
    super::pointer_target::require_before_effect(window, from_screen)?;
    let pressed = winsafe::SendInput(&[HwKbMouse::Mouse(MOUSEINPUT {
        dwFlags: co::MOUSEEVENTF::LEFTDOWN,
        ..Default::default()
    })])
    .map_err(|error| NativePlatformFailure::InputDelivery(error.to_string()))?;
    if pressed != 1 {
        return Err(NativePlatformFailure::InputDelivery(format!(
            "SendInput delivered {pressed} of 1 drag press events"
        )));
    }
    let release_guard = PressedPrimary;
    for step in 1..=DRAG_STEP_COUNT {
        thread::sleep(DRAG_STEP_PAUSE);
        let waypoint = (
            interpolate(from_screen.0, to_screen.0, step),
            interpolate(from_screen.1, to_screen.1, step),
        );
        super::input_environment::actuate_and_observe_cursor(waypoint).map_err(|denial| {
            post_effect_failure(NativeInputProbeKind::Pointer, 1, format!("{denial:?}"))
        })?;
    }
    thread::sleep(DRAG_STEP_PAUSE);
    let released = winsafe::SendInput(&[HwKbMouse::Mouse(MOUSEINPUT {
        dwFlags: co::MOUSEEVENTF::LEFTUP,
        ..Default::default()
    })])
    .map_err(|error| post_effect_failure(NativeInputProbeKind::Pointer, 1, error.to_string()))?;
    if released != 1 {
        return Err(post_effect_failure(
            NativeInputProbeKind::Pointer,
            1,
            format!("SendInput delivered {released} of 1 drag release events"),
        ));
    }
    std::mem::forget(release_guard);
    Ok(())
}

struct PressedPrimary;

impl Drop for PressedPrimary {
    fn drop(&mut self) {
        let cleanup = winsafe::SendInput(&[HwKbMouse::Mouse(MOUSEINPUT {
            dwFlags: co::MOUSEEVENTF::LEFTUP,
            ..Default::default()
        })]);
        if !matches!(cleanup, Ok(1)) {
            eprintln!("failed to release pointer after interrupted native drag: {cleanup:?}");
        }
    }
}

fn interpolate(from: i32, to: i32, step: i32) -> i32 {
    let span = i64::from(to) - i64::from(from);
    let offset = span * i64::from(step) / i64::from(DRAG_STEP_COUNT);
    (i64::from(from) + offset) as i32
}

fn focus_process_window(window: &HWND) -> Result<(), NativePlatformFailure> {
    let automation = UIAutomation::new().map_err(input_failure)?;
    automation
        .element_from_handle(Handle::from(window.ptr() as isize))
        .and_then(|element| element.set_focus())
        .map_err(input_failure)
}
