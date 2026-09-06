mod admission;
mod counters;
mod denial;
mod entry;
mod metadata;
mod publication;
mod reachability;
mod reservation;
mod slots;
mod support;
mod traversal;

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, OnceLock};

use crate::budget::RuntimeWorldBudgetLimit;
use crate::identity::{CompositeCommitIdentity, RuntimeWorldOwnerIdentity};

use super::reclamation::{
    CompositeHistoryReclamationRequest, HistoryReclamationDenial, HistoryReclamationOutcome,
};
use super::retention::{
    CompositeHistoryProtectionObligation, ExplicitCommitHistoryProtectionObligation,
    HistoryProtectionClass, ProductHeadHistoryProtectionObligation,
};
use super::{CompositeCommitParent, CompositeRuntimeWorldCommit};

pub use counters::HistoryCatalogCounters;
pub use denial::CompositeHistoryCatalogDenial;
pub(crate) use entry::CompositeHistoryCatalogEntry;
pub use metadata::HistoryMetadataLedger;
pub(super) use metadata::HistoryReservationMetadata;
pub(in crate::history) use reachability::{
    lock_index, HistoryReachabilityHandle, HistoryReachabilityIndex,
};
pub(crate) use reservation::ReservedCompositeCommitCapacity;
use support::{lock_state, prevalidate_candidate_prefix, remove_installed, validate_owner};
pub use traversal::CompositeHistoryTraversal;

/// Installed limits consumed by the immutable history owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeWorldHistoryCatalogContract {
    maximum_commits: RuntimeWorldBudgetLimit,
    maximum_metadata_bytes: RuntimeWorldBudgetLimit,
}

impl RuntimeWorldHistoryCatalogContract {
    pub(crate) const fn installed(
        maximum_commits: RuntimeWorldBudgetLimit,
        maximum_metadata_bytes: RuntimeWorldBudgetLimit,
    ) -> Self {
        Self {
            maximum_commits,
            maximum_metadata_bytes,
        }
    }

    pub(crate) const fn maximum_commits(self) -> RuntimeWorldBudgetLimit {
        self.maximum_commits
    }

    pub(crate) const fn maximum_metadata_bytes(self) -> RuntimeWorldBudgetLimit {
        self.maximum_metadata_bytes
    }
}

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod tests;

/// Immutable single-parent history owned by one Runtime World owner.
#[derive(Debug, Clone)]
pub(crate) struct CompositeHistoryCatalog {
    state: Arc<Mutex<CompositeHistoryCatalogState>>,
}

#[derive(Debug)]
pub(super) struct CompositeHistoryCatalogState {
    owner: RuntimeWorldOwnerIdentity,
    limits: RuntimeWorldHistoryCatalogContract,
    // Reservation allocates the eventual stable slot before owner effects.
    // Only a populated slot is an installed occurrence.
    entries: HashMap<CompositeCommitIdentity, Arc<OnceLock<CompositeHistoryCatalogEntry>>>,
    reservations: HashMap<CompositeCommitIdentity, HistoryReservationMetadata>,
    metadata: HistoryMetadataLedger,
    reachability: HistoryReachabilityHandle,
    counters: counters::HistoryCatalogCountersHandle,
    root: Option<CompositeCommitIdentity>,
    root_reserved: bool,
    root_ever_installed: bool,
}

impl CompositeHistoryCatalog {
    pub(crate) fn new(
        owner: RuntimeWorldOwnerIdentity,
        contract: RuntimeWorldHistoryCatalogContract,
    ) -> Self {
        let counters = counters::new_handle();
        let reachability = Arc::new(Mutex::new(HistoryReachabilityIndex::new(Arc::clone(
            &counters,
        ))));
        Self {
            state: Arc::new(Mutex::new(CompositeHistoryCatalogState {
                owner,
                limits: contract,
                entries: HashMap::new(),
                reservations: HashMap::new(),
                metadata: HistoryMetadataLedger::default(),
                reachability,
                counters,
                root: None,
                root_reserved: false,
                root_ever_installed: false,
            })),
        }
    }

