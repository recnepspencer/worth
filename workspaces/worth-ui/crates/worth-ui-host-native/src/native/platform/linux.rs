//! Pointer position and motion posture for the Linux windowing systems.
//!
//! winit's `CursorMoved` already carries a client-relative, event-time
//! position, so this port caches the latest one and hands it to the button or
//! wheel event that follows it in the same ordered stream. There is no
//! screen-to-client conversion and no cursor query, which is what both Linux
//! profiles declare:
//! `native_pointer_position_observation = "winit-0.30.13-CursorMoved;event-ordered-client-origin;no-cursor-query"`.
//!
//! The cached position is retained across `CursorLeft`: a button release can
//! arrive after the cursor has left the client area, and a port that had
//! forgotten the position would report it unavailable, which the input
//! observer treats as terminal.

use std::sync::Arc;

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};
use winit::window::Window;

pub(crate) struct UiNativePointerInputPort {
    last_cursor: Option<PhysicalPosition<f64>>,
}

/// Always installs: the port needs nothing from the window because the
/// positions it caches are already client-relative.
pub(crate) fn install_pointer_input(_window: Arc<Window>) -> Option<Box<UiNativePointerInputPort>> {
    Some(Box::new(UiNativePointerInputPort { last_cursor: None }))
}

/// Both Linux profiles declare `reduced_motion_observation = "unobserved"`.
/// Reading the XDG settings portal is blocking D-Bus I/O behind a new
/// dependency and lands with its own qualification, not behind a pure-looking
/// accessor.
pub(crate) const fn observe_reduced_motion_posture() -> crate::native::UiNativeReducedMotionPosture
{
    crate::native::UiNativeReducedMotionPosture::Unavailable
}

impl UiNativePointerInputPort {
    /// Nothing to refresh: `CursorMoved` positions are client-relative, so a
    /// window move or scale change does not invalidate them.
    pub(crate) fn refresh_client_origin(&mut self) {}

    pub(crate) fn observe_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.last_cursor = Some(position);
    }

    /// The position of the most recent `CursorMoved`, which winit delivers
    /// before the button event it precedes. Not cleared: a second click
    /// without an intervening move happens at the same position.
    pub(crate) fn take_button_position(
        &mut self,
        _button: MouseButton,
        _state: ElementState,
    ) -> Option<PhysicalPosition<f64>> {
        self.last_cursor
    }

    pub(crate) fn take_scroll_position(&mut self) -> Option<PhysicalPosition<f64>> {
        self.last_cursor
    }
}

#[cfg(test)]
mod tests {
    use super::UiNativePointerInputPort;
    use winit::dpi::PhysicalPosition;
    use winit::event::{ElementState, MouseButton};

    #[test]
    fn button_position_is_the_last_observed_cursor_move_and_is_retained() {
        let mut port = UiNativePointerInputPort { last_cursor: None };
        assert_eq!(
            port.take_button_position(MouseButton::Left, ElementState::Pressed),
            None
        );
        port.observe_cursor_moved(PhysicalPosition::new(12.5, 40.0));
        assert_eq!(
            port.take_button_position(MouseButton::Left, ElementState::Pressed),
            Some(PhysicalPosition::new(12.5, 40.0))
        );
        assert_eq!(
            port.take_button_position(MouseButton::Left, ElementState::Released),
            Some(PhysicalPosition::new(12.5, 40.0))
        );
        assert_eq!(
            port.take_scroll_position(),
            Some(PhysicalPosition::new(12.5, 40.0))
        );
        port.refresh_client_origin();
        assert_eq!(
            port.take_scroll_position(),
            Some(PhysicalPosition::new(12.5, 40.0))
        );
    }
}
