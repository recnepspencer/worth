//! Bounded canonical terminal history and a disposable correlation index.
//!
//! A row alone does not establish terminal truth. Only a consumed World
//! `Performed` carrier can enter this history, and it is paired with the
//! Relational completion settlement before any lookup reports completion.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

mod canonical;
mod provider_api;
mod source;
pub(in crate::domain_computation::primary_graph) use source::InboundTerminalSource;
mod rebuild;
mod transport;

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, CompositePublicationAttemptIdentity, ProductBranchIncarnation,
    RuntimeWorldPerformedPublicationProtection,
};

use super::inbound_completion::WorthQueryCanonicalCompletionRow;
use super::inbound_completion::WorthQueryCompletionProvenance;
use crate::domain_computation::application_aftermath::{
    ExternalEffectCorrelationIdentity, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundTerminalOwnerResult,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCanonicalInboundCompletion {
    correlation: ExternalEffectCorrelationIdentity,
    operation: String,
    audience: String,
    source: String,
    key_epoch: Option<u64>,
    message_identity: Option<[u8; 32]>,
    signed_meaning_digest: Option<[u8; 32]>,
    provenance: WorthQueryCompletionProvenance,
    correlation_family: String,
    protocol_identity: BoundaryProtocolIdentity,
    protocol_version: BoundaryProtocolVersion,
    payload: Vec<u8>,
    expires_at_unix_seconds: Option<u64>,
    original_incarnation: ProductBranchIncarnation,
    original_relational_commit: RelationalCommitReceipt,
    original_world_commit: CompositeCommitIdentity,
    completion_relational_commit: RelationalCommitReceipt,
    completion_world_commit: CompositeCommitIdentity,
    completion_attempt: CompositePublicationAttemptIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundTerminalIndexDenial {
    SourceRead(super::WorthQueryCommittedDispatchOutboxReadDenial),
    ReadRequest(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    IndexUnavailable,
    ReconstructionWorkExhausted,
    WorldPairMismatch,
    NotExpired,
}

#[derive(Default)]
pub(super) struct WorthQueryInboundTerminalIndex {
    state: Mutex<TerminalIndexState>,
}

struct TerminalIndexState {
    derived:
        Option<BTreeMap<ExternalEffectCorrelationIdentity, WorthQueryCanonicalInboundCompletion>>,
    protected: BTreeMap<ExternalEffectCorrelationIdentity, std::sync::Arc<InboundTerminalSource>>,
    pending: BTreeSet<ExternalEffectCorrelationIdentity>,
    generation: u64,
    rebuilding: Option<TerminalRebuildState>,
}

struct TerminalRebuildState {
    generation: u64,
    cursor: Option<ExternalEffectCorrelationIdentity>,
    entries: BTreeMap<ExternalEffectCorrelationIdentity, WorthQueryCanonicalInboundCompletion>,
}

impl Default for TerminalIndexState {
    fn default() -> Self {
        Self {
            derived: Some(BTreeMap::new()),
            protected: BTreeMap::new(),
            pending: BTreeSet::new(),
            generation: 0,
            rebuilding: None,
        }
    }
}

impl WorthQueryInboundTerminalIndex {
    fn retain(
        &self,
        terminal: &WorthQueryInboundTerminalOwnerResult,
        protection: RuntimeWorldPerformedPublicationProtection,
    ) -> Result<(), WorthQueryInboundTerminalIndexDenial> {
        use WorthQueryInboundTerminalIndexDenial as Denial;
        let sealed = WorthQueryCanonicalInboundCompletion::seal(terminal)
            .ok_or(Denial::WorldPairMismatch)?;
        self.retain_sealed(sealed, protection)
    }

    fn retain_sealed(
        &self,
        sealed: WorthQueryCanonicalInboundCompletion,
        protection: RuntimeWorldPerformedPublicationProtection,
    ) -> Result<(), WorthQueryInboundTerminalIndexDenial> {
        use WorthQueryInboundTerminalIndexDenial as Denial;
        if protection.commit_identity() != sealed.completion_world_commit() {
            return Err(Denial::WorldPairMismatch);
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(existing) = state.protected.get(sealed.correlation()) {
            return if existing.commit_identity() == sealed.completion_world_commit() {
                Ok(())
            } else {
                Err(Denial::WorldPairMismatch)
            };
        }
        if let Some(derived) = &mut state.derived {
            derived.insert(sealed.correlation, sealed.clone());
        }
        state.protected.insert(
            sealed.correlation,
            std::sync::Arc::new(InboundTerminalSource::Performed {
                protection,
                incarnation: sealed.original_incarnation,
            }),
        );
        state.pending.remove(sealed.correlation());
        state.generation = state.generation.wrapping_add(1);
        state.rebuilding = None;
        Ok(())
    }

    fn lookup(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Result<Option<WorthQueryCanonicalInboundCompletion>, WorthQueryInboundTerminalIndexDenial>
    {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.pending.contains(correlation) {
            return Err(WorthQueryInboundTerminalIndexDenial::IndexUnavailable);
        }
        let derived = state
            .derived
            .as_ref()
            .ok_or(WorthQueryInboundTerminalIndexDenial::IndexUnavailable)?;
        Ok(derived.get(correlation).cloned())
    }

    fn mark_pending(&self, correlation: ExternalEffectCorrelationIdentity) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.pending.insert(correlation) {
            state.generation = state.generation.wrapping_add(1);
            state.rebuilding = None;
        }
    }

    fn clear_pending(&self, correlation: &ExternalEffectCorrelationIdentity) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.pending.remove(correlation) {
            state.generation = state.generation.wrapping_add(1);
            state.rebuilding = None;
        }
    }

    fn verify_expired_for_cleanup(
        &self,
        correlations: &[ExternalEffectCorrelationIdentity],
        now: u64,
    ) -> Result<(), WorthQueryInboundTerminalIndexDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let derived = state
            .derived
            .as_ref()
            .ok_or(WorthQueryInboundTerminalIndexDenial::IndexUnavailable)?;
        for correlation in correlations {
            if derived.get(correlation).is_some_and(|entry| {
                entry
                    .expires_at_unix_seconds
                    .is_none_or(|expiry| expiry >= now)
            }) {
                return Err(WorthQueryInboundTerminalIndexDenial::NotExpired);
            }
            let entry = derived
                .get(correlation)
                .ok_or(WorthQueryInboundTerminalIndexDenial::IndexUnavailable)?;
            if state
                .protected
                .get(correlation)
                .is_none_or(|lease| lease.commit_identity() != entry.completion_world_commit())
            {
                return Err(WorthQueryInboundTerminalIndexDenial::WorldPairMismatch);
            }
        }
        Ok(())
    }

    fn destroy_derived(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.derived = None;
        state.rebuilding = None;
        state.generation = state.generation.wrapping_add(1);
    }
}

mod court_doors_for_test;
