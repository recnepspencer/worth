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
        self.accept_retained(operation, contract, claims, envelope.len() as u64, owner)
    }

    pub(in crate::domain_computation) fn accept_retained(
        &mut self,
        operation: &str,
        contract: &InstalledInboundOccurrenceContract,
        claims: WorthQueryInboundOccurrenceClaims,
        envelope_bytes: u64,
        owner: WorthQueryCommittedDispatchOutboxObservation,
    ) -> WorthQueryInboundCustodyAdmission {
        if let Some(duplicate) = self.duplicate(operation, &claims, &[]) {
            return duplicate;
        }
        let correlation = *owner.record().correlation();
        if self.by_correlation.contains_key(&correlation) {
            return WorthQueryInboundCustodyAdmission::CorrelationAlreadyOwned;
        }
        let Some((bytes, next_count, next_bytes)) =
            self.admitted_message_capacity(operation, contract, &claims, envelope_bytes, &owner)
        else {
            return WorthQueryInboundCustodyAdmission::CapacityExhausted;
        };
        let usage = self
            .usage_by_operation
            .entry(operation.to_owned())
            .or_default();
        let limits = contract.limits();
        let publish_now = usage.publishing < limits.maximum_concurrent_publications.get();
        let message = MessageKey::from_claims(&claims);
        let signed_meaning_digest = claims.signed_meaning_digest;
        let accepted = Arc::new(WorthQueryAcceptedInboundOccurrence {
            operation: operation.to_owned(),
            claims,
            owner: if owner
                .committed_product_publication()
                .product_branch()
                .owner_identity()
                .durable()
                .is_some()
            {
                owner.into_descriptive()
            } else {
                owner
            },
            signed_meaning_digest,
            preparation_denial: std::sync::Mutex::new(None),
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

impl WorthQueryInboundCustody {
    pub(in crate::domain_computation) fn preflight_message(
        &self,
        operation: &str,
        contract: &InstalledInboundOccurrenceContract,
        claims: &WorthQueryInboundOccurrenceClaims,
        envelope_bytes: u64,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<(), WorthQueryInboundCustodyAdmission> {
        if let Some(existing) = self.duplicate(operation, claims, &[]) {
            return Err(existing);
        }
        if self
            .by_correlation
            .contains_key(owner.record().correlation())
        {
            return Err(WorthQueryInboundCustodyAdmission::CorrelationAlreadyOwned);
        }
        self.admitted_message_capacity(operation, contract, claims, envelope_bytes, owner)
            .map(|_| ())
            .ok_or(WorthQueryInboundCustodyAdmission::CapacityExhausted)
    }

    pub(super) fn admitted_message_capacity(
        &self,
        operation: &str,
        contract: &InstalledInboundOccurrenceContract,
        claims: &WorthQueryInboundOccurrenceClaims,
        envelope_bytes: u64,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Option<(u64, u64, u64)> {
        use crate::domain_computation::application_aftermath::dispatch_journal::QueryInboundAcceptedMessage;
        let bytes = if owner
            .committed_product_publication()
            .product_branch()
            .owner_identity()
            .durable()
            .is_some()
        {
            QueryInboundAcceptedMessage::retained_bytes(
                operation,
                claims,
                envelope_bytes,
                &owner.commit_reference().branch_id.0,
            )?
        } else {
            envelope_bytes
                .checked_add(owner.record().payload().len() as u64)?
                .checked_add(256)?
        };
        let (count, retained) = self.usage_for_operation(operation);
        let next_count = count.checked_add(1)?;
        let next_bytes = retained.checked_add(bytes)?;
        let limits = contract.limits();
        if next_count > limits.maximum_accepted_occurrences.get()
            || next_bytes > limits.maximum_accepted_bytes.get()
        {
            return None;
        }
        Some((bytes, next_count, next_bytes))
    }
}
