//! Atomic lifecycle of the transient records owned by one application attempt.

use super::super::mutation_work::WorthQueryPrimaryMutationWorkCounters;
use crate::domain_computation::WorthQueryProposedFact;

pub(super) struct WorthQueryApplicationAttemptState {
    attempt: super::super::WorthQueryPrimaryGraphApplicationAttempt,
    phase: WorthQueryApplicationAttemptPhase,
}

pub(super) struct WorthQueryApplicationAttemptPhase(PhaseState);

enum PhaseState {
    Registered,
    OverlayStaged(WorthQueryPrimaryGraphOverlay),
    InvariantApproved {
        overlay: WorthQueryPrimaryGraphOverlay,
        candidate: worth_relational::facade::mvcc::ValidatedRelationalProposal,
        work: WorthQueryPrimaryMutationWorkCounters,
    },
    InvariantApprovedAfterOverlayDiscard,
}

pub(super) struct WorthQueryPrimaryGraphOverlay {
    identity: String,
    facts: Vec<WorthQueryProposedFact>,
}

impl WorthQueryApplicationAttemptPhase {
    pub(super) const fn registered() -> Self {
        Self(PhaseState::Registered)
    }

    pub(super) const fn accepts_overlay(&self) -> bool {
        matches!(self.0, PhaseState::Registered)
    }

