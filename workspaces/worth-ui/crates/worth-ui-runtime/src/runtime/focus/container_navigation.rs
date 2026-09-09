#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiFocusContainerNavigationKey {
    Left,
    Right,
    Up,
    Down,
}

pub(crate) enum UiFocusContainerNavigationReceipt {
    Roving(super::UiFocusTransitionReceipt),
    ActiveDescendant,
}

impl super::UiFocusRuntimeState {
    pub(crate) fn navigate_container(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        key: UiFocusContainerNavigationKey,
    ) -> Result<Option<UiFocusContainerNavigationReceipt>, super::UiFocusRoutingDenial> {
        let Some(current) = self
            .current
            .filter(|current| current.scope().semantic_surface() == surface)
        else {
            return Ok(None);
        };
        let Some(current_participant) = self.exact_current_successor(current) else {
            return Ok(None);
        };
        let container_identity = current_participant
            .container()
            .unwrap_or(current_participant.identity());
        let Some(container) = self
            .exact_participant(current.scope(), container_identity, current.incarnation())
            .ok()
            .or_else(|| {
                self.participant_index
                    .get(&container_identity)
                    .and_then(|(scope, index)| self.participants.get(scope)?.get(*index))
                    .copied()
            })
        else {
            return Ok(None);
        };
        let Some(policy) = container.container_policy() else {
            return Ok(None);
        };
        let Some(forward) = direction_for(policy.axis(), key) else {
            return Ok(None);
        };
        let children = self.participants[&current.scope()]
            .iter()
            .copied()
            .filter(|participant| participant.container() == Some(container_identity))
            .collect::<Vec<_>>();
        if children.is_empty() {
            return Ok(None);
        }
        let current_child = match policy {
            crate::capability::ComponentFocusContainerPolicy::Roving { .. } => current_participant
                .container()
                .map(|_| current_participant.identity()),
            crate::capability::ComponentFocusContainerPolicy::ActiveDescendant { .. } => self
                .active_descendant
                .filter(|active| active.composite() == container_identity)
                .map(super::UiActiveDescendant::descendant),
        };
        let next = next_child(&children, current_child, forward, policy.wraps());
        let Some(next) = next else {
            return Ok(None);
        };
        match policy {
            crate::capability::ComponentFocusContainerPolicy::Roving { .. } => self
                .apply_immediate(
                    Some(super::UiSemanticKeyboardFocus::new(next)),
                    super::UiFocusCause::RovingMovement,
                    1,
                )
                .map(UiFocusContainerNavigationReceipt::Roving)
                .map(Some),
            crate::capability::ComponentFocusContainerPolicy::ActiveDescendant { .. } => {
                if current.participant() != container_identity {
                    return Ok(None);
                }
                self.move_active_descendant(
                    container_identity,
                    next.identity(),
                    next.incarnation(),
                )
                .map_err(map_active_descendant_denial)?;
                Ok(Some(UiFocusContainerNavigationReceipt::ActiveDescendant))
            }
        }
    }
}

fn direction_for(
    axis: crate::capability::ComponentFocusNavigationAxis,
    key: UiFocusContainerNavigationKey,
) -> Option<bool> {
    use crate::capability::ComponentFocusNavigationAxis as Axis;
    use UiFocusContainerNavigationKey as Key;
    match (axis, key) {
        (Axis::Horizontal | Axis::Both, Key::Right) | (Axis::Vertical | Axis::Both, Key::Down) => {
            Some(true)
        }
        (Axis::Horizontal | Axis::Both, Key::Left) | (Axis::Vertical | Axis::Both, Key::Up) => {
            Some(false)
        }
        _ => None,
    }
}

fn next_child(
    children: &[super::UiFocusParticipant],
    current: Option<super::UiFocusParticipantIdentity>,
    forward: bool,
    wrap: bool,
) -> Option<super::UiFocusParticipant> {
    let current = current.and_then(|identity| {
        children
            .iter()
            .position(|participant| participant.identity() == identity)
    });
    match (forward, current) {
        (true, Some(index)) => children
            .get(index + 1)
            .copied()
            .or_else(|| wrap.then(|| children.first().copied()).flatten()),
        (false, Some(index)) => index
            .checked_sub(1)
            .and_then(|index| children.get(index).copied())
            .or_else(|| wrap.then(|| children.last().copied()).flatten()),
        (true, None) => children.first().copied(),
        (false, None) => children.last().copied(),
    }
}

fn map_active_descendant_denial(
    denial: super::active_descendant::UiActiveDescendantDenial,
) -> super::UiFocusRoutingDenial {
    match denial {
        super::active_descendant::UiActiveDescendantDenial::RevisionExhausted => {
            super::UiFocusRoutingDenial::RevisionExhausted
        }
        super::active_descendant::UiActiveDescendantDenial::StaleDescendantIncarnation => {
            super::UiFocusRoutingDenial::StaleParticipantIncarnation
        }
        super::active_descendant::UiActiveDescendantDenial::UnknownDescendant => {
            super::UiFocusRoutingDenial::UnknownParticipant
        }
        _ => super::UiFocusRoutingDenial::UnknownScope,
    }
}
