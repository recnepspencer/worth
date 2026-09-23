#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiNativeInputReachability {
    event_count: u64,
    pointer_button_events: u64,
    keyboard_events: u64,
    text_events: u64,
    ime_preedit_events: u64,
    ime_commit_events: u64,
    ime_cancel_events: u64,
}

#[must_use]
pub struct UiNativeObservationReadinessGrant {
    generation: u64,
    reachability: UiNativeInputReachability,
}

impl UiNativeInputReachability {
    /// Classifies a raw window event as reachability, before the semantic
    /// observation layer decides retention. `composition` is the IME
    /// composition owner's posture before this event: `Ime::Disabled` reaches
    /// as a cancel only while a composition is in progress; otherwise it is
    /// the platform's IME availability notice and reaches nothing.
    pub(in crate::native::event_loop) fn observe_window_event(
        event: &winit::event::WindowEvent,
        composition: crate::native::UiNativeImeCompositionPosture,
    ) -> Self {
        use winit::event::{Ime, WindowEvent};

        match event {
            WindowEvent::MouseInput { .. } => Self {
                event_count: 1,
                pointer_button_events: 1,
                ..Self::default()
            },
            WindowEvent::KeyboardInput { event, .. } => {
                Self::observed_keyboard(event.text.is_some())
            }
            WindowEvent::Ime(Ime::Preedit(_, _)) => Self {
                event_count: 1,
                ime_preedit_events: 1,
                ..Self::default()
            },
            WindowEvent::Ime(Ime::Commit(_)) => Self {
                event_count: 1,
                ime_commit_events: 1,
                ..Self::default()
            },
            WindowEvent::Ime(Ime::Disabled)
                if composition == crate::native::UiNativeImeCompositionPosture::Composing =>
            {
                Self {
                    event_count: 1,
                    ime_cancel_events: 1,
                    ..Self::default()
                }
            }
            _ => Self::default(),
        }
    }

    pub(in crate::native::event_loop) fn merge(&mut self, successor: Self) {
        self.event_count = self.event_count.saturating_add(successor.event_count);
        self.pointer_button_events = self
            .pointer_button_events
            .saturating_add(successor.pointer_button_events);
        self.keyboard_events = self
            .keyboard_events
            .saturating_add(successor.keyboard_events);
        self.text_events = self.text_events.saturating_add(successor.text_events);
        self.ime_preedit_events = self
            .ime_preedit_events
            .saturating_add(successor.ime_preedit_events);
        self.ime_commit_events = self
            .ime_commit_events
            .saturating_add(successor.ime_commit_events);
        self.ime_cancel_events = self
            .ime_cancel_events
            .saturating_add(successor.ime_cancel_events);
    }

    fn observed_keyboard(has_text: bool) -> Self {
        Self {
            event_count: 1_u64.saturating_add(u64::from(has_text)),
            keyboard_events: 1,
            text_events: u64::from(has_text),
            ..Self::default()
        }
    }

    pub const fn is_empty(self) -> bool {
        self.event_count == 0
    }

    pub const fn event_count(self) -> u64 {
        self.event_count
    }

    pub const fn pointer_button_events(self) -> u64 {
        self.pointer_button_events
    }

    pub const fn keyboard_events(self) -> u64 {
        self.keyboard_events
    }

    pub const fn text_events(self) -> u64 {
        self.text_events
    }

    pub const fn ime_preedit_events(self) -> u64 {
        self.ime_preedit_events
    }

    pub const fn ime_commit_events(self) -> u64 {
        self.ime_commit_events
    }

    pub const fn ime_cancel_events(self) -> u64 {
        self.ime_cancel_events
    }
}

impl UiNativeObservationReadinessGrant {
    pub(in crate::native::event_loop) const fn issued(
        generation: u64,
        reachability: UiNativeInputReachability,
    ) -> Self {
        Self {
            generation,
            reachability,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn reachability(&self) -> UiNativeInputReachability {
        self.reachability
    }
}

#[cfg(test)]
mod tests {
    use super::UiNativeInputReachability;
    use crate::native::UiNativeImeCompositionPosture;
    use winit::event::{Ime, WindowEvent};

    #[test]
    fn ime_disabled_reaches_as_a_cancel_only_while_a_composition_is_in_progress() {
        let event = WindowEvent::Ime(Ime::Disabled);

        let idle = UiNativeInputReachability::observe_window_event(
            &event,
            UiNativeImeCompositionPosture::Idle,
        );
        assert!(idle.is_empty(), "the availability notice is not input");
        assert_eq!(idle.ime_cancel_events(), 0);

        let composing = UiNativeInputReachability::observe_window_event(
            &event,
            UiNativeImeCompositionPosture::Composing,
        );
        assert_eq!(composing.event_count(), 1);
        assert_eq!(composing.ime_cancel_events(), 1);
    }

    #[test]
    fn ime_enabled_never_reaches() {
        for composition in [
            UiNativeImeCompositionPosture::Idle,
            UiNativeImeCompositionPosture::Composing,
        ] {
            let reached = UiNativeInputReachability::observe_window_event(
                &WindowEvent::Ime(Ime::Enabled),
                composition,
            );
            assert!(reached.is_empty());
        }
    }

    #[test]
    fn raw_keyboard_reachability_does_not_require_semantic_recipient_authority() {
        let reached = UiNativeInputReachability::observed_keyboard(true);

        assert_eq!(reached.event_count(), 2);
        assert_eq!(reached.keyboard_events(), 1);
        assert_eq!(reached.text_events(), 1);
    }
}
