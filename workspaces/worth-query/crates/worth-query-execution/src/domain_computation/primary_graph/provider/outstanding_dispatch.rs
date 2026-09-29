//! Bounded custody for inbound-capable dispatches before and after commit.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use worth_query_installation::facade::InstalledInboundOccurrenceContract;
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, CompositePublicationAttemptIdentity, ConsumedCompositePublication,
    ProductBranchIdentity, ProductBranchIncarnation, ProductBranchObservation,
};

use crate::domain_computation::application_aftermath::{
    ExternalEffectCorrelationIdentity, WorthQueryDispatchOutboxRecord,
    WorthQueryInboundTerminalOwnerResult,
};
use crate::domain_computation::primary_graph::{
    PerformedInstalledTransportCompletion, WorthQueryCommittedDispatchOutboxObservation,
};

mod in_flight;
pub(in crate::domain_computation::primary_graph) use in_flight::{
    OutstandingDispatchInFlightLease, WorthQueryOutstandingInFlightDenial,
};

#[derive(Clone, Default)]
pub(super) struct OutstandingDispatchOwner {
    entries: Arc<Mutex<OutstandingDispatchEntries>>,
}

#[derive(Default)]
struct OutstandingDispatchEntries {
    by_correlation: BTreeMap<ExternalEffectCorrelationIdentity, OutstandingDispatchEntry>,
    counts: BTreeMap<InstalledInboundOccurrenceContract, u64>,
    by_branch: BTreeMap<ProductBranchIncarnation, u64>,
}

