//! Finite admission and publication slot reservation for signed occurrences.

use std::sync::Arc;

use worth_query_installation::facade::InstalledInboundOccurrenceContract;

use super::{
    CustodyEntry, MessageKey, PublicationState, WorthQueryAcceptedInboundOccurrence,
    WorthQueryInboundCustody, WorthQueryInboundCustodyAdmission,
};
use crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceClaims;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;

impl WorthQueryInboundCustody {
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
        {
            return WorthQueryInboundCustodyAdmission::CapacityExhausted;
        }
        let publish_now = usage.publishing < limits.maximum_concurrent_publications.get();
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
        if publish_now {
            usage.publishing += 1;
        } else {
            self.pending_by_operation
                .entry(operation.to_owned())
                .or_default()
                .insert(correlation);
        }
        self.by_correlation.insert(correlation, message.clone());
        self.by_message.insert(
            message,
            CustodyEntry {
                accepted: Arc::clone(&accepted),
                charged_bytes: bytes,
                terminal: None,
                terminal_release_pending: false,
                unpublished: None,
                publication: if publish_now {
                    PublicationState::Publishing
                } else {
                    PublicationState::Retryable
                },
            },
        );
        WorthQueryInboundCustodyAdmission::New(accepted, publish_now)
    }
}
