impl super::UiFocusRuntimeState {
    pub(in crate::runtime) fn observe_window_focus(&mut self, focused: bool) {
        let next = crate::runtime::focus::UiWindowFocus::from_host_observation(focused);
        if self.window_focus != next {
            self.window_focus = next;
            self.bump_appearance_revision();
        }
    }

    pub(in crate::runtime) fn observe_keyboard_modality(&mut self) {
        let next = crate::runtime::focus::UiFocusVisibleModality::Keyboard;
        if self.modality != next {
            self.modality = next;
            self.bump_appearance_revision();
        }
    }

    pub(in crate::runtime) fn observe_pointer_modality(&mut self) {
        let next = crate::runtime::focus::UiFocusVisibleModality::Pointer;
        if self.modality != next {
            self.modality = next;
            self.bump_appearance_revision();
        }
    }

    pub(crate) fn observe_host_payload(
        &mut self,
        payload: &worth_ui_host_contract::UiHostObservationPayload,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) {
        match payload {
            worth_ui_host_contract::UiHostObservationPayload::WindowFocus { surface, focused }
                if *surface == presentation.host_surface() =>
            {
                self.observe_window_focus(*focused);
            }
            worth_ui_host_contract::UiHostObservationPayload::Keyboard {
                transition: worth_ui_host_contract::UiHostKeyTransition::Pressed { repeat: false },
                ..
            } => self.observe_keyboard_modality(),
            worth_ui_host_contract::UiHostObservationPayload::PointerButton {
                transition: worth_ui_host_contract::UiHostPointerButtonTransition::Pressed,
                ..
            } => self.observe_pointer_modality(),
            _ => {}
        }
    }

    #[cfg(test)]
    pub(crate) const fn window_is_focused_for_test(&self) -> bool {
        self.window_focus.is_focused()
    }

    #[cfg(test)]
    pub(crate) fn participant_count_for_test(&self) -> usize {
        self.participant_index.len()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn inspect_for_certification(
        &self,
    ) -> (Option<u64>, Option<u64>, usize, usize, u64) {
        (
            self.current
                .map(|focus| focus.participant().mounted_instance().diagnostic_value()),
            self.active_descendant
                .map(|active| active.descendant().mounted_instance().diagnostic_value()),
            self.participant_index.len(),
            self.pending_portal.len(),
            self.revision,
        )
    }

    pub(crate) const fn current_semantic_focus(
        &self,
    ) -> Option<crate::runtime::focus::UiSemanticKeyboardFocus> {
        self.current
    }

    pub(crate) fn inspect(&self) -> crate::runtime::focus::UiFocusInspectionSnapshot {
        #[cfg(not(test))]
        return crate::runtime::focus::UiFocusInspectionSnapshot::new(self.current, self.revision);

        #[cfg(test)]
        return crate::runtime::focus::UiFocusInspectionSnapshot::for_test(
            self.current,
            self.active_descendant,
            crate::runtime::focus::UiAccessibilityFocusHook.support(),
            self.current.is_some() && self.window_focus.is_focused() && self.modality.is_keyboard(),
            self.revision,
        );
    }
}