    #[cfg(test)]
    pub(crate) fn append(
        &self,
        commit: Arc<CompositeRuntimeWorldCommit>,
        pins: crate::retention::HistoryRetentionObligation,
    ) -> Result<Arc<CompositeRuntimeWorldCommit>, CompositeHistoryCatalogDenial> {
        let reservation = self.reserve(commit.as_ref())?;
        reservation.install(commit, pins)
    }

    #[cfg(test)]
    pub(crate) fn lookup(
        &self,
        identity: &CompositeCommitIdentity,
    ) -> Option<Arc<CompositeRuntimeWorldCommit>> {
        let state = lock_state(&self.state);
        counters::lock_counters(&state.counters).record_entry_lookup();
        state
            .entries
            .get(identity)
            .and_then(|slot| slot.get())
            .map(|entry| Arc::clone(&entry.commit))
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        let state = lock_state(&self.state);
        state.entries.len() - state.reservations.len()
    }

    #[cfg(test)]
    pub(crate) fn reserved_len(&self) -> usize {
        lock_state(&self.state).reservations.len()
    }

    pub(crate) fn arm_installed_root_rollback(
        &self,
        identity: &CompositeCommitIdentity,
    ) -> reservation::InstalledRootCommitRollback {
        let state = lock_state(&self.state);
        assert!(
            state
                .entries
                .get(identity)
                .and_then(|slot| slot.get())
                .is_some_and(|entry| matches!(
                    entry.commit().parent(),
                    CompositeCommitParent::Root
                )),
            "root rollback must be armed for an installed root entry"
        );
        reservation::InstalledRootCommitRollback::new(Arc::clone(&self.state), identity.clone())
    }

    pub(crate) fn snapshot(&self) -> crate::inspection::RuntimeWorldHistorySnapshot {
        let state = lock_state(&self.state);
        let costs = *counters::lock_counters(&state.counters);
        crate::inspection::RuntimeWorldHistorySnapshot {
            installed: state.entries.len() - state.reservations.len(),
            reserved: state.reservations.len(),
            metadata: state.metadata,
            costs,
        }
    }

    #[cfg(test)]
    pub(crate) fn metadata_ledger(&self) -> HistoryMetadataLedger {
        lock_state(&self.state).metadata
    }

    #[cfg(test)]
    pub(crate) fn counters(&self) -> HistoryCatalogCounters {
        let state = lock_state(&self.state);
        let counters = *counters::lock_counters(&state.counters);
        counters
    }

    fn protect_exact(
        &self,
        identity: CompositeCommitIdentity,
        class: HistoryProtectionClass,
    ) -> Result<CompositeHistoryProtectionObligation, CompositeHistoryCatalogDenial> {
        let state = lock_state(&self.state);
        validate_owner(&state, identity.owner_identity())?;
        if state
            .entries
            .get(&identity)
            .is_none_or(|slot| slot.get().is_none())
        {
            return Err(CompositeHistoryCatalogDenial::UnknownProtectionTarget(
                identity,
            ));
        }
        {
            let mut reachability = lock_index(&state.reachability);
            reachability.increment_direct_protection(&identity)?;
        }
        Ok(CompositeHistoryProtectionObligation::new(
            Arc::clone(&state.reachability),
            identity,
            class,
        ))
    }

    /// Issue the only cross-module history protection capability used by a
    /// product reference. Callers cannot choose another protection class.
    pub(crate) fn protect_product_head(
        &self,
        commit: &CompositeRuntimeWorldCommit,
    ) -> Result<ProductHeadHistoryProtectionObligation, CompositeHistoryCatalogDenial> {
        self.protect_exact(
            commit.identity().clone(),
            HistoryProtectionClass::ProductHead,
        )
        .map(ProductHeadHistoryProtectionObligation::issued)
    }