    pub(super) fn stage_overlay(
        &mut self,
        overlay: WorthQueryPrimaryGraphOverlay,
    ) -> Result<(), &'static str> {
        if !self.accepts_overlay() {
            return Err("provider session cannot stage this application overlay");
        }
        self.0 = PhaseState::OverlayStaged(overlay);
        Ok(())
    }

    pub(super) fn discard_overlay(&mut self, physical_overlay_identity: &str) -> bool {
        let prior = std::mem::replace(&mut self.0, PhaseState::Registered);
        match prior {
            PhaseState::OverlayStaged(overlay) if overlay.identity == physical_overlay_identity => {
                true
            }
            PhaseState::InvariantApproved { overlay, .. }
                if overlay.identity == physical_overlay_identity =>
            {
                self.0 = PhaseState::InvariantApprovedAfterOverlayDiscard;
                true
            }
            prior => {
                self.0 = prior;
                false
            }
        }
    }

    pub(super) fn overlay(&self) -> Option<&WorthQueryPrimaryGraphOverlay> {
        match &self.0 {
            PhaseState::OverlayStaged(overlay) | PhaseState::InvariantApproved { overlay, .. } => {
                Some(overlay)
            }
            PhaseState::Registered | PhaseState::InvariantApprovedAfterOverlayDiscard => None,
        }
    }

    pub(super) fn approve(
        &mut self,
        candidate: worth_relational::facade::mvcc::ValidatedRelationalProposal,
        work: WorthQueryPrimaryMutationWorkCounters,
    ) -> Result<(), &'static str> {
        let prior = std::mem::replace(&mut self.0, PhaseState::Registered);
        match prior {
            PhaseState::OverlayStaged(overlay) => {
                self.0 = PhaseState::InvariantApproved {
                    overlay,
                    candidate,
                    work,
                };
                Ok(())
            }
            prior => {
                self.0 = prior;
                Err("provider session cannot retain an invariant-approved candidate in this phase")
            }
        }
    }

    pub(super) const fn is_preparable(&self) -> bool {
        matches!(
            self.0,
            PhaseState::OverlayStaged(_) | PhaseState::InvariantApproved { .. }
        )
    }

    pub(super) const fn is_commit_ready(&self) -> bool {
        matches!(self.0, PhaseState::InvariantApproved { .. })
    }

    pub(super) fn approved_candidate(
        &self,
    ) -> Option<&worth_relational::facade::mvcc::ValidatedRelationalProposal> {
        match &self.0 {
            PhaseState::InvariantApproved { candidate, .. } => Some(candidate),
            _ => None,
        }
    }

    pub(super) fn take_commit_ready(
        self,
    ) -> Option<(
        worth_relational::facade::mvcc::ValidatedRelationalProposal,
        WorthQueryPrimaryMutationWorkCounters,
    )> {
        match self.0 {
            PhaseState::InvariantApproved {
                candidate, work, ..
            } => Some((candidate, work)),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(super) const fn resource_count(&self) -> usize {
        match &self.0 {
            PhaseState::Registered | PhaseState::InvariantApprovedAfterOverlayDiscard => 1,
            PhaseState::OverlayStaged(_) => 3,
            PhaseState::InvariantApproved { .. } => 5,
        }
    }
}

impl WorthQueryApplicationAttemptState {
    pub(super) const fn registered(
        attempt: super::super::WorthQueryPrimaryGraphApplicationAttempt,
    ) -> Self {
        Self {
            attempt,
            phase: WorthQueryApplicationAttemptPhase::registered(),
        }
    }

    pub(super) const fn attempt(&self) -> &super::super::WorthQueryPrimaryGraphApplicationAttempt {
        &self.attempt
    }

    pub(super) fn accepts_overlay(
        &self,
        expected_steps: &[crate::domain_computation::WorthQueryProvisionalEffectStep],
    ) -> bool {
        self.attempt.expected_steps() == expected_steps && self.phase.accepts_overlay()
    }

    pub(super) fn stage_overlay(
        &mut self,
        overlay: WorthQueryPrimaryGraphOverlay,
    ) -> Result<(), &'static str> {
        self.phase.stage_overlay(overlay)
    }

    pub(super) fn staged(&self) -> Option<super::WorthQueryStagedApplicationAttempt<'_>> {
        Some(super::WorthQueryStagedApplicationAttempt {
            attempt: &self.attempt,
            overlay: self.phase.overlay()?,
        })
    }

    pub(super) fn approve(
        &mut self,
        candidate: worth_relational::facade::mvcc::ValidatedRelationalProposal,
        work: WorthQueryPrimaryMutationWorkCounters,
    ) -> Result<(), &'static str> {
        self.phase.approve(candidate, work)
    }

    pub(super) const fn is_preparable(&self) -> bool {
        self.phase.is_preparable()
    }

    pub(super) const fn phase_is_commit_ready(&self) -> bool {
        self.phase.is_commit_ready()
    }

    pub(super) fn approved_candidate(
        &self,
    ) -> Option<&worth_relational::facade::mvcc::ValidatedRelationalProposal> {
        self.phase.approved_candidate()
    }

    pub(super) fn take_commit_parts(
        self,
    ) -> Option<(
        super::super::WorthQueryPrimaryGraphApplicationAttempt,
        worth_relational::facade::mvcc::ValidatedRelationalProposal,
        WorthQueryPrimaryMutationWorkCounters,
    )> {
        let (candidate, work) = self.phase.take_commit_ready()?;
        Some((self.attempt, candidate, work))
    }

    pub(super) fn discard_overlay(
        &mut self,
        cleanup: &crate::domain_computation::provider_session::WorthQueryProvisionalOverlayCleanupBinding,
        physical_overlay_identity: &str,
    ) -> bool {
        self.attempt.affinity().admits_cleanup(cleanup)
            && self.phase.discard_overlay(physical_overlay_identity)
    }

    pub(super) fn admits_session(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
    ) -> bool {
        self.attempt.affinity().admits_session(session)
    }

    #[cfg(test)]
    pub(super) const fn resource_count(&self) -> usize {
        self.phase.resource_count()
    }
}

impl WorthQueryPrimaryGraphOverlay {
    pub(super) fn new(identity: String, facts: Vec<WorthQueryProposedFact>) -> Self {
        Self { identity, facts }
    }
    pub(super) fn identity(&self) -> &str {
        &self.identity
    }
    pub(super) fn facts(&self) -> &[WorthQueryProposedFact] {
        &self.facts
    }
}
