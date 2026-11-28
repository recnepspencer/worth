//! Affinity-keyed transient state for one Primary Graph application attempt.

use crate::domain_computation::{
    WorthQueryProviderSessionAffinityIdentity, WorthQueryProviderSessionView,
    WorthQueryProvisionalEffectStep,
};
use std::{collections::BTreeMap, sync::Arc};

pub mod candidate_read_support;
pub mod candidate_support_store;
pub use candidate_read_support::{
    CandidateReadSupport, ParentRead, RetainedCandidateRead, RetainedReadRequalification,
};
pub mod applied_output;
pub use applied_output::AppliedOutputParticipants;
pub mod commit_completion;
pub mod consumed_settlement;
pub mod coupled_extraction;
pub mod coupled_preparation;
pub mod coupled_settlement;
pub mod successor_independence;
pub mod writer_qualification;
pub(in crate::domain_computation::primary_graph::provider) use coupled_preparation::rank_consumption;
pub use coupled_preparation::{
    AppliedParticipantOutputs, CoupledApplicationMember, CoupledApplicationMemberKind,
    CoupledAttemptPreparation, CoupledProducerPreparation, SupersededRound,
};
pub mod commit_preparation;
pub mod coupled_publication;
pub use commit_preparation::publish_recovered;
pub use commit_preparation::ManagedUnpublishedAttempt;
pub mod phase;
pub(super) use phase::{
    WorthQueryApprovedApplicationCandidate, WorthQueryRetiredApplicationAttemptPhase,
};
pub mod registered_reads;
pub mod registration;
pub mod retained_basis;
pub use registered_reads::RegisteredPrimaryRead;
pub mod owned_members;
pub mod staged;
use commit_completion::WorthQueryPreparedProviderApplicationAttempt;
pub use commit_preparation::commit_prepared_application;
pub use commit_preparation::RebaseVerificationFailure;
pub use commit_preparation::{FactlessCurrentness, OwnEffectOnReads, RebaseVerificationReason};
pub use commit_preparation::{
    WorthQueryMutationWorkCommitSeal, WorthQueryPreImageRetentionWork,
    WorthQueryPrimaryGraphCommittedApplication,
};
pub use commit_preparation::{
    WorthQueryRetainedPreImageSeal, WorthQueryRetainedRelationPreImageSeal,
};
use phase::WorthQueryApplicationAttemptState;
pub(super) use retained_basis::{
    WorthQueryApplicationIdempotencyBasis, WorthQueryObservedApplicationFactBasis,
};

use super::WorthQueryPrimaryGraphApplicationAttempt;

pub mod lookup_key;
mod shared_store;
use lookup_key::WorthQueryApplicationAttemptLookupKey;
pub use shared_store::SharedApplicationAttemptStore;

#[derive(Default)]
pub struct WorthQueryPrimaryGraphApplicationAttemptStore {
    attempts: BTreeMap<WorthQueryApplicationAttemptLookupKey, WorthQueryApplicationAttemptEntry>,
    next_registration: u64,
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

pub struct WorthQueryApplicationAttemptReservation {
    key: WorthQueryApplicationAttemptLookupKey,
    identity: u64,
}

pub(super) struct WorthQueryStagedApplicationAttempt<'attempt> {
    attempt: &'attempt WorthQueryPrimaryGraphApplicationAttempt,
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

    pub(super) fn stage_candidate(
        &mut self,
        session: WorthQueryProviderSessionView<'_>,
        steps: &[WorthQueryProvisionalEffectStep],
    ) -> Result<(), &'static str> {
        self.attempt_state_mut(session)
            .ok_or("provider session has no registered application attempt")?
            .stage(steps)
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
        work: super::mutation_work::WorthQueryPrimaryMutationWorkCounters,
    ) -> Result<(), &'static str> {
        self.attempt_state_mut(session)
            .ok_or("coupled writer is not registered")?
            .approve_contribution(work)
    }
    pub(super) fn is_staged_session_preparable(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> bool {
        self.attempt_state(session)
            .is_some_and(WorthQueryApplicationAttemptState::is_preparable)
    }

    pub(super) fn retire_candidate(
        &mut self,
        binding: &crate::domain_computation::WorthQueryProviderSessionTerminalBinding,
    ) -> Option<WorthQueryRetiredApplicationAttemptPhase> {
        let Some(WorthQueryApplicationAttemptEntry::Registered { state, .. }) = self
            .attempts
            .get_mut(&WorthQueryApplicationAttemptLookupKey::from_identity(
                binding.affinity_identity(),
            ))
        else {
            return None;
        };
        state.retire_candidate(binding)
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

pub use commit_preparation::{PreparedRebaseDenial, PreparedSourceFactRebase};

pub use commit_preparation::QueryPreparingApplicationCommit;
pub use commit_preparation::RetainedQueryDurablePreparation;
pub mod court_doors_for_test;
#[cfg(test)]
#[path = "application_attempt_state_inline_type_budget.rs"]
pub mod inline_type_budget;
