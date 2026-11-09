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
mod coupled_extraction;
mod coupled_preparation;
mod coupled_settlement;
pub(in crate::domain_computation::primary_graph) use coupled_preparation::{
    CoupledApplicationMember, CoupledProducerPreparation,
};
pub(crate) mod commit_preparation;
pub(in crate::domain_computation::primary_graph::provider) mod coupled_publication;
pub(in crate::domain_computation::primary_graph::provider) use commit_preparation::publish_recovered;
pub(in crate::domain_computation::primary_graph::provider) use commit_preparation::ManagedUnpublishedAttempt;
mod phase;
pub(super) use phase::{
    WorthQueryApprovedApplicationCandidate, WorthQueryRetiredApplicationAttemptPhase,
};
mod registered_reads;
pub(crate) mod registration;
mod retained_basis;
pub(in crate::domain_computation) use registered_reads::RegisteredPrimaryRead;
mod staged;
use commit_completion::WorthQueryPreparedProviderApplicationAttempt;
pub(in crate::domain_computation::primary_graph::provider) use commit_preparation::commit_prepared_application;
pub(in crate::domain_computation::primary_graph) use commit_preparation::RebaseVerificationFailure;
pub use commit_preparation::{FactlessCurrentness, OwnEffectOnReads, RebaseVerificationReason};
pub(in crate::domain_computation::primary_graph) use commit_preparation::{
    WorthQueryMutationWorkCommitSeal, WorthQueryPreImageRetentionWork,
    WorthQueryPrimaryGraphCommittedApplication,
};
pub(crate) use commit_preparation::{
    WorthQueryRetainedPreImageSeal, WorthQueryRetainedRelationPreImageSeal,
};
use phase::{WorthQueryApplicationAttemptState, WorthQueryPrimaryGraphOverlay};
pub(super) use retained_basis::{
    WorthQueryAdmittedApplicationOverlay, WorthQueryApplicationIdempotencyBasis,
    WorthQueryObservedApplicationFactBasis,
};

use super::WorthQueryPrimaryGraphApplicationAttempt;

mod lookup_key;
use lookup_key::WorthQueryApplicationAttemptLookupKey;

#[derive(Default)]
pub struct WorthQueryPrimaryGraphApplicationAttemptStore {
    attempts: BTreeMap<WorthQueryApplicationAttemptLookupKey, WorthQueryApplicationAttemptEntry>,
    next_registration: u64,
    next_overlay: u64,
}
impl WorthQueryPrimaryGraphApplicationAttemptStore {
    pub(super) fn home_close_has_work(&self) -> bool {
        !self.attempts.is_empty()
    }
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
/// Keeps the real attempt alive until the caller leaves the attempt-store gate.
pub(super) struct WorthQueryRetiredApplicationAttempt {
    entry_custody: Option<WorthQueryApplicationAttemptEntry>,
}
impl Drop for WorthQueryRetiredApplicationAttempt {
    fn drop(&mut self) {
        drop(self.entry_custody.take());
    }
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
    pub(super) fn binds_original_read(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        read: &super::super::application_attempt::CompletedOriginalRead,
    ) -> bool {
        self.attempt(session)
            .is_some_and(|attempt| attempt.affinity().binds_original_read(read))
    }
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
            attempt.facts().get(locator)?,
            attempt.affinity().product_publication().clone(),
        ))
    }

    pub(super) fn shared_observed_facts(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Option<(
        Arc<crate::domain_computation::primary_graph::ApplicationFactTable>,
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    )>{
        let attempt = self.attempt(session)?;
        Some((
            attempt.shared_facts(),
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
        incoming: &mut Option<WorthQueryApprovedApplicationCandidate>,
    ) -> Result<(), &'static str> {
        let state = self
            .attempt_state_mut(session)
            .ok_or("provider session has no registered application attempt")?;
        state.approve(incoming)
    }

    pub(super) fn retain_coupled_invariant_approved(
        &mut self,
        session: WorthQueryProviderSessionView<'_>,
        evidence: worth_relational::facade::mvcc::RelationalMutationInvariantEvidence,
        work: super::mutation_work::WorthQueryPrimaryMutationWorkCounters,
    ) -> Result<(), &'static str> {
        self.attempt_state_mut(session)
            .ok_or("coupled writer is not registered")?
            .approve_contribution(phase::ApprovedCoupledContribution { evidence, work })
    }
    pub(super) fn approved_invariant_evidence(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Option<&worth_relational::facade::mvcc::RelationalMutationInvariantEvidence> {
        self.attempt_state(session)?.approved_evidence()
    }

    pub(super) fn is_staged_session_preparable(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> bool {
        self.attempt_state(session)
            .is_some_and(WorthQueryApplicationAttemptState::is_preparable)
    }

    pub(super) fn discard_overlay(
        &mut self,
        evidence: crate::domain_computation::provider_session::WorthQueryProvisionalOverlayEvidenceView<'_>,
    ) -> Option<WorthQueryRetiredApplicationAttemptPhase> {
        let cleanup = evidence.cleanup_binding();
        let Some(WorthQueryApplicationAttemptEntry::Registered { state, .. }) = self
            .attempts
            .get_mut(&WorthQueryApplicationAttemptLookupKey::from_identity(
                cleanup.affinity_identity(),
            ))
        else {
            return None;
        };
        state.discard_overlay(cleanup, evidence.physical_overlay_identity())
    }

    pub(super) fn abort(
        &mut self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Option<WorthQueryRetiredApplicationAttempt> {
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
        admitted
            .then(|| {
                self.attempts
                    .remove(&key)
                    .map(|entry| WorthQueryRetiredApplicationAttempt {
                        entry_custody: Some(entry),
                    })
            })
            .flatten()
    }

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

#[cfg(test)]
mod tests;

pub(in crate::domain_computation::primary_graph) use commit_preparation::{
    PreparedRebaseDenial, PreparedSourceFactRebase,
};

pub(in crate::domain_computation) use commit_preparation::QueryPreparingApplicationCommit;
pub(in crate::domain_computation::primary_graph::provider) use commit_preparation::RetainedQueryDurablePreparation;
mod court_doors_for_test;
#[cfg(test)]
#[path = "application_attempt_state_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
