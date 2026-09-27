#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiActiveDescendant {
    composite: super::UiFocusParticipantIdentity,
    scope: super::UiFocusScopeIdentity,
    descendant: super::UiFocusParticipantIdentity,
    descendant_incarnation: worth_ui_host_contract::UiMountIncarnation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum UiActiveDescendantDenial {
    NoSemanticFocus,
    CompositeMismatch,
    CompositePolicyMismatch,
    CompositeCannotBeItsOwnDescendant,
    UnknownDescendant,
    StaleDescendantIncarnation,
    ForeignScope,
    RevisionExhausted,
}

impl super::UiFocusRuntimeState {
    pub(in crate::runtime) fn move_active_descendant(
        &mut self,
        composite: super::UiFocusParticipantIdentity,
        descendant: super::UiFocusParticipantIdentity,
        descendant_incarnation: worth_ui_host_contract::UiMountIncarnation,
    ) -> Result<(), UiActiveDescendantDenial> {
        let current = self
            .current
            .ok_or(UiActiveDescendantDenial::NoSemanticFocus)?;
        if current.participant() != composite {
            return Err(UiActiveDescendantDenial::CompositeMismatch);
        }
        let composite_participant = self
            .exact_current_successor(current)
            .ok_or(UiActiveDescendantDenial::CompositeMismatch)?;
        if !matches!(
            composite_participant.container_policy(),
            Some(crate::capability::ComponentFocusContainerPolicy::ActiveDescendant { .. })
        ) {
            return Err(UiActiveDescendantDenial::CompositePolicyMismatch);
        }
        if composite == descendant {
            return Err(UiActiveDescendantDenial::CompositeCannotBeItsOwnDescendant);
        }
        let descendant = self
            .exact_participant(current.scope(), descendant, descendant_incarnation)
            .map_err(|denial| match denial {
                super::UiFocusRoutingDenial::StaleParticipantIncarnation => {
                    UiActiveDescendantDenial::StaleDescendantIncarnation
                }
                super::UiFocusRoutingDenial::UnknownParticipant => {
                    UiActiveDescendantDenial::UnknownDescendant
                }
                super::UiFocusRoutingDenial::UnknownScope
                | super::UiFocusRoutingDenial::StalePlan
                | super::UiFocusRoutingDenial::RevisionExhausted
                | super::UiFocusRoutingDenial::VisitCounterOverflow => {
                    UiActiveDescendantDenial::ForeignScope
                }
            })?;
        if descendant.scope() != current.scope() {
            return Err(UiActiveDescendantDenial::ForeignScope);
        }
        let next = UiActiveDescendant {
            composite,
            scope: current.scope(),
            descendant: descendant.identity(),
            descendant_incarnation: descendant.incarnation(),
        };
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or(UiActiveDescendantDenial::RevisionExhausted)?;
        self.active_descendant = Some(next);
        Ok(())
    }
}

impl UiActiveDescendant {
    pub(in crate::runtime) const fn composite(self) -> super::UiFocusParticipantIdentity {
        self.composite
    }

    pub(in crate::runtime) const fn descendant(self) -> super::UiFocusParticipantIdentity {
        self.descendant
    }

    pub(in crate::runtime) const fn scope(self) -> super::UiFocusScopeIdentity {
        self.scope
    }

    pub(in crate::runtime) const fn descendant_incarnation(
        self,
    ) -> worth_ui_host_contract::UiMountIncarnation {
        self.descendant_incarnation
    }
}
