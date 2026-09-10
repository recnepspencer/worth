//! Affinity-keyed transient state for one Primary Graph application attempt.

use crate::domain_computation::{
    WorthQueryProposedFact, WorthQueryProviderSessionAffinityIdentity,
    WorthQueryProviderSessionView, WorthQueryProvisionalEffectStep,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

mod commit_completion;
mod commit_preparation;
mod phase;
mod registration;
mod retained_basis;
use commit_completion::WorthQueryPreparedProviderApplicationAttempt;
pub(in crate::domain_computation::primary_graph::provider) use commit_preparation::commit_prepared_application;
pub(crate) use commit_preparation::WorthQueryRetainedPreImageSeal;
pub(in crate::domain_computation::primary_graph) use commit_preparation::{
    WorthQueryMutationWorkCommitSeal, WorthQueryPreImageRetentionWork,
    WorthQueryPrimaryGraphCommittedApplication,
};
#[cfg(test)]
use phase::WorthQueryApplicationAttemptPhase;
use phase::{WorthQueryApplicationAttemptState, WorthQueryPrimaryGraphOverlay};
pub(super) use retained_basis::{
    WorthQueryAdmittedApplicationOverlay, WorthQueryApplicationIdempotencyBasis,
    WorthQueryObservedApplicationFactBasis,
};

use super::{
    mutation_work::WorthQueryPrimaryMutationWorkCounters, WorthQueryPrimaryGraphApplicationAttempt,
    WorthQueryPrimaryGraphApplicationDecisionFact,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct WorthQueryApplicationAttemptLookupKey(WorthQueryProviderSessionAffinityIdentity);

impl WorthQueryApplicationAttemptLookupKey {
    fn from_affinity(
        affinity: &super::super::application_attempt::WorthQueryApplicationAttemptAffinity,
    ) -> Self {
        Self(affinity.lookup_identity())
    }

    const fn from_identity(identity: WorthQueryProviderSessionAffinityIdentity) -> Self {
        Self(identity)
    }
}

#[derive(Default)]
pub(super) struct WorthQueryPrimaryGraphApplicationAttemptStore {
    attempts: BTreeMap<WorthQueryApplicationAttemptLookupKey, WorthQueryApplicationAttemptEntry>,
    next_registration: u64,
    next_overlay: u64,
}

enum WorthQueryApplicationAttemptEntry {
    Reserved {
        identity: u64,
        terminal:
            crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    },
    Registered {
        identity: u64,
        state: WorthQueryApplicationAttemptState,
    },
    Committing {
        identity: u64,
    },
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationAttemptReservation {
    key: WorthQueryApplicationAttemptLookupKey,
    identity: u64,
}

pub(super) struct WorthQueryStagedApplicationAttempt<'attempt> {
    attempt: &'attempt WorthQueryPrimaryGraphApplicationAttempt,
    overlay: &'attempt WorthQueryPrimaryGraphOverlay,
}

impl WorthQueryPrimaryGraphApplicationAttemptStore {
    pub(super) fn contains_observed_fact(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        locator: &str,
    ) -> bool {
        self.attempt(session)
            .is_some_and(|attempt| attempt.facts().contains_key(locator))
    }

    pub(super) fn observed_fact_and_product(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        locator: &str,
    ) -> Option<WorthQueryObservedApplicationFactBasis> {
        let attempt = self.attempt(session)?;
        Some(WorthQueryObservedApplicationFactBasis::new(
            attempt.facts().get(locator)?.clone(),
            attempt.affinity().product_publication().clone(),
        ))
    }

    pub(super) fn idempotency_basis(
        &self,
        session: &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    ) -> Option<WorthQueryApplicationIdempotencyBasis> {
        let key = WorthQueryApplicationAttemptLookupKey::from_identity(session.affinity_identity());
        let WorthQueryApplicationAttemptEntry::Registered { state, .. } =
            self.attempts.get(&key)?
        else {
            return None;
        };
        let attempt = state.attempt();
        if !attempt.affinity().same_session(session) {
            return None;
        }
        Some(WorthQueryApplicationIdempotencyBasis::new(
            attempt.idempotency(),
            attempt.affinity().product_publication().clone(),
        ))
    }

    pub(super) fn stage_overlay(
        &mut self,
        session: WorthQueryProviderSessionView<'_>,
        expected_steps: &[WorthQueryProvisionalEffectStep],
        generation: u64,
        facts: Vec<WorthQueryProposedFact>,
    ) -> Result<WorthQueryAdmittedApplicationOverlay, &'static str> {
        let accepted = self
            .attempt_state(session)
            .is_some_and(|state| state.accepts_overlay(expected_steps));
        if !accepted {
            return Err("provider session cannot stage this application overlay");
        }
        let next_overlay = self
            .next_overlay
            .checked_add(1)
            .ok_or("primary graph overlay identity space is exhausted")?;
        self.next_overlay = next_overlay;
        let identity = format!("primary-overlay:{generation}:{next_overlay}");
        let state = self
            .attempt_state_mut(session)
            .ok_or("provider session has no registered application attempt")?;
        state.stage_overlay(WorthQueryPrimaryGraphOverlay::new(
            identity.clone(),
            facts.clone(),
        ))?;
        Ok(WorthQueryAdmittedApplicationOverlay::new(identity, facts))
    }

    pub(super) fn staged_attempt(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Option<WorthQueryStagedApplicationAttempt<'_>> {
        self.attempt_state(session)?.staged()
    }

    pub(super) fn retain_invariant_approved(
        &mut self,
        session: WorthQueryProviderSessionView<'_>,
        candidate: worth_relational::facade::mvcc::ValidatedRelationalProposal,
        work: WorthQueryPrimaryMutationWorkCounters,
    ) -> Result<(), &'static str> {
        let state = self
            .attempt_state_mut(session)
            .ok_or("provider session has no registered application attempt")?;
        state.approve(candidate, work)
    }

    pub(super) fn is_staged_session_preparable(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> bool {
        self.attempt_state(session)
            .is_some_and(WorthQueryApplicationAttemptState::is_preparable)
    }

    pub(super) fn has_invariant_approved_candidate(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> bool {
        self.attempt_state(session)
            .is_some_and(WorthQueryApplicationAttemptState::phase_is_commit_ready)
    }

    fn take_commit_prepared(
        &mut self,
        session: WorthQueryProviderSessionView<'_>,
        attempts: Arc<Mutex<Self>>,
    ) -> Option<WorthQueryPreparedProviderApplicationAttempt> {
        let key = WorthQueryApplicationAttemptLookupKey::from_identity(session.affinity_identity());
        if !self
            .attempt_state(session)
            .is_some_and(|state| state.phase_is_commit_ready())
        {
            return None;
        }
        let WorthQueryApplicationAttemptEntry::Registered { identity, state } = self
            .attempts
            .remove(&key)
            .expect("commit-ready state exists")
        else {
            unreachable!("commit readiness excludes reservation entries")
        };
        let (attempt, candidate, work) = state
            .take_commit_parts()
            .expect("commit-ready state retains its prepared parts");
        self.attempts.insert(
            key,
            WorthQueryApplicationAttemptEntry::Committing { identity },
        );
        Some(WorthQueryPreparedProviderApplicationAttempt::new(
            attempt, candidate, work, attempts, key, identity,
        ))
    }

    pub(super) fn discard_overlay(
        &mut self,
        evidence: crate::domain_computation::provider_session::WorthQueryProvisionalOverlayEvidenceView<'_>,
    ) -> bool {
        let cleanup = evidence.cleanup_binding();
        let Some(WorthQueryApplicationAttemptEntry::Registered { state, .. }) = self
            .attempts
            .get_mut(&WorthQueryApplicationAttemptLookupKey::from_identity(
                cleanup.affinity_identity(),
            ))
        else {
            return false;
        };
        state.discard_overlay(cleanup, evidence.physical_overlay_identity())
    }

    pub(super) fn abort(&mut self, session: WorthQueryProviderSessionView<'_>) {
        let key = WorthQueryApplicationAttemptLookupKey::from_identity(session.affinity_identity());
        let admitted = self.attempts.get(&key).is_some_and(|entry| match entry {
            WorthQueryApplicationAttemptEntry::Reserved { terminal, .. } => {
                terminal.admits_session_view(session)
            }
            WorthQueryApplicationAttemptEntry::Registered { state, .. } => {
                state.admits_session(session)
            }
            WorthQueryApplicationAttemptEntry::Committing { .. } => false,
        });
        if admitted {
            self.attempts.remove(&key);
        }
    }

    #[cfg(test)]
    pub(super) fn resource_count(&self) -> usize {
        self.attempts
            .values()
            .map(|entry| match entry {
                WorthQueryApplicationAttemptEntry::Reserved { .. } => 1,
                WorthQueryApplicationAttemptEntry::Registered { state, .. } => {
                    state.resource_count()
                }
                WorthQueryApplicationAttemptEntry::Committing { .. } => 1,
            })
            .sum()
    }

    #[cfg(feature = "test-world-operation-control")]
    pub(super) fn active_attempt_count(&self) -> usize {
        self.attempts.len()
    }

    fn attempt(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Option<&WorthQueryPrimaryGraphApplicationAttempt> {
        self.attempt_state(session)
            .map(WorthQueryApplicationAttemptState::attempt)
    }

    fn attempt_state(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Option<&WorthQueryApplicationAttemptState> {
        let WorthQueryApplicationAttemptEntry::Registered { state, .. } =
            self.attempts
                .get(&WorthQueryApplicationAttemptLookupKey::from_identity(
                    session.affinity_identity(),
                ))?
        else {
            return None;
        };
        state.admits_session(session).then_some(state)
    }

    fn attempt_state_mut(
        &mut self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Option<&mut WorthQueryApplicationAttemptState> {
        let WorthQueryApplicationAttemptEntry::Registered { state, .. } =
            self.attempts
                .get_mut(&WorthQueryApplicationAttemptLookupKey::from_identity(
                    session.affinity_identity(),
                ))?
        else {
            return None;
        };
        state.admits_session(session).then_some(state)
    }
}

impl WorthQueryStagedApplicationAttempt<'_> {
    pub(super) fn overlay_identity(&self) -> &str {
        self.overlay.identity()
    }

    pub(super) fn overlay_facts(&self) -> &[WorthQueryProposedFact] {
        self.overlay.facts()
    }

    pub(super) fn expected_step_count(&self) -> usize {
        self.attempt.expected_steps().len()
    }

    pub(super) fn batch(&self) -> &worth_relational::facade::transactions::WorkerIntentBatch {
        self.attempt.batch()
    }

    pub(super) fn branch(&self) -> &worth_relational::facade::history::BranchId {
        self.attempt.affinity().branch()
    }

    pub(super) fn product_publication(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding
    {
        self.attempt.affinity().product_publication()
    }

    pub(super) fn decision_fact_count(&self) -> usize {
        self.attempt.decision_fact_count()
    }

    pub(super) fn aftermath_causality(
        &self,
    ) -> Option<
        &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    > {
        self.attempt.aftermath_causality()
    }

    pub(super) fn application_graph_reads(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationGraphReadContract> {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .application_graph_reads()
    }

    pub(super) fn application_touches(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationTouchContract> {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .application_touches()
    }

    pub(super) fn application_read_touch_overlap(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationReadTouchOverlapIndex> {
        self.attempt
            .affinity()
            .provider_session()
            .plan()
            .application_read_touch_overlap()
    }
}

#[cfg(test)]
mod tests;
