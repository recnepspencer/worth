//! Winit events reach the native input fixture only from the runtime's own
//! tests; certification consumers drive input through the host instead.
use super::ScriptedPresentationHost;

impl ScriptedPresentationHost {
    pub fn observe_native_window_event(
        &self,
        event: &winit::event::WindowEvent,
        tick: u64,
        pointer: Option<winit::dpi::PhysicalPosition<f64>>,
    ) -> worth_ui_host_native::UiNativeLifecycleTransition {
        self.native_input
            .as_ref()
            .expect("native input fixture installed")
            .lock()
            .unwrap()
            .protocol
            .observe_window_event_at(event, tick, pointer)
    }
}
