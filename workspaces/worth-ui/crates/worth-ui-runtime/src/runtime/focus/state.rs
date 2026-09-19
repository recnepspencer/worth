use std::collections::BTreeMap;

#[path = "state/observation.rs"]
mod observation;
#[path = "state/portal_proposal.rs"]
mod portal_proposal;

/// Sole owner of semantic keyboard focus for one active application session.
pub(crate) struct UiFocusRuntimeState {
    persistence: crate::runtime::UiServiceStatePersistencePosture,
    policy: crate::declaration::UiFocusPolicy,
    pub(super) participants: BTreeMap<super::UiFocusScopeIdentity, Vec<super::UiFocusParticipant>>,
    pub(super) participant_index:
        BTreeMap<super::UiFocusParticipantIdentity, (super::UiFocusScopeIdentity, usize)>,
    pub(super) current: Option<super::UiSemanticKeyboardFocus>,
    pub(super) active_descendant: Option<super::UiActiveDescendant>,
    pub(super) window_focus: super::UiWindowFocus,
    pub(super) modality: super::UiFocusVisibleModality,
    pub(super) pending_portal: BTreeMap<
        crate::runtime::session::service_proposal::UiServiceProposalIdentity,
        super::portal_transition::UiPreparedPortalFocusTransition,
    >,
    pub(super) portal_restorations:
        BTreeMap<super::UiPortalFocusBoundaryIdentity, Option<super::UiFocusRestorationToken>>,
    pub(super) structural_revision: u64,
    pub(super) revision: u64,
    pub(super) appearance_revision: u64,
    pub(super) last_transition: Option<super::UiFocusTransitionReceipt>,
    pub(super) last_restoration_failure: Option<super::UiFocusTransitionReceipt>,
}

impl UiFocusRuntimeState {
    pub(crate) const fn new_session_restore_candidate() -> Self {
        Self::new_with_policy(
            crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
            crate::declaration::UiFocusPolicy::workbench(),
        )
    }

    pub(crate) const fn new_session_restore_candidate_with_policy(
        policy: crate::declaration::UiFocusPolicy,
    ) -> Self {
        Self::new_with_policy(
            crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
            policy,
        )
    }

    const fn new_with_policy(
        persistence: crate::runtime::UiServiceStatePersistencePosture,
        policy: crate::declaration::UiFocusPolicy,
    ) -> Self {
        Self {
            persistence,
            policy,
            participants: BTreeMap::new(),
            participant_index: BTreeMap::new(),
            current: None,
            active_descendant: None,
            window_focus: super::UiWindowFocus::Unfocused,
            modality: super::UiFocusVisibleModality::Initial,
            pending_portal: BTreeMap::new(),
            portal_restorations: BTreeMap::new(),
            structural_revision: 0,
            revision: 0,
            appearance_revision: 0,
            last_transition: None,
            last_restoration_failure: None,
        }
    }

    pub(crate) fn apply_policy(&mut self, policy: crate::declaration::UiFocusPolicy) {
        self.policy = policy;
    }

    pub(crate) fn shutdown(&mut self) -> usize {
        debug_assert_eq!(
            self.persistence,
            crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate
        );
        let released = self.participant_index.len()
            + self.pending_portal.len()
            + self.portal_restorations.len()
            + usize::from(self.current.is_some())
            + usize::from(self.active_descendant.is_some());
        self.participants.clear();
        self.participant_index.clear();
        self.current = None;
        self.active_descendant = None;
        self.pending_portal.clear();
        self.portal_restorations.clear();
        self.last_transition = None;
        self.last_restoration_failure = None;
        released
    }

    pub(in crate::runtime) fn commit(
        &mut self,
        plan: super::UiFocusPlan,
    ) -> Result<super::UiFocusTransitionReceipt, super::UiFocusRoutingDenial> {
        if plan.expected_revision() != self.revision {
            return Err(super::UiFocusRoutingDenial::StalePlan);
        }
        self.apply_immediate(plan.next(), plan.cause(), plan.participants_visited())
    }

    pub(super) fn exact_current_successor(
        &self,
        current: super::UiSemanticKeyboardFocus,
    ) -> Option<super::UiFocusParticipant> {
        self.exact_participant(
            current.scope(),
            current.participant(),
            current.incarnation(),
        )
        .ok()
    }

    pub(super) fn exact_participant(
        &self,
        scope: super::UiFocusScopeIdentity,
        identity: super::UiFocusParticipantIdentity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
    ) -> Result<super::UiFocusParticipant, super::UiFocusRoutingDenial> {
        let Some((indexed_scope, index)) = self.participant_index.get(&identity).copied() else {
            return Err(super::UiFocusRoutingDenial::UnknownParticipant);
        };
        if indexed_scope != scope {
            return Err(super::UiFocusRoutingDenial::UnknownParticipant);
        }
        let participant = self.participants[&scope][index];
        if participant.incarnation() != incarnation {
            return Err(super::UiFocusRoutingDenial::StaleParticipantIncarnation);
        }
        Ok(participant)
    }

    pub(super) fn apply_immediate(
        &mut self,
        next: Option<super::UiSemanticKeyboardFocus>,
        cause: super::UiFocusCause,
        participants_visited: u32,
    ) -> Result<super::UiFocusTransitionReceipt, super::UiFocusRoutingDenial> {
        let previous = self.current;
        let previous_modality = self.modality;
        let outcome = super::routing::transition_outcome(previous, next);
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or(super::UiFocusRoutingDenial::RevisionExhausted)?;
        self.current = next;
        if self.active_descendant.is_some_and(|active| {
            next.is_none_or(|focus| focus.participant() != active.composite())
        }) {
            self.active_descendant = None;
        }
        if matches!(
            cause,
            super::UiFocusCause::KeyboardTraversal | super::UiFocusCause::RovingMovement
        ) {
            self.modality = super::UiFocusVisibleModality::Keyboard;
        }
        if previous != self.current || previous_modality != self.modality {
            self.bump_appearance_revision();
        }
        let receipt = super::UiFocusTransitionReceipt::new(
            previous,
            next,
            cause,
            outcome,
            participants_visited,
            self.revision,
        );
        self.last_transition = Some(receipt);
        if cause == super::UiFocusCause::PortalRestoration
            && outcome == super::UiFocusOutcome::NoEligibleParticipant
        {
            self.last_restoration_failure = Some(receipt);
        }
        Ok(receipt)
    }

    pub(super) fn bump_appearance_revision(&mut self) {
        self.appearance_revision = self
            .appearance_revision
            .checked_add(1)
            .expect("bounded focus appearance revision exhausted");
    }
}
