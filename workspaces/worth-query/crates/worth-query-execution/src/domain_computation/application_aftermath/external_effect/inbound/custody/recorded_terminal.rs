//! Readmitted completed messages retain dedup meaning without fresh delivery.
use super::*;
use crate::domain_computation::primary_graph::provider::WorthQueryCanonicalInboundCompletion;

impl WorthQueryInboundCustody {
    pub(in crate::domain_computation) fn preflight_completed_message(
        &self,
        operation: &str,
        contract: &InstalledInboundOccurrenceContract,
        claims: &WorthQueryInboundOccurrenceClaims,
        envelope_bytes: u64,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<(), WorthQueryInboundCustodyAdmission> {
        self.admitted_message_capacity(operation, contract, claims, envelope_bytes, owner)
            .map(|_| ())
            .ok_or(WorthQueryInboundCustodyAdmission::CapacityExhausted)
    }
    pub(in crate::domain_computation) fn retain_recorded_terminal_message(
        &mut self,
        operation: &str,
        contract: &InstalledInboundOccurrenceContract,
        claims: &WorthQueryInboundOccurrenceClaims,
        envelope_bytes: u64,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
        terminal: &WorthQueryCanonicalInboundCompletion,
    ) -> Result<(), WorthQueryInboundCustodyAdmission> {
        if let Some(duplicate) = self.duplicate(operation, claims, &[]) {
            return match duplicate {
                WorthQueryInboundCustodyAdmission::CompactTerminalReplay(_) => Ok(()),
                other => Err(other),
            };
        }
        let Some((bytes, next_count, next_bytes)) =
            self.admitted_message_capacity(operation, contract, claims, envelope_bytes, owner)
        else {
            return Err(WorthQueryInboundCustodyAdmission::CapacityExhausted);
        };
        let message = MessageKey::from_claims(claims);
        self.compact_terminal_messages.insert(
            message.clone(),
            CompactTerminalMessage {
                operation: operation.to_owned(),
                signed_meaning_digest: claims.signed_meaning_digest,
                correlation: *owner.record().correlation(),
                expiry: claims.expires_at_unix_seconds,
                charged_bytes: bytes,
                original_world: terminal.original_world_commit().clone(),
                completion_world: terminal.completion_world_commit().clone(),
            },
        );
        self.compact_terminal_expiry
            .entry(operation.to_owned())
            .or_default()
            .entry(claims.expires_at_unix_seconds)
            .or_default()
            .insert(message);
        let usage = self
            .usage_by_operation
            .entry(operation.to_owned())
            .or_default();
        usage.count = next_count;
        usage.bytes = next_bytes;
        Ok(())
    }
}
