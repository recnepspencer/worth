//! The fallback for targets no qualified profile covers yet.
//!
//! No pointer port is installed and reduced motion is unobserved. This is the
//! fallback `platform/mod.rs` carried inline before the Linux port existed; it
//! is kept as a sibling so the facade only selects. It is not a qualified
//! lane: a target that reaches this module reports its posture through the
//! profile record, never as certified.

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};

pub(crate) struct UiNativePointerInputPort;

pub(crate) const fn observe_reduced_motion_posture() -> crate::native::UiNativeReducedMotionPosture
{
    crate::native::UiNativeReducedMotionPosture::Unavailable
}

impl UiNativePointerInputPort {
    pub(crate) fn refresh_client_origin(&mut self) {}

    pub(crate) fn observe_cursor_moved(&mut self, _position: PhysicalPosition<f64>) {}

    pub(crate) fn take_button_position(
        &mut self,
        _button: MouseButton,
        _state: ElementState,
    ) -> Option<PhysicalPosition<f64>> {
        None
    }

    pub(crate) fn take_scroll_position(&mut self) -> Option<PhysicalPosition<f64>> {
        None
    }
}
