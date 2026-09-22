pub(super) type UiNativePointerInputPort = crate::native::UiNativePointerInputPort;

#[cfg(any(target_os = "windows", target_os = "linux"))]
pub(super) fn install_pointer_input(
    window: &super::UiNativeOwnedWindow,
) -> Option<Box<UiNativePointerInputPort>> {
    crate::native::platform::install_pointer_input(std::sync::Arc::clone(window))
}

pub(super) fn event_pointer_witness(
    input: &mut Option<Box<UiNativePointerInputPort>>,
    event: &winit::event::WindowEvent,
) -> crate::native::UiNativePointerPositionWitness {
    use crate::native::UiNativePointerPositionWitness as Witness;
    match event {
        winit::event::WindowEvent::Moved(_) => {
            if let Some(input) = input.as_mut() {
                input.refresh_client_origin();
            }
            Witness::Unavailable
        }
        winit::event::WindowEvent::MouseInput { state, button, .. } => input
            .as_mut()
            .and_then(|input| input.take_button_position(*button, *state))
            .map(Witness::EventTime)
            .unwrap_or(Witness::Unavailable),
        winit::event::WindowEvent::MouseWheel { .. } => input
            .as_mut()
            .and_then(|input| input.take_scroll_position())
            .map(Witness::EventTime)
            .unwrap_or(Witness::Unavailable),
        winit::event::WindowEvent::CursorMoved { position, .. } => {
            if let Some(input) = input.as_mut() {
                input.observe_cursor_moved(*position);
            }
            Witness::Unavailable
        }
        _ => Witness::Unavailable,
    }
}
