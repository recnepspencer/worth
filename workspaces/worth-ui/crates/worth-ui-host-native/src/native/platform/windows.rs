use std::sync::Arc;

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};
use winit::window::Window;

/// The Windows message position is sampled while winit is dispatching the
/// corresponding button message. It is deliberately not a cursor query: the
/// cursor may have moved again by the time the runtime drains the retained
/// observation.
pub(crate) struct UiNativePointerInputPort {
    window: Arc<Window>,
    client_origin: Option<PhysicalPosition<i32>>,
}

pub(crate) fn install_pointer_input(window: Arc<Window>) -> Option<Box<UiNativePointerInputPort>> {
    let client_origin = window.inner_position().ok();
    Some(Box::new(UiNativePointerInputPort {
        window,
        client_origin,
    }))
}

pub(crate) fn observe_reduced_motion_posture() -> crate::native::UiNativeReducedMotionPosture {
    let observed = windows::UI::ViewManagement::UISettings::new()
        .and_then(|settings| settings.AnimationsEnabled());
    match observed {
        Ok(false) => crate::native::UiNativeReducedMotionPosture::Reduce,
        Ok(true) => crate::native::UiNativeReducedMotionPosture::NoPreference,
        Err(_) => crate::native::UiNativeReducedMotionPosture::Unavailable,
    }
}

impl UiNativePointerInputPort {
    pub(crate) fn refresh_client_origin(&mut self) {
        self.client_origin = self.window.inner_position().ok();
    }

    /// Windows samples the message position when the button message itself
    /// is dispatched, so the cursor stream carries nothing this port needs.
    pub(crate) fn observe_cursor_moved(&mut self, _position: PhysicalPosition<f64>) {}
}

impl UiNativePointerInputPort {
    pub(crate) fn take_button_position(
        &mut self,
        _button: MouseButton,
        _state: ElementState,
    ) -> Option<PhysicalPosition<f64>> {
        let message_position = winsafe::GetMessagePos();
        let client_origin = self.client_origin?;
        Some(decode_client_position(message_position, client_origin))
    }

    pub(crate) fn take_scroll_position(&mut self) -> Option<PhysicalPosition<f64>> {
        let message_position = winsafe::GetMessagePos();
        let client_origin = self.client_origin?;
        Some(decode_client_position(message_position, client_origin))
    }
}

fn decode_client_position(
    message_position: winsafe::POINT,
    client_origin: PhysicalPosition<i32>,
) -> PhysicalPosition<f64> {
    PhysicalPosition::new(
        f64::from(wrapped_axis(message_position.x, client_origin.x)),
        f64::from(wrapped_axis(message_position.y, client_origin.y)),
    )
}

fn wrapped_axis(message_coordinate: i32, client_origin: i32) -> i16 {
    let message_word = message_coordinate as u16;
    let origin_word = client_origin as u16;
    let client_word = message_word.wrapping_sub(origin_word);
    i16::from_ne_bytes(client_word.to_ne_bytes())
}

#[cfg(test)]
mod tests {
    use super::wrapped_axis;

    #[test]
    fn reconstructs_signed_client_coordinates_with_wrapping_virtual_desktop_words() {
        assert_eq!(wrapped_axis((-7_i16) as i32, (-20_i16) as i32), 13);
        assert_eq!(wrapped_axis(13, (-7_i16) as i32), 20);
        assert_eq!(wrapped_axis((-32_759_i16) as i32, 32_760), 17);
    }
}
