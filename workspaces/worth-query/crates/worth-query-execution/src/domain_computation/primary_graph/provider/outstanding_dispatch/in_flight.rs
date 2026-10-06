//! Physical dispatch lease transferred into Completed recovery custody.

use std::sync::{Arc, Mutex};

use super::{decrement, OutstandingDispatchOwner};
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryOutstandingInFlightDenial {
    Missing,
    PendingPublication,
    OriginalMismatch,
    /// World already performed this effect's terminal completion.
    TerminalReached,
    CapacityExhausted,
}

/// Move-only claim that keeps the original dispatch's finite capacity charge
/// while a physical send or its Completed recovery custody may still act.
pub(in crate::domain_computation::primary_graph) struct OutstandingDispatchInFlightLease {
    owner: OutstandingDispatchOwner,
    basis_retention: Arc<Mutex<super::super::session_commit::WorthQueryReceiptBasisRetentionStore>>,
    correlation: ExternalEffectCorrelationIdentity,
}

impl OutstandingDispatchOwner {
    pub(in crate::domain_computation::primary_graph::provider) fn begin_in_flight(
        &self,
        original: &WorthQueryCommittedDispatchOutboxObservation,
        basis_retention: Arc<
            Mutex<super::super::session_commit::WorthQueryReceiptBasisRetentionStore>,
        >,
    ) -> Result<OutstandingDispatchInFlightLease, WorthQueryOutstandingInFlightDenial> {
        use WorthQueryOutstandingInFlightDenial as Denial;
        let record = original.record();
        let contract = record.inbound().ok_or(Denial::OriginalMismatch)?;
        let publication = original.committed_product_publication();
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = entries
            .by_correlation
            .get_mut(record.correlation())
            .ok_or(Denial::Missing)?;
        let commit = entry.commit.ok_or(Denial::PendingPublication)?;
        if entry.contract != *contract
            || commit != original.commit_reference().commit_id
            || publication.relational_commit() != original.commit_reference()
            || entry.branch != *publication.product_branch()
            || entry.incarnation != publication.product_incarnation()
            || entry.performed_world.as_ref().is_none_or(|performed| {
                performed.composite_commit() != publication.composite_commit()
                    || performed.publication_attempt() != publication.publication_attempt()
            })
        {
            return Err(Denial::OriginalMismatch);
        }
        if entry.terminal_world.is_some() {
            return Err(Denial::TerminalReached);
        }
        let next = entry
            .in_flight
            .checked_add(1)
            .ok_or(Denial::CapacityExhausted)?;
        if next
            > contract
                .limits()
                .maximum_outstanding_dispatch_provenance
                .get()
        {
            return Err(Denial::CapacityExhausted);
        }
        entry.in_flight = next;
        Ok(OutstandingDispatchInFlightLease {
            owner: self.clone(),
            basis_retention,
            correlation: *record.correlation(),
        })
    }
}

impl Drop for OutstandingDispatchInFlightLease {
    fn drop(&mut self) {
        let mut basis = self
            .basis_retention
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut entries = self
            .owner
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(entry) = entries.by_correlation.get_mut(&self.correlation) else {
            return;
        };
        assert!(
            entry.in_flight > 0,
            "in-flight dispatch lease remains counted"
        );
        entry.in_flight -= 1;
        if entry.in_flight != 0 || entry.terminal_world.is_none() {
            return;
        }
        let Some(commit) = entry.commit else {
            return;
        };
        if !basis.has_mandatory(commit) {
            return;
        }
        let removed = entries
            .by_correlation
            .remove(&self.correlation)
            .expect("lease owner remains");
        decrement(&mut entries.counts, &removed.contract);
        decrement(&mut entries.by_branch, &removed.incarnation);
        basis.release_mandatory(commit);
    }
}
