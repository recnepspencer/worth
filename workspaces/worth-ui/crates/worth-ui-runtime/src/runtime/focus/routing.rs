#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiFocusRoutingDenial {
    UnknownScope,
    UnknownParticipant,
    StaleParticipantIncarnation,
    StalePlan,
    RevisionExhausted,
    VisitCounterOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiHostFocusTraversalDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiFocusPlan {
    expected_revision: u64,
    next: Option<super::UiSemanticKeyboardFocus>,
    cause: super::UiFocusCause,
    outcome: super::UiFocusOutcome,
    participants_visited: u32,
}

impl UiFocusPlan {
    pub(super) const fn new(
        expected_revision: u64,
        next: Option<super::UiSemanticKeyboardFocus>,
        cause: super::UiFocusCause,
        outcome: super::UiFocusOutcome,
        participants_visited: u32,
    ) -> Self {
        Self {
            expected_revision,
            next,
            cause,
            outcome,
            participants_visited,
        }
    }

    pub(super) const fn expected_revision(self) -> u64 {
        self.expected_revision
    }
    pub(super) const fn next(self) -> Option<super::UiSemanticKeyboardFocus> {
        self.next
    }
    pub(super) const fn cause(self) -> super::UiFocusCause {
        self.cause
    }
    pub(super) const fn participants_visited(self) -> u32 {
        self.participants_visited
    }
}

impl super::UiFocusRuntimeState {
    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) fn plan(
        &self,
        request: super::UiFocusRequest,
    ) -> Result<UiFocusPlan, UiFocusRoutingDenial> {
        match request {
            super::UiFocusRequest::Direct {
                scope,
                participant,
                incarnation,
                cause,
            } => {
                let participant = self.exact_participant(scope, participant, incarnation)?;
                Ok(self.plan_for(Some(participant), cause, 1))
            }
            super::UiFocusRequest::Traverse {
                scope,
                direction,
                wrap,
            } => self.plan_traversal(scope, direction, wrap),
            #[cfg(test)]
            super::UiFocusRequest::First { scope, cause } => {
                Ok(self.plan_for(self.first_in_scope(scope), cause, 1))
            }
            #[cfg(test)]
            super::UiFocusRequest::Restore(token) => self.plan_restoration(token),
        }
    }

    pub(crate) fn commit_host_traversal(
        &mut self,
        scope: super::UiFocusScopeIdentity,
        direction: UiHostFocusTraversalDirection,
        wrap: bool,
    ) -> Result<super::UiFocusTransitionReceipt, UiFocusRoutingDenial> {
        let direction = match direction {
            UiHostFocusTraversalDirection::Forward => super::UiFocusTraversalDirection::Forward,
            UiHostFocusTraversalDirection::Backward => super::UiFocusTraversalDirection::Backward,
        };
        let plan = self.plan_traversal(scope, direction, wrap)?;
        self.commit(plan)
    }

    fn plan_traversal(
        &self,
        scope: super::UiFocusScopeIdentity,
        direction: super::UiFocusTraversalDirection,
        wrap: bool,
    ) -> Result<UiFocusPlan, UiFocusRoutingDenial> {
        let scoped = self
            .participants
            .get(&scope)
            .ok_or(UiFocusRoutingDenial::UnknownScope)?;
        let tab_stops = scoped
            .iter()
            .copied()
            .filter(|participant| participant.container().is_none())
            .collect::<Vec<_>>();
        let current_index = self.current.and_then(|current| {
            (current.scope() == scope)
                .then(|| {
                    let participant = self.exact_current_successor(current)?;
                    let tab_identity = participant.container().unwrap_or(participant.identity());
                    tab_stops
                        .iter()
                        .position(|candidate| candidate.identity() == tab_identity)
                })
                .flatten()
        });
        let next = match (direction, current_index) {
            (super::UiFocusTraversalDirection::Forward, Some(index)) => tab_stops
                .get(index + 1)
                .copied()
                .or_else(|| wrap.then(|| tab_stops.first().copied()).flatten()),
            (super::UiFocusTraversalDirection::Backward, Some(index)) => index
                .checked_sub(1)
                .and_then(|previous| tab_stops.get(previous).copied())
                .or_else(|| wrap.then(|| tab_stops.last().copied()).flatten()),
            (super::UiFocusTraversalDirection::Forward, None) => tab_stops.first().copied(),
            (super::UiFocusTraversalDirection::Backward, None) => tab_stops.last().copied(),
        };
        Ok(self.plan_for(
            next,
            super::UiFocusCause::KeyboardTraversal,
            u32::from(next.is_some()),
        ))
    }

    #[cfg(test)]
    pub(super) fn first_in_scope(
        &self,
        scope: super::UiFocusScopeIdentity,
    ) -> Option<super::UiFocusParticipant> {
        self.participants
            .get(&scope)?
            .iter()
            .find(|participant| participant.container().is_none())
            .copied()
    }

    pub(crate) fn default_scope_for_surface(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<super::UiFocusScopeIdentity> {
        self.participants
            .keys()
            .copied()
            .find(|scope| scope.semantic_surface() == surface)
    }

    pub(super) fn plan_for(
        &self,
        next: Option<super::UiFocusParticipant>,
        cause: super::UiFocusCause,
        participants_visited: u32,
    ) -> UiFocusPlan {
        let next = next.map(super::UiSemanticKeyboardFocus::new);
        let outcome = transition_outcome(self.current, next);
        UiFocusPlan::new(self.revision, next, cause, outcome, participants_visited)
    }
}

pub(super) fn transition_outcome(
    previous: Option<super::UiSemanticKeyboardFocus>,
    next: Option<super::UiSemanticKeyboardFocus>,
) -> super::UiFocusOutcome {
    match (previous, next) {
        (Some(previous), Some(next)) if previous.participant() == next.participant() => {
            super::UiFocusOutcome::Unchanged
        }
        (_, Some(_)) => super::UiFocusOutcome::Moved,
        (Some(_), None) => super::UiFocusOutcome::Cleared,
        (None, None) => super::UiFocusOutcome::NoEligibleParticipant,
    }
}
