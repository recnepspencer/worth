//! Atomic lifecycle of the transient records owned by one application attempt.

use super::super::mutation_work::WorthQueryPrimaryMutationWorkCounters;
use crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner;

pub struct WorthQueryApprovedApplicationCandidate {
    pub members: Vec<super::CoupledApplicationMember>,
    pub native: worth_relational::facade::mvcc::ValidatedRelationalProposal,
    pub work: WorthQueryPrimaryMutationWorkCounters,
    pub predicate: Option<PredicatePreparationOwner>,
}
/// The caller releases the actual retired phase after leaving the attempt gate.
pub struct WorthQueryRetiredApplicationAttemptPhase {
    phase_custody: Option<PhaseState>,
}
impl Drop for WorthQueryRetiredApplicationAttemptPhase {
    fn drop(&mut self) {
        drop(self.phase_custody.take());
    }
}

pub(super) struct WorthQueryApplicationAttemptState {
    attempt: super::super::WorthQueryPrimaryGraphApplicationAttempt,
    phase: WorthQueryApplicationAttemptPhase,
}

pub(super) struct WorthQueryApplicationAttemptPhase(PhaseState);

enum PhaseState {
    Registered,
    Staged,
    InvariantApproved(WorthQueryApprovedApplicationCandidate),
    CoupledInvariantApproved(WorthQueryPrimaryMutationWorkCounters),
    Retired,
}

impl WorthQueryApplicationAttemptPhase {
    pub(super) const fn registered() -> Self {
        Self(PhaseState::Registered)
    }

    pub(super) fn stage(&mut self) -> Result<(), &'static str> {
        if !matches!(self.0, PhaseState::Registered) {
            return Err("provider session cannot stage this application candidate");
        }
        self.0 = PhaseState::Staged;
        Ok(())
    }

    fn retire(&mut self) -> Option<WorthQueryRetiredApplicationAttemptPhase> {
        if matches!(self.0, PhaseState::Registered | PhaseState::Retired) {
            return None;
        }
        Some(WorthQueryRetiredApplicationAttemptPhase {
            phase_custody: Some(std::mem::replace(&mut self.0, PhaseState::Retired)),
        })
    }

    pub(super) fn approve(
        &mut self,
        incoming: &mut Option<WorthQueryApprovedApplicationCandidate>,
    ) -> Result<(), &'static str> {
        let prior = std::mem::replace(&mut self.0, PhaseState::Registered);
        match prior {
            PhaseState::Staged => {
                self.0 = PhaseState::InvariantApproved(
                    incoming
                        .take()
                        .expect("caller owns the incoming candidate until approval"),
                );
                Ok(())
            }
            prior => {
                self.0 = prior;
                Err("provider session cannot retain an invariant-approved candidate in this phase")
            }
        }
    }

    fn approve_contribution(
        &mut self,
        approved: WorthQueryPrimaryMutationWorkCounters,
    ) -> Result<(), &'static str> {
        let prior = std::mem::replace(&mut self.0, PhaseState::Registered);
        match prior {
            PhaseState::Staged => {
                self.0 = PhaseState::CoupledInvariantApproved(approved);
                Ok(())
            }
            prior => {
                self.0 = prior;
                Err("coupled writer has no staged contribution")
            }
        }
    }
    pub(super) const fn is_preparable(&self) -> bool {
        matches!(
            self.0,
            PhaseState::InvariantApproved(_) | PhaseState::CoupledInvariantApproved(_)
        )
    }

    pub(super) fn take_commit_ready(self) -> Option<WorthQueryApprovedApplicationCandidate> {
        match self.0 {
            PhaseState::InvariantApproved(approved) => Some(approved),
            _ => None,
        }
    }

    pub(super) const fn resource_count(&self) -> usize {
        match &self.0 {
            PhaseState::Registered | PhaseState::Retired => 1,
            PhaseState::Staged => 1,
            PhaseState::InvariantApproved(_) | PhaseState::CoupledInvariantApproved(_) => 2,
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

    pub(super) fn stage(
        &mut self,
        steps: &[crate::domain_computation::WorthQueryProvisionalEffectStep],
    ) -> Result<(), &'static str> {
        if self.attempt.expected_steps() != steps {
            return Err("lowered effect program conflicts with registered application effects");
        }
        self.phase.stage()
    }

    pub(super) fn staged(&self) -> Option<super::WorthQueryStagedApplicationAttempt<'_>> {
        matches!(
            self.phase.0,
            PhaseState::Staged
                | PhaseState::InvariantApproved(_)
                | PhaseState::CoupledInvariantApproved(_)
        )
        .then_some(super::WorthQueryStagedApplicationAttempt {
            attempt: &self.attempt,
        })
    }

    pub(super) fn attempt_mut(
        &mut self,
    ) -> &mut super::super::WorthQueryPrimaryGraphApplicationAttempt {
        &mut self.attempt
    }
    pub(super) const fn attempt(&self) -> &super::super::WorthQueryPrimaryGraphApplicationAttempt {
        &self.attempt
    }

    pub(super) fn approve(
        &mut self,
        incoming: &mut Option<WorthQueryApprovedApplicationCandidate>,
    ) -> Result<(), &'static str> {
        self.phase.approve(incoming)
    }

    pub(super) fn approve_contribution(
        &mut self,
        approved: WorthQueryPrimaryMutationWorkCounters,
    ) -> Result<(), &'static str> {
        self.phase.approve_contribution(approved)
    }
    pub(super) const fn is_preparable(&self) -> bool {
        self.phase.is_preparable()
    }

    pub(super) fn take_commit_parts(
        self,
    ) -> Option<(
        super::super::WorthQueryPrimaryGraphApplicationAttempt,
        WorthQueryApprovedApplicationCandidate,
    )> {
        let approved = self.phase.take_commit_ready()?;
        Some((self.attempt, approved))
    }

    pub(super) fn coupled_members(&self) -> Option<&[super::CoupledApplicationMember]> {
        match &self.phase.0 {
            PhaseState::InvariantApproved(approved) => Some(&approved.members),
            _ => None,
        }
    }
    pub(super) fn is_coupled_ready(&self) -> bool {
        matches!(self.phase.0, PhaseState::CoupledInvariantApproved(_))
    }
    pub(super) fn take_coupled_parts(
        self,
    ) -> (
        super::super::WorthQueryPrimaryGraphApplicationAttempt,
        WorthQueryPrimaryMutationWorkCounters,
    ) {
        let PhaseState::CoupledInvariantApproved(approved) = self.phase.0 else {
            unreachable!("coupled member extraction follows the atomic precheck")
        };
        (self.attempt, approved)
    }

    pub(super) fn retire_candidate(
        &mut self,
        binding: &crate::domain_computation::WorthQueryProviderSessionTerminalBinding,
    ) -> Option<WorthQueryRetiredApplicationAttemptPhase> {
        if !self.attempt.affinity().same_session(binding) {
            return None;
        }
        self.phase.retire()
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
