#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiFocusAppearanceClass {
    Unfocused,
    Focused,
    FocusVisible,
    FocusedWindowInactive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiFocusAppearancePosture {
    class: UiFocusAppearanceClass,
    target: Option<UiFocusAppearanceTarget>,
    window: super::UiWindowFocus,
    modality: super::UiFocusVisibleModality,
    owner_revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiFocusAppearanceTarget {
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
}

impl super::UiFocusRuntimeState {
    pub(crate) fn appearance_posture(&self) -> UiFocusAppearancePosture {
        let target = self.current.map(|focus| UiFocusAppearanceTarget {
            graph_node: focus.graph_node(),
            mounted_instance: focus.mounted_instance(),
            incarnation: focus.incarnation(),
        });
        let class = match (target, self.window_focus, self.modality) {
            (None, _, _) => UiFocusAppearanceClass::Unfocused,
            (Some(_), super::UiWindowFocus::Unfocused, _) => {
                UiFocusAppearanceClass::FocusedWindowInactive
            }
            (Some(_), super::UiWindowFocus::Focused, super::UiFocusVisibleModality::Keyboard) => {
                UiFocusAppearanceClass::FocusVisible
            }
            (Some(_), super::UiWindowFocus::Focused, _) => UiFocusAppearanceClass::Focused,
        };
        UiFocusAppearancePosture {
            class,
            target,
            window: self.window_focus,
            modality: self.modality,
            owner_revision: self.appearance_revision,
        }
    }
}

impl UiFocusAppearancePosture {
    pub(crate) fn appearance_dependency_eq(self, other: Self) -> bool {
        self.target == other.target
            && self.class == other.class
            && self.window == other.window
            && self.modality == other.modality
    }

    pub(crate) const fn class(self) -> UiFocusAppearanceClass {
        self.class
    }
    pub(crate) const fn owner_revision(self) -> u64 {
        self.owner_revision
    }
    pub(crate) const fn target(self) -> Option<UiFocusAppearanceTarget> {
        self.target
    }
    pub(in crate::runtime) const fn window(self) -> super::UiWindowFocus {
        self.window
    }
    pub(in crate::runtime) const fn modality(self) -> super::UiFocusVisibleModality {
        self.modality
    }
}

impl UiFocusAppearanceTarget {
    pub(crate) const fn graph_node(self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }
    pub(crate) const fn mounted_instance(
        self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }
    pub(crate) const fn incarnation(self) -> worth_ui_host_contract::UiMountIncarnation {
        self.incarnation
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn missing_target_is_unfocused_even_under_initial_modality() {
        let state = super::super::UiFocusRuntimeState::new_session_restore_candidate();
        assert_eq!(
            state.appearance_posture().class(),
            super::UiFocusAppearanceClass::Unfocused
        );
        assert_eq!(state.appearance_posture().owner_revision(), 0);
    }

    #[test]
    fn appearance_revision_advances_only_when_observed_inputs_change() {
        let mut state = super::super::UiFocusRuntimeState::new_session_restore_candidate();

        state.observe_window_focus(false);
        assert_eq!(state.appearance_posture().owner_revision(), 0);
        state.observe_window_focus(true);
        assert_eq!(state.appearance_posture().owner_revision(), 1);
        state.observe_window_focus(true);
        assert_eq!(state.appearance_posture().owner_revision(), 1);
        state.observe_keyboard_modality();
        assert_eq!(state.appearance_posture().owner_revision(), 2);
        state.observe_keyboard_modality();
        assert_eq!(state.appearance_posture().owner_revision(), 2);
        state.observe_pointer_modality();
        assert_eq!(state.appearance_posture().owner_revision(), 3);
    }

    #[test]
    fn projection_classes_and_target_lifecycle_share_one_appearance_revision() {
        let mut state = super::super::UiFocusRuntimeState::new_session_restore_candidate();
        let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let mounted = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let incarnation = worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap();
        let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let participant = super::super::UiFocusParticipant::from_mounted(
            crate::mounting::UiMountedFocusParticipant::new(
                crate::graph::UiGraphNodeIdentity::new(1),
                surface,
                mounted,
                incarnation,
                issuer.receipt_for(mounted),
                crate::capability::ComponentFocusSupport::focusable(),
                crate::mounting::UiMountedFocusScope::ActiveSurface,
                0,
            ),
        )
        .unwrap();

        state.observe_window_focus(true);
        state
            .apply_immediate(
                Some(super::super::UiSemanticKeyboardFocus::new(participant)),
                super::super::UiFocusCause::Direct,
                1,
            )
            .unwrap();
        assert_eq!(
            state.appearance_posture().class(),
            super::UiFocusAppearanceClass::Focused
        );
        assert_eq!(state.appearance_posture().owner_revision(), 2);

        state.observe_keyboard_modality();
        assert_eq!(
            state.appearance_posture().class(),
            super::UiFocusAppearanceClass::FocusVisible
        );
        state.observe_window_focus(false);
        assert_eq!(
            state.appearance_posture().class(),
            super::UiFocusAppearanceClass::FocusedWindowInactive
        );
        state
            .apply_immediate(None, super::super::UiFocusCause::Direct, 1)
            .unwrap();
        assert_eq!(
            state.appearance_posture().class(),
            super::UiFocusAppearanceClass::Unfocused
        );
        assert_eq!(state.appearance_posture().owner_revision(), 5);
    }
}