struct OutstandingDispatchEntry {
    contract: InstalledInboundOccurrenceContract,
    branch: ProductBranchIdentity,
    incarnation: ProductBranchIncarnation,
    commit: Option<CommitId>,
    performed_world: Option<(CompositeCommitIdentity, CompositePublicationAttemptIdentity)>,
    in_flight: u64,
    terminal_world: Option<CompositeCommitIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OutstandingDispatchPosture {
    PendingPublication,
    Committed(CommitId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OutstandingDispatchReservationDenial {
    CapacityExhausted,
    CorrelationAlreadyReserved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryTerminalDispatchReleaseDenial {
    TerminalIndexUnavailable,
    Missing,
    PendingPublication,
    OriginalMismatch,
    CompletionWorldMismatch,
    ExactCommitUnavailable,
}

/// Move-only reservation; drop releases only work that never reached cutover.
pub(in crate::domain_computation::primary_graph) struct OutstandingDispatchReservation {
    owner: OutstandingDispatchOwner,
    correlation: ExternalEffectCorrelationIdentity,
    committed: bool,
}

impl OutstandingDispatchOwner {
    pub(super) fn reserve(
        &self,
        record: &WorthQueryDispatchOutboxRecord,
        observation: &ProductBranchObservation,
    ) -> Result<Option<OutstandingDispatchReservation>, OutstandingDispatchReservationDenial> {
        let Some(contract) = record.inbound() else {
            return Ok(None);
        };
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if entries.by_correlation.contains_key(record.correlation()) {
            return Err(OutstandingDispatchReservationDenial::CorrelationAlreadyReserved);
        }
        let count = entries.counts.get(contract).copied().unwrap_or(0);
        if count
            >= contract
                .limits()
                .maximum_outstanding_dispatch_provenance
                .get()
        {
            return Err(OutstandingDispatchReservationDenial::CapacityExhausted);
        }
        entries.counts.insert(contract.clone(), count + 1);
        *entries
            .by_branch
            .entry(observation.lifecycle_incarnation())
            .or_default() += 1;
        entries.by_correlation.insert(
            *record.correlation(),
            OutstandingDispatchEntry {
                contract: contract.clone(),
                branch: observation.branch_identity().clone(),
                incarnation: observation.lifecycle_incarnation(),
                commit: None,
                performed_world: None,
                in_flight: 0,
                terminal_world: None,
            },
        );
        Ok(Some(OutstandingDispatchReservation {
            owner: self.clone(),
            correlation: *record.correlation(),
            committed: false,
        }))
    }

    pub(super) fn posture(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Option<OutstandingDispatchPosture> {
        let entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        entries
            .by_correlation
            .get(correlation)
            .map(|entry| match entry.commit {
                Some(commit) => OutstandingDispatchPosture::Committed(commit),
                None => OutstandingDispatchPosture::PendingPublication,
            })
    }

    pub(super) fn has_branch_obligation(&self, incarnation: ProductBranchIncarnation) -> bool {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .by_branch
            .contains_key(&incarnation)
    }

    pub(super) fn release_terminal(
        &self,
        terminal: &WorthQueryInboundTerminalOwnerResult,
        basis_retention: &mut super::session_commit::WorthQueryReceiptBasisRetentionStore,
    ) -> Result<(), WorthQueryTerminalDispatchReleaseDenial> {
        self.release_world_terminal(
            terminal.accepted().owner(),
            terminal.original_incarnation(),
            terminal.publication().publication(),
            basis_retention,
        )
    }

    pub(super) fn release_transport_terminal(
        &self,
        terminal: &PerformedInstalledTransportCompletion,
        basis_retention: &mut super::session_commit::WorthQueryReceiptBasisRetentionStore,
    ) -> Result<(), WorthQueryTerminalDispatchReleaseDenial> {
        self.release_world_terminal(
            terminal.evidence().committed(),
            terminal.original_incarnation(),
            terminal.publication().publication(),
            basis_retention,
        )
    }

    fn release_world_terminal(
        &self,
        original: &WorthQueryCommittedDispatchOutboxObservation,
        original_incarnation: ProductBranchIncarnation,
        completed: &ConsumedCompositePublication,
        basis_retention: &mut super::session_commit::WorthQueryReceiptBasisRetentionStore,
    ) -> Result<(), WorthQueryTerminalDispatchReleaseDenial> {
        use WorthQueryTerminalDispatchReleaseDenial as Denial;
        let correlation = original.record().correlation();
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = entries
            .by_correlation
            .get_mut(correlation)
            .ok_or(Denial::Missing)?;
        let commit = entry.commit.ok_or(Denial::PendingPublication)?;
        let original_world = original.committed_product_publication();
        if original.commit_reference().commit_id != commit
            || original.record().inbound() != Some(&entry.contract)
            || original_world.relational_commit() != original.commit_reference()
            || entry.branch != *original_world.product_branch()
            || entry.incarnation != original_incarnation
            || entry.incarnation != original_world.product_incarnation()
            || entry
                .performed_world
                .as_ref()
                .is_none_or(|(world, attempt)| {
                    world != original_world.composite_commit()
                        || attempt != original_world.publication_attempt()
                })
        {
            return Err(Denial::OriginalMismatch);
        }
        if completed.new_product_head().branch_identity() != &entry.branch
            || completed.new_product_head().lifecycle_incarnation() != entry.incarnation
            || completed.commit().identity().owner_identity() != entry.branch.owner_identity()
            || completed
                .component_results()
                .relational_commit_result()
                .is_none()
        {
            return Err(Denial::CompletionWorldMismatch);
        }
        if entry
            .terminal_world
            .as_ref()
            .is_some_and(|prior| prior != completed.commit().identity())
        {
            return Err(Denial::CompletionWorldMismatch);
        }
        if !basis_retention.has_mandatory(commit) {
            return Err(Denial::ExactCommitUnavailable);
        }
        entry.terminal_world = Some(completed.commit().identity().clone());
        if entry.in_flight != 0 {
            return Ok(());
        }
        let removed = entries
            .by_correlation
            .remove(correlation)
            .expect("validated terminal correlation remains reserved");
        decrement(&mut entries.counts, &removed.contract);
        decrement(&mut entries.by_branch, &removed.incarnation);
        basis_retention.release_mandatory(commit);
        Ok(())
    }

    fn release_uncommitted(&self, correlation: ExternalEffectCorrelationIdentity) {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if entries
            .by_correlation
            .get(&correlation)
            .is_some_and(|entry| entry.commit.is_none())
        {
            let entry = entries
                .by_correlation
                .remove(&correlation)
                .expect("present reservation");
            decrement(&mut entries.counts, &entry.contract);
            decrement(&mut entries.by_branch, &entry.incarnation);
        }
    }
}

impl OutstandingDispatchReservation {
    pub(in crate::domain_computation::primary_graph) fn commit(
        mut self,
        commit: CommitId,
        publication: &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    ) {
        let mut entries = self
            .owner
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = entries
            .by_correlation
            .get_mut(&self.correlation)
            .expect("reserved correlation remains owner-held");
        assert!(entry.commit.is_none(), "one dispatch is cut over once");
        assert_eq!(
            publication.relational_commit().commit_id,
            commit,
            "World performed the reserved Relational commit"
        );
        assert_eq!(
            publication.product_branch(),
            &entry.branch,
            "World performed the reserved branch"
        );
        assert_eq!(
            publication.product_incarnation(),
            entry.incarnation,
            "World performed the reserved incarnation"
        );
        entry.commit = Some(commit);
        entry.performed_world = Some((
            publication.composite_commit().clone(),
            publication.publication_attempt().clone(),
        ));
        self.committed = true;
    }
}

impl Drop for OutstandingDispatchReservation {
    fn drop(&mut self) {
        if !self.committed {
            self.owner.release_uncommitted(self.correlation);
        }
    }
}

fn decrement<Key: Ord>(counts: &mut BTreeMap<Key, u64>, key: &Key) {
    let count = counts.get_mut(key).expect("reserved key counted");
    *count -= 1;
    if *count == 0 {
        counts.remove(key);
    }
}
