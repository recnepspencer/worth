//! Put the cursor on a control and confirm it is still there when the button
//! goes down.
//!
//! The harness drives the one physical cursor this desktop has. Anything else
//! that moves it -- a hand on the mouse, a window grabbing the pointer -- lands
//! in the gap between the move and the check, and the product never sees the
//! click at all. That is the desktop refusing the delivery, not the product
//! failing it, so re-arm a bounded number of times and then name which it was.
use uiautomation::inputs::Mouse;
use winsafe::HWND;

use crate::external_observation::ProcessBoundNativeClientAreaObservation;

use super::input_delivery::{input_failure, prime_pointer_motion};
use super::input_environment::WindowsInputEnvironmentDenial;
use super::NativePlatformFailure;

/// Attempts allowed to put the cursor on the control and still find it there.
const ATTEMPTS: u32 = 4;

/// The screen point the cursor was actually holding when the button may go down.
pub(super) fn arm_pointer_at(
    window: &HWND,
    observed: ProcessBoundNativeClientAreaObservation,
    screen_point: (i32, i32),
    tolerance: Option<u32>,
) -> Result<(i32, i32), NativePlatformFailure> {
    let mut drifted_to = screen_point;
    for _ in 0..ATTEMPTS {
        prime_pointer_motion(window, screen_point)?;
        crate::native_platform::pointer_visual_settlement::await_client_stability(|| {
            Ok(
                super::gdi_capture::capture_client_area(observed.bounds(), observed.process_id())?
                    .rgba()
                    .to_vec(),
            )
        })?;
        super::pointer_target::require_before_effect(window, screen_point)?;
        let cursor = Mouse::get_cursor_pos().map_err(input_failure)?;
        let held = (cursor.get_x(), cursor.get_y());
        let drifted = tolerance.is_some_and(|tolerance| {
            held.0.abs_diff(screen_point.0) > tolerance
                || held.1.abs_diff(screen_point.1) > tolerance
        });
        if !drifted {
            return Ok(held);
        }
        drifted_to = held;
    }
    Err(NativePlatformFailure::InputEnvironment(
        WindowsInputEnvironmentDenial::CursorHeldAgainstHarness {
            requested: screen_point,
            observed: drifted_to,
            attempts: ATTEMPTS,
        },
    ))
}
