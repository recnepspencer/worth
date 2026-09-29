//! Bounded owner custody for authenticated, exactly correlated occurrences.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use worth_query_installation::facade::InstalledInboundOccurrenceContract;
use worth_runtime_world::facade::CompositeCommitIdentity;

use super::super::ExternalEffectCorrelationIdentity;
use super::WorthQueryInboundOccurrenceClaims;
use super::WorthQueryInboundTerminalOwnerResult;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;

mod publication_permit;
mod recovery;
mod terminal_state;
pub(in crate::domain_computation) use publication_permit::{
    WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial,
};
pub(in crate::domain_computation) use recovery::WorthQueryInboundRecoveryState;

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
}

/// Immutable accepted evidence retained by Query even if every caller drops
/// its observer. Publication may borrow this but cannot change its meaning.
pub(in crate::domain_computation) struct WorthQueryAcceptedInboundOccurrence {
    operation: String,
    claims: WorthQueryInboundOccurrenceClaims,
    owner: WorthQueryCommittedDispatchOutboxObservation,
    signed_meaning_digest: [u8; 32],
}

impl WorthQueryAcceptedInboundOccurrence {
    #[cfg(test)]
    pub(in crate::domain_computation) fn seal_for_world_test(
        claims: WorthQueryInboundOccurrenceClaims,
        owner: WorthQueryCommittedDispatchOutboxObservation,
    ) -> Arc<Self> {
        Arc::new(Self {
            operation: "world-substrate-test".to_owned(),
            claims,
            owner,
            signed_meaning_digest: [0; 32],
        })
    }

    pub(in crate::domain_computation) fn claims(&self) -> &WorthQueryInboundOccurrenceClaims {
        &self.claims
    }

    pub(in crate::domain_computation) fn operation(&self) -> &str {
        &self.operation
    }

    pub(in crate::domain_computation) fn owner(
        &self,
    ) -> &WorthQueryCommittedDispatchOutboxObservation {
        &self.owner
    }

    pub(in crate::domain_computation) fn signed_meaning_digest(&self) -> &[u8; 32] {
        &self.signed_meaning_digest
    }
}

pub(in crate::domain_computation) enum WorthQueryInboundCustodyAdmission {
    New(Arc<WorthQueryAcceptedInboundOccurrence>),
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
pub(in crate::domain_computation) struct WorthQueryInboundCustody {
    by_message: BTreeMap<MessageKey, CustodyEntry>,
    compact_terminal_messages: BTreeMap<MessageKey, CompactTerminalMessage>,
    by_correlation: BTreeMap<ExternalEffectCorrelationIdentity, MessageKey>,
    usage_by_operation: BTreeMap<String, Usage>,
    terminal_expiry: BTreeMap<String, BTreeMap<u64, BTreeSet<MessageKey>>>,
    compact_terminal_expiry: BTreeMap<String, BTreeMap<u64, BTreeSet<MessageKey>>>,
    pending_by_operation: BTreeMap<String, BTreeSet<ExternalEffectCorrelationIdentity>>,
    maintenance_cursor_by_operation: BTreeMap<String, ExternalEffectCorrelationIdentity>,
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

mod cleanup;
mod maintenance;
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
pub(in crate::domain_computation) enum WorthQueryInboundPublicationClaim {
    Claimed,
    AtCapacity,
    Publishing,
    Unpublished,
    Terminal,
}

impl WorthQueryInboundCustody {
    pub(in crate::domain_computation) fn reserve_transport_publication(
        custody: Arc<Mutex<Self>>,
        operation: &str,
        contract: &InstalledInboundOccurrenceContract,
    ) -> Result<WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial>
    {
        publication_permit::reserve(custody, operation, contract)
    }

