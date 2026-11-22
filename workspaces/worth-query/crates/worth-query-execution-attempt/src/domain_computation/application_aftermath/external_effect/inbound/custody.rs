//! Bounded owner custody for authenticated, exactly correlated occurrences.

pub mod recovery;
use super::super::ExternalEffectCorrelationIdentity;
use super::WorthQueryInboundOccurrenceClaims;
use super::WorthQueryInboundTerminalOwnerResult;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;
pub use recovery::WorthQueryInboundRecoveryState;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use worth_query_installation::facade::InstalledInboundOccurrenceContract;
use worth_runtime_world::facade::CompositeCommitIdentity;
pub mod acceptance;
pub mod publication_permit;
pub mod recorded_terminal;
pub mod retirement;
pub mod terminal_state;
pub use publication_permit::{
    WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct MessageKey {
    audience: String,
    protocol: String,
    source: String,
    key_epoch: u64,
    message_identity: [u8; 32],
}

impl MessageKey {
    fn from_claims(claims: &WorthQueryInboundOccurrenceClaims) -> Self {
        Self {
            audience: claims.audience.clone(),
            protocol: claims.protocol_identity.as_str().to_owned(),
            source: claims.source_identity.clone(),
            key_epoch: claims.key_epoch,
            message_identity: claims.message_identity,
        }
    }

    fn recorded(
        &self,
    ) -> crate::domain_computation::application_aftermath::dispatch_journal::QueryInboundMessageKey
    {
        crate::domain_computation::application_aftermath::dispatch_journal::QueryInboundMessageKey {
            audience: self.audience.clone(),
            protocol: self.protocol.clone(),
            source: self.source.clone(),
            epoch: self.key_epoch,
            message: self.message_identity,
        }
    }
    fn from_recorded(
        key: &crate::domain_computation::application_aftermath::dispatch_journal::QueryInboundMessageKey,
    ) -> Self {
        Self {
            audience: key.audience.clone(),
            protocol: key.protocol.clone(),
            source: key.source.clone(),
            key_epoch: key.epoch,
            message_identity: key.message,
        }
    }
}

pub mod accepted;
pub use accepted::WorthQueryAcceptedInboundOccurrence;

pub enum WorthQueryInboundCustodyAdmission {
    New(Arc<WorthQueryAcceptedInboundOccurrence>, bool),
    Duplicate(Arc<WorthQueryAcceptedInboundOccurrence>),
    CompactTerminalReplay(WorthQueryInboundOccurrenceClaims),
    MessageIdentityConflict,
    CorrelationAlreadyOwned,
    CapacityExhausted,
}

#[derive(Default)]
struct Usage {
    count: u64,
    bytes: u64,
    publishing: u64,
}

#[derive(Default)]
pub struct WorthQueryInboundCustody {
    pub durable_admission_cursor: u64,
    by_message: BTreeMap<MessageKey, CustodyEntry>,
    compact_terminal_messages: BTreeMap<MessageKey, CompactTerminalMessage>,
    by_correlation: BTreeMap<ExternalEffectCorrelationIdentity, MessageKey>,
    usage_by_operation: BTreeMap<String, Usage>,
    terminal_expiry: BTreeMap<String, BTreeMap<u64, BTreeSet<MessageKey>>>,
    compact_terminal_expiry: BTreeMap<String, BTreeMap<u64, BTreeSet<MessageKey>>>,
    pending_by_operation: BTreeMap<String, BTreeSet<ExternalEffectCorrelationIdentity>>,
    maintenance_cursor_by_operation: BTreeMap<String, ExternalEffectCorrelationIdentity>,
    retired_message_terminals: Vec<Arc<WorthQueryInboundTerminalOwnerResult>>,
    retired_terminal_receipts: Vec<Arc<WorthQueryInboundTerminalOwnerResult>>,
    retired_entries: Vec<CustodyEntry>,
}

struct CompactTerminalMessage {
    operation: String,
    signed_meaning_digest: [u8; 32],
    correlation: ExternalEffectCorrelationIdentity,
    expiry: u64,
    charged_bytes: u64,
    original_world: CompositeCommitIdentity,
    completion_world: CompositeCommitIdentity,
}

pub mod cleanup;
pub mod maintenance;
pub use cleanup::WorthQueryInboundCleanupReport;

struct CustodyEntry {
    accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
    charged_bytes: u64,
    terminal: Option<Arc<WorthQueryInboundTerminalOwnerResult>>,
    terminal_release_pending: bool,
    unpublished: Option<WorthQueryInboundRecoveryState>,
    publication: PublicationState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PublicationState {
    Publishing,
    Recovering,
    Retryable,
    Unpublished,
    Terminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundPublicationClaim {
    Claimed,
    AtCapacity,
    Publishing,
    Unpublished,
    Terminal,
    Gone,
}

impl WorthQueryInboundCustody {
    pub fn usage_for_operation(&self, operation: &str) -> (u64, u64) {
        self.usage_by_operation
            .get(operation)
            .map_or((0, 0), |usage| (usage.count, usage.bytes))
    }
    pub fn reserve_transport_publication(
        custody: Arc<Mutex<Self>>,
        operation: &str,
        contract: &InstalledInboundOccurrenceContract,
    ) -> Result<WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial>
    {
        publication_permit::reserve(custody, operation, contract)
    }

    /// Duplicates are checked before owner lookup so terminal replay can use
    /// compact retained meaning even after heavy Relational leases are freed.
    pub fn duplicate(
        &self,
        operation: &str,
        claims: &WorthQueryInboundOccurrenceClaims,
        _envelope: &[u8],
    ) -> Option<WorthQueryInboundCustodyAdmission> {
        if let Some(compact) = self
            .compact_terminal_messages
            .get(&MessageKey::from_claims(claims))
        {
            return Some(
                if compact.operation == operation
                    && compact.signed_meaning_digest == claims.signed_meaning_digest
                {
                    WorthQueryInboundCustodyAdmission::CompactTerminalReplay(claims.clone())
                } else {
                    WorthQueryInboundCustodyAdmission::MessageIdentityConflict
                },
            );
        }
        let accepted = Arc::clone(
            &self
                .by_message
                .get(&MessageKey::from_claims(claims))?
                .accepted,
        );
        Some(
            if accepted.operation == operation
                && accepted.signed_meaning_digest == claims.signed_meaning_digest
            {
                WorthQueryInboundCustodyAdmission::Duplicate(accepted)
            } else {
                WorthQueryInboundCustodyAdmission::MessageIdentityConflict
            },
        )
    }

    pub fn terminal_for(
        &self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
    ) -> Option<(Arc<WorthQueryInboundTerminalOwnerResult>, bool)> {
        let entry = self
            .by_message
            .get(&MessageKey::from_claims(accepted.claims()))?;
        Arc::ptr_eq(&entry.accepted, accepted)
            .then(|| {
                entry
                    .terminal
                    .as_ref()
                    .map(|terminal| (Arc::clone(terminal), entry.terminal_release_pending))
            })
            .flatten()
    }

    pub fn retains_unpublished(&self, accepted: &Arc<WorthQueryAcceptedInboundOccurrence>) -> bool {
        self.by_message
            .get(&MessageKey::from_claims(accepted.claims()))
            .is_some_and(|entry| {
                Arc::ptr_eq(&entry.accepted, accepted) && entry.unpublished.is_some()
            })
    }

    pub fn terminal_by_correlation(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Option<Arc<WorthQueryInboundTerminalOwnerResult>> {
        self.by_message
            .get(self.by_correlation.get(correlation)?)?
            .terminal
            .as_ref()
            .map(Arc::clone)
    }

    pub fn accepted_by_correlation(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Option<Arc<WorthQueryAcceptedInboundOccurrence>> {
        self.by_message
            .get(self.by_correlation.get(correlation)?)
            .map(|entry| Arc::clone(&entry.accepted))
    }

    /// Store terminal proof in the same finite custody slot before the heavy
    /// original dispatch lease may be released.
    pub fn retain_terminal(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        terminal: Arc<WorthQueryInboundTerminalOwnerResult>,
    ) -> bool {
        let Some(entry) = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
        else {
            return false;
        };
        if !Arc::ptr_eq(&entry.accepted, accepted) || entry.terminal.is_some() {
            return false;
        }
        entry.terminal = Some(terminal);
        entry.terminal_release_pending = true;
        entry.publication = PublicationState::Terminal;
        self.pending_by_operation
            .entry(accepted.operation().to_owned())
            .or_default()
            .insert(*accepted.owner().record().correlation());
        self.usage_by_operation
            .get_mut(accepted.operation())
            .expect("accepted operation counted")
            .publishing -= 1;
        true
    }

    pub fn claim_retryable_publication(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
    ) -> WorthQueryInboundPublicationClaim {
        let Some(entry) = self
            .by_message
            .get(&MessageKey::from_claims(accepted.claims()))
        else {
            return WorthQueryInboundPublicationClaim::Gone;
        };
        if !Arc::ptr_eq(&entry.accepted, accepted) {
            return WorthQueryInboundPublicationClaim::Gone;
        }
        let limits = accepted
            .owner()
            .record()
            .inbound()
            .map(InstalledInboundOccurrenceContract::limits);
        let Some(limits) = limits else {
            return WorthQueryInboundPublicationClaim::Gone;
        };
        let Some(usage) = self.usage_by_operation.get_mut(accepted.operation()) else {
            return WorthQueryInboundPublicationClaim::Gone;
        };
        let entry = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
            .expect("entry checked under this custody lock");
        match entry.publication {
            PublicationState::Retryable => {
                if usage.publishing >= limits.maximum_concurrent_publications.get() {
                    return WorthQueryInboundPublicationClaim::AtCapacity;
                }
                usage.publishing += 1;
                entry.publication = PublicationState::Publishing;
                self.remove_pending(accepted);
                WorthQueryInboundPublicationClaim::Claimed
            }
            PublicationState::Publishing => WorthQueryInboundPublicationClaim::Publishing,
            PublicationState::Recovering => WorthQueryInboundPublicationClaim::Publishing,
            PublicationState::Unpublished => WorthQueryInboundPublicationClaim::Unpublished,
            PublicationState::Terminal => WorthQueryInboundPublicationClaim::Terminal,
        }
    }

    pub fn mark_publication_retryable(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
    ) -> bool {
        let Some(entry) = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
        else {
            return false;
        };
        if !Arc::ptr_eq(&entry.accepted, accepted)
            || entry.publication != PublicationState::Publishing
        {
            return false;
        }
        let Some(usage) = self.usage_by_operation.get_mut(accepted.operation()) else {
            return false;
        };
        if usage.publishing == 0 {
            return false;
        }
        entry.publication = PublicationState::Retryable;
        self.pending_by_operation
            .entry(accepted.operation().to_owned())
            .or_default()
            .insert(*accepted.owner().record().correlation());
        usage.publishing -= 1;
        true
    }
}
