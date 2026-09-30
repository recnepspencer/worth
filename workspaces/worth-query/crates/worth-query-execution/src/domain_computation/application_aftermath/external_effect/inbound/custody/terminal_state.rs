//! Terminal custody releases accepted payload capacity while preserving owner truth.

use std::sync::Arc;
use worth_runtime_world::facade::CompositeCommitIdentity;

use super::{
    CompactTerminalMessage, MessageKey, PublicationState, WorthQueryAcceptedInboundOccurrence,
    WorthQueryInboundCustody,
};

impl WorthQueryInboundCustody {
    /// A separate exact transport terminal won while this signed occurrence
    /// waited for the incarnation lane. Keep compact signed meaning until its
    /// cutoff while dropping the heavy accepted payload and publication claim.
    pub(in crate::domain_computation) fn release_accepted_after_external_terminal(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        completion_world: CompositeCommitIdentity,
    ) {
        let message = MessageKey::from_claims(accepted.claims());
        let entry = self
            .by_message
            .get(&message)
            .expect("accepted occurrence remains in owner custody");
        assert!(Arc::ptr_eq(&entry.accepted, accepted));
        assert_eq!(entry.publication, PublicationState::Publishing);
        assert!(entry.terminal.is_none() && entry.unpublished.is_none());
        let correlation = *accepted.owner().record().correlation();
        assert_eq!(self.by_correlation.get(&correlation), Some(&message));
        self.remove_pending(accepted);
        let entry = self
            .by_message
            .remove(&message)
            .expect("accepted message was indexed");
        let operation = accepted.operation().to_owned();
        let expiry = accepted.claims().expires_at_unix_seconds;
        assert!(self
            .compact_terminal_messages
            .insert(
                message.clone(),
                CompactTerminalMessage {
                    operation: operation.clone(),
                    signed_meaning_digest: *accepted.signed_meaning_digest(),
                    correlation,
                    expiry,
                    charged_bytes: entry.charged_bytes,
                    original_world: accepted
                        .owner()
                        .committed_product_publication()
                        .composite_commit()
                        .clone(),
                    completion_world,
                }
            )
            .is_none());
        self.compact_terminal_expiry
            .entry(operation)
            .or_default()
            .entry(expiry)
            .or_default()
            .insert(message);
        let usage = self
            .usage_by_operation
            .get_mut(accepted.operation())
            .expect("accepted operation was charged");
        usage.publishing -= 1;
    }

    pub(in crate::domain_computation) fn mark_terminal_release_complete(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
    ) {
        let entry = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
            .expect("terminal owner remains in custody");
        assert!(Arc::ptr_eq(&entry.accepted, accepted) && entry.terminal.is_some());
        entry.terminal_release_pending = false;
        if entry
            .terminal
            .as_ref()
            .is_some_and(|terminal| terminal.delivery_settled())
        {
            self.remove_pending(accepted);
            self.terminal_expiry
                .entry(accepted.operation().to_owned())
                .or_default()
                .entry(accepted.claims().expires_at_unix_seconds)
                .or_default()
                .insert(MessageKey::from_claims(accepted.claims()));
        }
    }

    pub(super) fn remove_pending(&mut self, accepted: &Arc<WorthQueryAcceptedInboundOccurrence>) {
        if let Some(pending) = self.pending_by_operation.get_mut(accepted.operation()) {
            pending.remove(accepted.owner().record().correlation());
            if pending.is_empty() {
                self.pending_by_operation.remove(accepted.operation());
                self.maintenance_cursor_by_operation
                    .remove(accepted.operation());
            }
        }
    }
}