    /// Duplicates are checked before owner lookup so terminal replay can use
    /// compact retained meaning even after heavy Relational leases are freed.
    pub(in crate::domain_computation) fn duplicate(
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

    pub(in crate::domain_computation) fn accept(
        &mut self,
        operation: &str,
        contract: &InstalledInboundOccurrenceContract,
        claims: WorthQueryInboundOccurrenceClaims,
        envelope: &[u8],
        owner: WorthQueryCommittedDispatchOutboxObservation,
    ) -> WorthQueryInboundCustodyAdmission {
        if let Some(duplicate) = self.duplicate(operation, &claims, envelope) {
            return duplicate;
        }
        let correlation = *owner.record().correlation();
        if self.by_correlation.contains_key(&correlation) {
            return WorthQueryInboundCustodyAdmission::CorrelationAlreadyOwned;
        }
        let Some(bytes) = envelope
            .len()
            .checked_add(owner.record().payload().len())
            .and_then(|bytes| bytes.checked_add(256))
            .and_then(|bytes| u64::try_from(bytes).ok())
        else {
            return WorthQueryInboundCustodyAdmission::CapacityExhausted;
        };
        let usage = self
            .usage_by_operation
            .entry(operation.to_owned())
            .or_default();
        let limits = contract.limits();
        let Some(next_count) = usage.count.checked_add(1) else {
            return WorthQueryInboundCustodyAdmission::CapacityExhausted;
        };
        let Some(next_bytes) = usage.bytes.checked_add(bytes) else {
            return WorthQueryInboundCustodyAdmission::CapacityExhausted;
        };
        if next_count > limits.maximum_accepted_occurrences.get()
            || next_bytes > limits.maximum_accepted_bytes.get()
            || usage.publishing >= limits.maximum_concurrent_publications.get()
        {
            return WorthQueryInboundCustodyAdmission::CapacityExhausted;
        }
        let message = MessageKey::from_claims(&claims);
        let signed_meaning_digest = claims.signed_meaning_digest;
        let accepted = Arc::new(WorthQueryAcceptedInboundOccurrence {
            operation: operation.to_owned(),
            claims,
            owner,
            signed_meaning_digest,
        });
        usage.count = next_count;
        usage.bytes = next_bytes;
        usage.publishing += 1;
        self.by_correlation.insert(correlation, message.clone());
        self.by_message.insert(
            message,
            CustodyEntry {
                accepted: Arc::clone(&accepted),
                charged_bytes: bytes,
                terminal: None,
                terminal_release_pending: false,
                unpublished: None,
                publication: PublicationState::Publishing,
            },
        );
        WorthQueryInboundCustodyAdmission::New(accepted)
    }

    pub(in crate::domain_computation) fn terminal_for(
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

    pub(in crate::domain_computation) fn terminal_by_correlation(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Option<Arc<WorthQueryInboundTerminalOwnerResult>> {
        self.by_message
            .get(self.by_correlation.get(correlation)?)?
            .terminal
            .as_ref()
            .map(Arc::clone)
    }

    pub(in crate::domain_computation) fn accepted_by_correlation(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Option<Arc<WorthQueryAcceptedInboundOccurrence>> {
        self.by_message
            .get(self.by_correlation.get(correlation)?)
            .map(|entry| Arc::clone(&entry.accepted))
    }

    /// Store terminal proof in the same finite custody slot before the heavy
    /// original dispatch lease may be released.
    pub(in crate::domain_computation) fn retain_terminal(
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

    pub(in crate::domain_computation) fn claim_retryable_publication(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
    ) -> WorthQueryInboundPublicationClaim {
        let limits = accepted
            .owner()
            .record()
            .inbound()
            .expect("accepted occurrence retains installed inbound binding")
            .limits();
        let usage = self
            .usage_by_operation
            .get_mut(accepted.operation())
            .expect("accepted operation counted");
        let entry = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
            .expect("accepted occurrence remains in owner custody");
        assert!(Arc::ptr_eq(&entry.accepted, accepted));
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

    pub(in crate::domain_computation) fn mark_publication_retryable(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
    ) {
        let entry = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
            .expect("accepted occurrence remains in owner custody");
        assert!(Arc::ptr_eq(&entry.accepted, accepted));
        assert_eq!(entry.publication, PublicationState::Publishing);
        entry.publication = PublicationState::Retryable;
        self.pending_by_operation
            .entry(accepted.operation().to_owned())
            .or_default()
            .insert(*accepted.owner().record().correlation());
        self.usage_by_operation
            .get_mut(accepted.operation())
            .expect("accepted operation counted")
            .publishing -= 1;
    }
}