    /// Issue the exact history protection carried by a live commit-bound
    /// consumer such as a managed product-branch observation.
    pub(crate) fn protect_explicit_commit(
        &self,
        commit: &CompositeRuntimeWorldCommit,
    ) -> Result<ExplicitCommitHistoryProtectionObligation, CompositeHistoryCatalogDenial> {
        self.protect_exact(
            commit.identity().clone(),
            HistoryProtectionClass::ExplicitObligation,
        )
        .map(ExplicitCommitHistoryProtectionObligation::issued)
    }

    /// Walk one parent chain up to an explicit caller bound. Reclamation does
    /// not call this method; its reachability decision is index-local.
    pub(crate) fn trace_ancestry(
        &self,
        start: CompositeCommitIdentity,
        maximum_commits: NonZeroUsize,
    ) -> Result<CompositeHistoryTraversal, CompositeHistoryCatalogDenial> {
        let state = lock_state(&self.state);
        validate_owner(&state, start.owner_identity())?;
        if state
            .entries
            .get(&start)
            .is_none_or(|slot| slot.get().is_none())
        {
            return Err(CompositeHistoryCatalogDenial::UnknownProtectionTarget(
                start,
            ));
        }
        lock_index(&state.reachability).increment_direct_protection(&start)?;
        let protection = CompositeHistoryProtectionObligation::new(
            Arc::clone(&state.reachability),
            start.clone(),
            HistoryProtectionClass::ExplicitObligation,
        );
        let mut current = start;
        let mut commits = Vec::with_capacity(maximum_commits.get().min(state.entries.len()));
        for _ in 0..maximum_commits.get() {
            let entry = state
                .entries
                .get(&current)
                .and_then(|slot| slot.get())
                .ok_or_else(|| CompositeHistoryCatalogDenial::MissingParent(current.clone()))?;
            commits.push(Arc::clone(&entry.commit));
            let Some(parent) = support::ordinary_parent_identity(entry.commit().parent()) else {
                return Ok(CompositeHistoryTraversal {
                    commits,
                    _protection: protection,
                    _catalog: self.clone(),
                    next_parent: None,
                });
            };
            current = parent;
        }
        Ok(CompositeHistoryTraversal {
            commits,
            _protection: protection,
            _catalog: self.clone(),
            next_parent: Some(current),
        })
    }

    pub(crate) fn reclaim_batch(
        &self,
        request: CompositeHistoryReclamationRequest,
    ) -> Result<HistoryReclamationOutcome, HistoryReclamationDenial> {
        let candidate_count = request
            .candidate_commits()
            .len()
            .min(request.maximum_reclaims());
        // Removed entries can own the last immutable component-result image.
        // Allocate the bounded release list before taking the catalog lock,
        // and release those images only after that lock is gone.
        let mut released_entries = Vec::with_capacity(candidate_count);
        let mut state = lock_state(&self.state);
        validate_owner(&state, request.owner()).map_err(HistoryReclamationDenial::Catalog)?;
        let maximum_reclaims = request.maximum_reclaims();
        if maximum_reclaims == 0 {
            return Ok(HistoryReclamationOutcome::new(0));
        }
        let candidates = &request.candidate_commits()[..candidate_count];
        prevalidate_candidate_prefix(&mut state, candidates)?;

        let mut outcome = HistoryReclamationOutcome::new(maximum_reclaims);
        for candidate in candidates {
            outcome.examined_one();
            let reachability = {
                let mut index = lock_index(&state.reachability);
                index
                    .lookup(candidate)
                    .expect("prevalidated candidate has a reachability row")
            };
            if reachability.direct_protections() > 0 {
                outcome.record_skipped_protected();
                continue;
            }
            if reachability.descendant_dependencies() > 0 {
                outcome.record_skipped_with_descendant_dependencies();
                continue;
            }
            let entry = remove_installed(&mut state, candidate);
            outcome.reclaimed_one(
                candidate.clone(),
                entry
                    .get()
                    .expect("removed installed slot")
                    .metadata_charge()
                    .total(),
            );
            released_entries.push(entry);
        }
        drop(state);
        drop(released_entries);
        Ok(outcome)
    }
}
