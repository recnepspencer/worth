#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiHostFocusNavigation {
    Traverse(crate::runtime::focus::UiHostFocusTraversalDirection),
    Container(crate::runtime::focus::UiFocusContainerNavigationKey),
}

impl super::WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn observe_focus_navigation_report(
        &mut self,
        payload: &worth_ui_host_contract::UiHostObservationPayload,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> bool {
        if self.ime_composing {
            return false;
        }
        let Some(navigation) = host_focus_navigation(payload) else {
            return false;
        };
        let Some(publication) = self.mounted.current_publication().cloned() else {
            return false;
        };
        let Ok(surface) = self
            .mounted
            .current_semantic_surface_for_presentation(presentation)
        else {
            return false;
        };
        if let UiHostFocusNavigation::Traverse(direction) = navigation {
            let transition = {
                let Some(focus) = self.focus.as_mut() else {
                    return false;
                };
                let scope = focus
                    .current_semantic_focus()
                    .filter(|current| current.scope().semantic_surface() == surface)
                    .map(crate::runtime::focus::UiSemanticKeyboardFocus::scope)
                    .or_else(|| focus.default_scope_for_surface(surface));
                let Some(scope) = scope else {
                    return false;
                };
                let Ok(transition) = focus.commit_host_traversal(
                    scope,
                    direction,
                    scope.kind() == crate::capability::MosaicFocusScopeKind::ModalTrapScope,
                ) else {
                    return false;
                };
                transition
            };
            let _placement = self.place_committed_semantic_focus(transition, &publication);
            return true;
        }
        let UiHostFocusNavigation::Container(key) = navigation else {
            unreachable!("traversal returned above")
        };
        let navigation = {
            let Some(focus) = self.focus.as_mut() else {
                return false;
            };
            match focus.navigate_container(surface, key) {
                Ok(Some(navigation)) => navigation,
                Ok(None) | Err(_) => return false,
            }
        };
        if let crate::runtime::focus::UiFocusContainerNavigationReceipt::Roving(transition) =
            navigation
        {
            let _placement = self.place_committed_semantic_focus(transition, &publication);
        }
        true
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn inspect_focus_runtime_for_certification(
        &self,
    ) -> crate::certification_support::UiFocusRuntimeCertificationSnapshot {
        let Some(focus) = self.focus.as_ref() else {
            return crate::certification_support::UiFocusRuntimeCertificationSnapshot::uninstalled(
            );
        };
        let (current, active_descendant, participants, pending, revision) =
            focus.inspect_for_certification();
        crate::certification_support::UiFocusRuntimeCertificationSnapshot::new(
            current,
            active_descendant,
            participants,
            pending,
            revision,
        )
    }
}

fn host_focus_navigation(
    payload: &worth_ui_host_contract::UiHostObservationPayload,
) -> Option<UiHostFocusNavigation> {
    let worth_ui_host_contract::UiHostObservationPayload::Keyboard {
        logical_key,
        modifiers,
        transition: worth_ui_host_contract::UiHostKeyTransition::Pressed { repeat: false },
        ..
    } = payload
    else {
        return None;
    };
    if modifiers.alt() || modifiers.control() || modifiers.mac_command() || modifiers.command() {
        return None;
    }
    if *logical_key == worth_ui_host_contract::UiHostKey::Tab {
        return Some(UiHostFocusNavigation::Traverse(if modifiers.shift() {
            crate::runtime::focus::UiHostFocusTraversalDirection::Backward
        } else {
            crate::runtime::focus::UiHostFocusTraversalDirection::Forward
        }));
    }
    if modifiers.shift() {
        return None;
    }
    Some(UiHostFocusNavigation::Container(match logical_key {
        worth_ui_host_contract::UiHostKey::ArrowLeft => {
            crate::runtime::focus::UiFocusContainerNavigationKey::Left
        }
        worth_ui_host_contract::UiHostKey::ArrowRight => {
            crate::runtime::focus::UiFocusContainerNavigationKey::Right
        }
        worth_ui_host_contract::UiHostKey::ArrowUp => {
            crate::runtime::focus::UiFocusContainerNavigationKey::Up
        }
        worth_ui_host_contract::UiHostKey::ArrowDown => {
            crate::runtime::focus::UiFocusContainerNavigationKey::Down
        }
        _ => return None,
    }))
}

#[cfg(test)]
mod tests {
    use super::{host_focus_navigation, UiHostFocusNavigation};

    #[test]
    fn production_keyboard_parser_distinguishes_traversal_and_container_navigation() {
        let tab = keyboard(
            worth_ui_host_contract::UiHostKey::Tab,
            worth_ui_host_contract::UiHostKeyboardModifiers::default(),
            false,
        );
        let backward = keyboard(
            worth_ui_host_contract::UiHostKey::Tab,
            worth_ui_host_contract::UiHostKeyboardModifiers::new(false, false, true, false, false),
            false,
        );
        let down = keyboard(
            worth_ui_host_contract::UiHostKey::ArrowDown,
            worth_ui_host_contract::UiHostKeyboardModifiers::default(),
            false,
        );

        assert_eq!(
            host_focus_navigation(&tab),
            Some(UiHostFocusNavigation::Traverse(
                crate::runtime::focus::UiHostFocusTraversalDirection::Forward
            ))
        );
        assert_eq!(
            host_focus_navigation(&backward),
            Some(UiHostFocusNavigation::Traverse(
                crate::runtime::focus::UiHostFocusTraversalDirection::Backward
            ))
        );
        assert_eq!(
            host_focus_navigation(&down),
            Some(UiHostFocusNavigation::Container(
                crate::runtime::focus::UiFocusContainerNavigationKey::Down
            ))
        );
    }

    #[test]
    fn production_keyboard_parser_rejects_repeat_and_modified_arrow_navigation() {
        assert_eq!(
            host_focus_navigation(&keyboard(
                worth_ui_host_contract::UiHostKey::Tab,
                worth_ui_host_contract::UiHostKeyboardModifiers::default(),
                true,
            )),
            None
        );
        assert_eq!(
            host_focus_navigation(&keyboard(
                worth_ui_host_contract::UiHostKey::ArrowRight,
                worth_ui_host_contract::UiHostKeyboardModifiers::new(
                    false, true, false, false, false,
                ),
                false,
            )),
            None
        );
    }

    fn keyboard(
        logical_key: worth_ui_host_contract::UiHostKey,
        modifiers: worth_ui_host_contract::UiHostKeyboardModifiers,
        repeat: bool,
    ) -> worth_ui_host_contract::UiHostObservationPayload {
        worth_ui_host_contract::UiHostObservationPayload::Keyboard {
            logical_key,
            physical_key: None,
            modifiers,
            transition: worth_ui_host_contract::UiHostKeyTransition::Pressed { repeat },
        }
    }
}
