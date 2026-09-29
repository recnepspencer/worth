//! Bounded canonical terminal history and a disposable correlation index.
//!
//! A row alone does not establish terminal truth. Only a consumed World
//! `Performed` carrier can enter this history, and it is paired with the
//! Relational completion settlement before any lookup reports completion.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

mod provider_api;
mod rebuild;
mod transport;

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, CompositePublicationAttemptIdentity, ProductBranchIncarnation,
    RuntimeWorldPerformedPublicationProtection, RuntimeWorldPublicationCursor,
    RuntimeWorldPublicationFrontier,
};

use super::inbound_completion::WorthQueryCanonicalCompletionRow;
use super::inbound_completion::WorthQueryCompletionProvenance;
use crate::domain_computation::application_aftermath::{
    ExternalEffectCorrelationIdentity, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundTerminalOwnerResult,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryCanonicalInboundCompletion {
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

impl WorthQueryCanonicalInboundCompletion {
    /// A workflow may consume only the terminal for its original committed
    /// operation. Correlation selects this row; these owner facts authorize its
    /// use after both the dispatch and completion have been World-performed.
    pub(in crate::domain_computation::primary_graph) fn matches_original_dispatch(
        &self,
        record: &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord,
        original_relational_commit: &RelationalCommitReceipt,
        original_world_commit: &CompositeCommitIdentity,
        original_incarnation: ProductBranchIncarnation,
    ) -> bool {
        let Some(inbound) = record.inbound() else {
            return false;
        };
        self.correlation == *record.correlation()
            && record.operation_slot() == Some(self.operation.as_str())
            && record.effect() == inbound.effect()
            && self.source == inbound.source_identity()
            && self.correlation_family == record.correlation_family().as_str()
            && self.protocol_identity == *record.protocol_identity()
            && inbound.protocol().identity() == record.protocol_identity()
            && self.protocol_version == record.protocol_version()
            && inbound.protocol().version() == record.protocol_version()
            && self.payload == record.payload()
            && self.original_incarnation == original_incarnation
            && &self.original_relational_commit == original_relational_commit
            && &self.original_world_commit == original_world_commit
    }

    pub(in crate::domain_computation::primary_graph) fn from_verified_row(
        row: WorthQueryCanonicalCompletionRow,
        original_incarnation: ProductBranchIncarnation,
        original_world_commit: CompositeCommitIdentity,
        completion_world_commit: CompositeCommitIdentity,
        completion_attempt: CompositePublicationAttemptIdentity,
    ) -> Self {
        Self {
            correlation: row.correlation,
            operation: row.operation,
            audience: row.audience,
            source: row.source,
            key_epoch: row.key_epoch,
            message_identity: row.message_identity,
            signed_meaning_digest: row.signed_meaning_digest,
            provenance: row.provenance,
            correlation_family: row.family,
            protocol_identity: row.protocol_identity,
            protocol_version: row.protocol_version,
            payload: row.payload,
            expires_at_unix_seconds: row.expires_at,
            original_incarnation,
            original_relational_commit: row.original_commit,
            original_world_commit,
            completion_relational_commit: row.completion_commit,
            completion_world_commit,
            completion_attempt,
        }
    }
    fn seal(terminal: &WorthQueryInboundTerminalOwnerResult) -> Option<Self> {
        let accepted = terminal.accepted();
        let original = accepted.owner();
        original.record().inbound()?;
        let performed = terminal.publication().publication();
        let settlement = performed.component_results().relational_settlement()?;
        if performed
            .component_results()
            .relational_commit_result()
            .is_none()
            || performed.commit().identity() != performed.new_product_head().selected_commit()
            || performed.new_product_head().lifecycle_incarnation()
                != terminal.original_incarnation()
            || original
                .committed_product_publication()
                .product_incarnation()
                != terminal.original_incarnation()
            || original.committed_product_publication().relational_commit()
                != original.commit_reference()
        {
            return None;
        }
        let claims = accepted.claims();
        Some(Self {
            correlation: *original.record().correlation(),
            operation: accepted.operation().to_owned(),
            audience: claims.audience.clone(),
            source: claims.source_identity.clone(),
            key_epoch: Some(claims.key_epoch),
            message_identity: Some(claims.message_identity),
            signed_meaning_digest: Some(*accepted.signed_meaning_digest()),
            provenance: WorthQueryCompletionProvenance::AuthenticatedInbound,
            correlation_family: claims.correlation_family.clone(),
            protocol_identity: claims.protocol_identity.clone(),
            protocol_version: claims.protocol_version,
            payload: claims.payload.clone(),
            expires_at_unix_seconds: Some(claims.expires_at_unix_seconds),
            original_incarnation: terminal.original_incarnation(),
            original_relational_commit: original.commit_reference().clone(),
            original_world_commit: original
                .committed_product_publication()
                .composite_commit()
                .clone(),
            completion_relational_commit: settlement.clone(),
            completion_world_commit: performed.commit().identity().clone(),
            completion_attempt: performed.attempt_identity().clone(),
        })
    }

    pub(in crate::domain_computation) const fn correlation(
        &self,
    ) -> &ExternalEffectCorrelationIdentity {
        &self.correlation
    }

    pub(in crate::domain_computation) fn operation(&self) -> &str {
        &self.operation
    }

    pub(in crate::domain_computation) fn matches_effect(
        &self,
        operation: &str,
        claims: &WorthQueryInboundOccurrenceClaims,
    ) -> bool {
        self.operation == operation
            && self.audience == claims.audience
            && self.source == claims.source_identity
            && self.correlation_family == claims.correlation_family
            && self.protocol_identity == claims.protocol_identity
            && self.protocol_version == claims.protocol_version
            && self.payload == claims.payload
    }

    pub(in crate::domain_computation) fn matches_message(
        &self,
        claims: &WorthQueryInboundOccurrenceClaims,
    ) -> bool {
        self.message_identity == Some(claims.message_identity)
            && self.key_epoch == Some(claims.key_epoch)
            && self.signed_meaning_digest == Some(claims.signed_meaning_digest)
    }

    pub(in crate::domain_computation) const fn authenticated_message_identity(
        &self,
    ) -> Option<&[u8; 32]> {
        self.message_identity.as_ref()
    }

    pub(in crate::domain_computation) const fn original_world_commit(
        &self,
    ) -> &CompositeCommitIdentity {
        &self.original_world_commit
    }

    pub(in crate::domain_computation) const fn completion_world_commit(
        &self,
    ) -> &CompositeCommitIdentity {
        &self.completion_world_commit
    }

    pub(in crate::domain_computation) const fn completion_attempt(
        &self,
    ) -> &CompositePublicationAttemptIdentity {
        &self.completion_attempt
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryInboundTerminalIndexDenial {
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
    protected:
        BTreeMap<ExternalEffectCorrelationIdentity, RuntimeWorldPerformedPublicationProtection>,
    pending: BTreeSet<ExternalEffectCorrelationIdentity>,
    generation: u64,
    rebuilding: Option<TerminalRebuildState>,
}

struct TerminalRebuildState {
    generation: u64,
    frontier: Option<RuntimeWorldPublicationFrontier>,
    cursor: Option<RuntimeWorldPublicationCursor>,
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
        state.protected.insert(sealed.correlation, protection);
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

    fn prune_expired(
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

    #[cfg(test)]
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
