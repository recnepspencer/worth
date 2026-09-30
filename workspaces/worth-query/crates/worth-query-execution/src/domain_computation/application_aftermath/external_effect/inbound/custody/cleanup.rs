//! Explicit bounded reclamation after terminal owner handoff and signed expiry.

use super::WorthQueryInboundCustody;
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use worth_runtime_world::facade::CompositeCommitIdentity;

/// Exact accepted-custody count and bytes released by one bounded cleanup.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryInboundCleanupReport {
    reclaimed: u64,
    released_bytes: u64,
}

impl WorthQueryInboundCleanupReport {
    pub const fn reclaimed(&self) -> u64 {
        self.reclaimed
    }

    pub const fn released_bytes(&self) -> u64 {
        self.released_bytes
    }

    pub(in crate::domain_computation) fn combine(self, other: Self) -> Self {
        Self {
            reclaimed: self
                .reclaimed
                .checked_add(other.reclaimed)
                .expect("bounded cleanup count"),
            released_bytes: self
                .released_bytes
                .checked_add(other.released_bytes)
                .expect("bounded cleanup bytes"),
        }
    }
}

impl WorthQueryInboundCustody {
    /// Compact signed meaning is independently bounded by its admitted slot.
    /// The transport terminal has no signed expiry, so only this slot turns over.
    pub(in crate::domain_computation) fn eligible_expired_compact_terminals(
        &self,
        operation: &str,
        now_unix_seconds: u64,
        maximum_work: usize,
    ) -> Vec<(
        ExternalEffectCorrelationIdentity,
        CompositeCommitIdentity,
        CompositeCommitIdentity,
    )> {
        let mut selected = Vec::new();
        if maximum_work == 0 {
            return selected;
        }
        let Some(by_expiry) = self.compact_terminal_expiry.get(operation) else {
            return selected;
        };
        for (&expiry, messages) in by_expiry {
            if expiry >= now_unix_seconds {
                break;
            }
            for message in messages {
                let compact = self
                    .compact_terminal_messages
                    .get(message)
                    .expect("compact expiry index selects retained signed meaning");
                selected.push((
                    compact.correlation,
                    compact.original_world.clone(),
                    compact.completion_world.clone(),
                ));
                if selected.len() == maximum_work {
                    return selected;
                }
            }
        }
        selected
    }

    /// Reclaim only signed meaning already re-paired to canonical World truth.
    pub(in crate::domain_computation) fn cleanup_compact_exact(
        &mut self,
        operation: &str,
        now_unix_seconds: u64,
        selected: &[(
            ExternalEffectCorrelationIdentity,
            CompositeCommitIdentity,
            CompositeCommitIdentity,
        )],
    ) -> WorthQueryInboundCleanupReport {
        let mut report = WorthQueryInboundCleanupReport::default();
        for (correlation, original, completion) in selected {
            let Some(message) = self.by_correlation.get(correlation).cloned() else {
                continue;
            };
            let Some(compact) = self.compact_terminal_messages.get(&message) else {
                continue;
            };
            if compact.operation != operation
                || compact.expiry >= now_unix_seconds
                || &compact.original_world != original
                || &compact.completion_world != completion
            {
                continue;
            }
            let expiry = compact.expiry;
            let by_expiry = self
                .compact_terminal_expiry
                .get_mut(operation)
                .expect("compact operation remains indexed");
            let messages = by_expiry
                .get_mut(&expiry)
                .expect("compact cutoff remains indexed");
            assert!(messages.remove(&message));
            if messages.is_empty() {
                by_expiry.remove(&expiry);
            }
            if by_expiry.is_empty() {
                self.compact_terminal_expiry.remove(operation);
            }
            let compact = self
                .compact_terminal_messages
                .remove(&message)
                .expect("selected compact signed meaning remains retained");
            assert_eq!(self.by_correlation.remove(correlation), Some(message));
            let usage = self
                .usage_by_operation
                .get_mut(operation)
                .expect("compact signed meaning keeps admitted charge");
            usage.count = usage
                .count
                .checked_sub(1)
                .expect("compact count charged once");
            usage.bytes = usage
                .bytes
                .checked_sub(compact.charged_bytes)
                .expect("compact bytes charged once");
            report.reclaimed += 1;
            report.released_bytes += compact.charged_bytes;
        }
        report
    }

    /// The expiry index contains only World-performed terminals whose original
    /// dispatch lease has been handed to canonical history and whose delivery
    /// obligation is settled. Pending and unpublished work never enters it.
    pub(in crate::domain_computation) fn eligible_expired_terminal_correlations(
        &self,
        operation: &str,
        now_unix_seconds: u64,
        maximum_work: usize,
    ) -> Vec<ExternalEffectCorrelationIdentity> {
        let mut correlations = Vec::new();
        let Some(by_expiry) = self.terminal_expiry.get(operation) else {
            return correlations;
        };
        for (&expiry, messages) in by_expiry {
            if expiry >= now_unix_seconds {
                break;
            }
            for message in messages {
                let entry = self
                    .by_message
                    .get(message)
                    .expect("terminal expiry index selects retained custody");
                correlations.push(*entry.accepted.owner().record().correlation());
                if correlations.len() == maximum_work {
                    return correlations;
                }
            }
        }
        correlations
    }

    /// Remove exactly the terminals independently revalidated by the provider.
    /// A concurrent cleanup may have removed one already; that is harmless.
    pub(in crate::domain_computation) fn cleanup_completed_exact(
        &mut self,
        operation: &str,
        now_unix_seconds: u64,
        correlations: &[ExternalEffectCorrelationIdentity],
    ) -> WorthQueryInboundCleanupReport {
        let mut report = WorthQueryInboundCleanupReport::default();
        for correlation in correlations {
            let Some(message) = self.by_correlation.get(correlation).cloned() else {
                continue;
            };
            let Some(entry) = self.by_message.get(&message) else {
                continue;
            };
            let expiry = entry.accepted.claims().expires_at_unix_seconds;
            if entry.accepted.operation() != operation
                || expiry >= now_unix_seconds
                || entry.terminal_release_pending
                || !entry
                    .terminal
                    .as_ref()
                    .is_some_and(|terminal| terminal.delivery_settled())
            {
                continue;
            }
            let by_expiry = self
                .terminal_expiry
                .get_mut(operation)
                .expect("selected terminal operation remains indexed");
            let messages = by_expiry
                .get_mut(&expiry)
                .expect("selected terminal expiry remains indexed");
            assert!(messages.remove(&message));
            if messages.is_empty() {
                by_expiry.remove(&expiry);
            }
            if by_expiry.is_empty() {
                self.terminal_expiry.remove(operation);
            }
            let entry = self
                .by_message
                .remove(&message)
                .expect("expiry index selects retained accepted occurrence");
            let terminal = entry.terminal.as_ref().expect("only terminal is indexed");
            assert!(!entry.terminal_release_pending && terminal.delivery_settled());
            assert_eq!(entry.accepted.operation(), operation);
            assert_eq!(self.by_correlation.remove(correlation), Some(message));
            let usage = self
                .usage_by_operation
                .get_mut(operation)
                .expect("accepted operation remains charged");
            usage.count = usage
                .count
                .checked_sub(1)
                .expect("count charged exactly once");
            usage.bytes = usage
                .bytes
                .checked_sub(entry.charged_bytes)
                .expect("bytes charged exactly once");
            report.reclaimed += 1;
            report.released_bytes += entry.charged_bytes;
        }
        report
    }
}
