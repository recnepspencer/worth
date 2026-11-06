//! Atomic lifecycle of the transient records owned by one application attempt.

use super::super::mutation_work::WorthQueryPrimaryMutationWorkCounters;
use crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner;
use crate::domain_computation::WorthQueryProposedFact;

pub(in crate::domain_computation::primary_graph::provider) struct WorthQueryApprovedApplicationCandidate
{
    pub(in crate::domain_computation::primary_graph::provider) native:
        worth_relational::facade::mvcc::ValidatedRelationalProposal,
    pub(in crate::domain_computation::primary_graph::provider) work:
        WorthQueryPrimaryMutationWorkCounters,
    pub(in crate::domain_computation::primary_graph::provider) predicate:
        Option<PredicatePreparationOwner>,
}
/// The caller releases the actual retired phase after leaving the attempt gate.
pub(in crate::domain_computation::primary_graph::provider) struct WorthQueryRetiredApplicationAttemptPhase
{
    phase_custody: Option<PhaseState>,
}
impl Drop for WorthQueryRetiredApplicationAttemptPhase {
    fn drop(&mut self) {
        drop(self.phase_custody.take());
    }
}

pub(super) struct WorthQueryApplicationAttemptState {
    attempt: crate::domain_computation::primary_graph::provider::application_attempt_state::registration::registered_attempt::WorthQueryPrimaryGraphApplicationAttempt,
    phase: WorthQueryApplicationAttemptPhase,
}

pub(super) struct WorthQueryApplicationAttemptPhase(PhaseState);

enum PhaseState {
    Registered,
    OverlayStaged(WorthQueryPrimaryGraphOverlay),
    InvariantApproved {
        overlay: WorthQueryPrimaryGraphOverlay,
        approved: WorthQueryApprovedApplicationCandidate,
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

    pub(super) fn discard_overlay(
        &mut self,
        physical_overlay_identity: &str,
    ) -> Option<WorthQueryRetiredApplicationAttemptPhase> {
        if self
            .overlay()
            .is_none_or(|overlay| overlay.identity != physical_overlay_identity)
        {
            return None;
        }
        let prior = std::mem::replace(&mut self.0, PhaseState::Registered);
        if matches!(&prior, PhaseState::InvariantApproved { .. }) {
            self.0 = PhaseState::InvariantApprovedAfterOverlayDiscard;
        }
        Some(WorthQueryRetiredApplicationAttemptPhase {
            phase_custody: Some(prior),
        })
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
        incoming: &mut Option<WorthQueryApprovedApplicationCandidate>,
    ) -> Result<(), &'static str> {
        let prior = std::mem::replace(&mut self.0, PhaseState::Registered);
        match prior {
            PhaseState::OverlayStaged(overlay) => {
                self.0 = PhaseState::InvariantApproved {
                    overlay,
                    approved: incoming
                        .take()
                        .expect("caller owns the incoming candidate until approval"),
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
            PhaseState::InvariantApproved { approved, .. } => Some(&approved.native),
            _ => None,
        }
    }

    pub(super) fn take_commit_ready(self) -> Option<WorthQueryApprovedApplicationCandidate> {
        match self.0 {
            PhaseState::InvariantApproved { approved, .. } => Some(approved),
            _ => None,
        }
    }

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
        attempt: crate::domain_computation::primary_graph::provider::application_attempt_state::registration::registered_attempt::WorthQueryPrimaryGraphApplicationAttempt,
    ) -> Self {
        Self {
            attempt,
            phase: WorthQueryApplicationAttemptPhase::registered(),
        }
    }

    pub(super) const fn attempt(&self) -> &crate::domain_computation::primary_graph::provider::application_attempt_state::registration::registered_attempt::WorthQueryPrimaryGraphApplicationAttempt{
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
        incoming: &mut Option<WorthQueryApprovedApplicationCandidate>,
    ) -> Result<(), &'static str> {
        self.phase.approve(incoming)
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
        crate::domain_computation::primary_graph::provider::application_attempt_state::registration::registered_attempt::WorthQueryPrimaryGraphApplicationAttempt,
        WorthQueryApprovedApplicationCandidate,
    )>{
        let approved = self.phase.take_commit_ready()?;
        Some((self.attempt, approved))
    }

    pub(super) fn discard_overlay(
        &mut self,
        cleanup: &crate::domain_computation::provider_session::WorthQueryProvisionalOverlayCleanupBinding,
        physical_overlay_identity: &str,
    ) -> Option<WorthQueryRetiredApplicationAttemptPhase> {
        if !self.attempt.affinity().admits_cleanup(cleanup) {
            return None;
        }
        self.phase.discard_overlay(physical_overlay_identity)
    }

    pub(super) fn admits_session(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
    ) -> bool {
        self.attempt.affinity().admits_session(session)
    }

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
