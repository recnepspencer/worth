#[cfg(any(test, feature = "certification-support"))]
use winit::event::ElementState;
use winit::event::WindowEvent;
#[cfg(any(test, feature = "certification-support"))]
use winit::keyboard::{Key, PhysicalKey};

use super::{
    event_focus, event_ime, event_keyboard, event_pointer, event_scroll,
    UiNativeInputObservationDisposition, UiNativeInputObservationState,
    UiNativePointerPositionWitness,
};

impl UiNativeInputObservationState {
    #[cfg(test)]
    pub(crate) fn observe_window_event(
        &mut self,
        event: &WindowEvent,
    ) -> UiNativeInputObservationDisposition {
        self.observe_window_event_at(event, self.event_tick)
    }

    #[cfg(test)]
    pub(crate) fn observe_window_event_at(
        &mut self,
        event: &WindowEvent,
        event_tick: u64,
    ) -> UiNativeInputObservationDisposition {
        self.observe_window_event_at_with_pointer_witness(
            event,
            event_tick,
            UiNativePointerPositionWitness::Unavailable,
        )
    }

    pub(crate) fn observe_window_event_at_with_pointer_witness(
        &mut self,
        event: &WindowEvent,
        event_tick: u64,
        pointer_witness: UiNativePointerPositionWitness,
    ) -> UiNativeInputObservationDisposition {
        self.set_event_tick(event_tick);
        if self.terminal_stop.is_some() {
            // These are current platform postures, not queued user actions.
            // Keep them accurate while the discontinuous action stream stops.
            match event {
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.modifiers = super::keyboard::modifiers(*modifiers)
                }
                WindowEvent::Ime(winit::event::Ime::Enabled) => self.ime_enabled = true,
                WindowEvent::Ime(winit::event::Ime::Disabled) => self.ime_enabled = false,
                _ => {}
            }
            return UiNativeInputObservationDisposition::Stopped;
        }
        event_focus::observe(self, event)
            .or_else(|| event_pointer::observe(self, event, pointer_witness))
            .or_else(|| event_keyboard::observe(self, event))
            .or_else(|| event_scroll::observe(self, event, pointer_witness))
            .or_else(|| event_ime::observe(self, event))
            .unwrap_or(UiNativeInputObservationDisposition::Ignored)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn observe_keyboard_components_at(
        &mut self,
        logical_key: &Key,
        physical_key: PhysicalKey,
        key_state: ElementState,
        repeat: bool,
        text: Option<&str>,
        event_tick: u64,
    ) -> UiNativeInputObservationDisposition {
        self.set_event_tick(event_tick);
        if self.terminal_stop.is_some() {
            return UiNativeInputObservationDisposition::Stopped;
        }
        event_keyboard::observe_components(self, logical_key, physical_key, key_state, repeat, text)
    }
}
