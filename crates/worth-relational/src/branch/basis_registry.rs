use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

use crate::history::data::BranchId;

use super::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisInner,
    RelationalBranchBasisDescriptor, RelationalBranchVersion,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RelationalRetainedBasisKey {
    runtime_instance_id: u64,
    branch_id: BranchId,
    generation: u64,
    truth_version: RelationalBranchVersion,
    root_identity: u64,
    schema_commitment: [u8; 32],
    visibility_commitment: [u8; 32],
}

impl From<&RelationalBranchBasisDescriptor> for RelationalRetainedBasisKey {
    fn from(descriptor: &RelationalBranchBasisDescriptor) -> Self {
        Self {
            runtime_instance_id: descriptor.runtime_instance_id(),
            branch_id: descriptor.branch_id().clone(),
            generation: descriptor.reference().generation().get(),
            truth_version: descriptor.truth_version(),
            root_identity: descriptor.root_identity(),
            schema_commitment: descriptor.schema_commitment(),
            visibility_commitment: descriptor.visibility_commitment(),
        }
    }
}

/// Weak owner index for exact admitted roots.
///
/// The index does not retain a root by itself. An admitted basis, observation,
/// or explicit external pin owns the `Arc`; once the last obligation is gone,
/// readmission returns unavailable instead of reconstructing authority.
#[derive(Debug, Clone)]
pub(crate) struct RelationalBranchBasisRegistry {
    retained: Arc<Mutex<RelationalRetainedBasisState>>,
    metrics: Arc<RelationalBranchBasisRegistryMetrics>,
}

#[derive(Debug, Default)]
pub(crate) struct RelationalBranchBasisRegistryMetrics {
    entries: AtomicU64,
    key_lookups: AtomicU64,
    mutations: AtomicU64,
}

#[derive(Debug, Default)]
struct RelationalRetainedBasisState {
    next_registration_id: u64,
    entries: HashMap<RelationalRetainedBasisKey, RelationalRetainedBasisEntry>,
}

#[derive(Debug)]
struct RelationalRetainedBasisEntry {
    registration_id: u64,
    basis: Weak<AdmittedRelationalBranchBasisInner>,
}

#[derive(Debug)]
pub(super) struct RelationalBasisRegistryLease {
    registry: Weak<Mutex<RelationalRetainedBasisState>>,
    metrics: Weak<RelationalBranchBasisRegistryMetrics>,
    key: RelationalRetainedBasisKey,
    registration_id: u64,
}

impl RelationalBranchBasisRegistry {
    pub(crate) fn with_metrics(metrics: Arc<RelationalBranchBasisRegistryMetrics>) -> Self {
        Self {
            retained: Arc::new(Mutex::new(RelationalRetainedBasisState::default())),
            metrics,
        }
    }

    pub(crate) fn bind_metrics(&mut self, metrics: Arc<RelationalBranchBasisRegistryMetrics>) {
        if Arc::ptr_eq(&self.metrics, &metrics) {
            return;
        }
        let retained = self
            .retained
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(
            /* Proof: All bind_metrics callers bind a fresh registry from RelationalBranchReferenceCell construction, detached_owner_snapshot, rebind_runtime or reset_basis_registry. clone_for_head_replacement keeps the existing metrics and does not rebind a populated registry; there is no facade rebinding door. */
            retained.entries.is_empty(),
            "a live basis registry cannot change metric ownership"
        );
        drop(retained);
        self.metrics = metrics;
    }

    pub(crate) fn register(
        &self,
        basis: AdmittedRelationalBranchBasis,
    ) -> Result<AdmittedRelationalBranchBasis, super::RelationalBranchBasisDenial> {
        let key = RelationalRetainedBasisKey::from(basis.descriptor());
        let mut retained = self
            .retained
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.metrics.record_lookup();
        if let Some(existing) = retained
            .entries
            .get(&key)
            .and_then(|entry| entry.basis.upgrade())
        {
            return Ok(AdmittedRelationalBranchBasis { inner: existing });
        }
        let registration_id = retained
            .next_registration_id
            .checked_add(1)
            .ok_or(super::RelationalBranchBasisDenial::OwnerFailure)?;
        retained.next_registration_id = registration_id;
        let lease = RelationalBasisRegistryLease {
            registry: Arc::downgrade(&self.retained),
            metrics: Arc::downgrade(&self.metrics),
            key: key.clone(),
            registration_id,
        };
        if let Err(uninstalled) = basis.inner.registry_lease.set(lease) {
            std::mem::forget(uninstalled);
            return Err(super::RelationalBranchBasisDenial::OwnerFailure);
        }
        retained.entries.insert(
            key,
            RelationalRetainedBasisEntry {
                registration_id,
                basis: Arc::downgrade(&basis.inner),
            },
        );
        self.metrics.record_insert();
        Ok(basis)
    }

    pub(crate) fn readmit_retained(
        &self,
        descriptor: &RelationalBranchBasisDescriptor,
    ) -> Option<AdmittedRelationalBranchBasis> {
        let key = RelationalRetainedBasisKey::from(descriptor);
        let mut retained_bases = self
            .retained
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.metrics.record_lookup();
        let retained = retained_bases.entries.get(&key)?.basis.upgrade();
        match retained {
            Some(inner) if inner.descriptor == *descriptor => {
                Some(AdmittedRelationalBranchBasis { inner })
            }
            Some(_) => None,
            None => {
                if retained_bases.entries.remove(&key).is_some() {
                    self.metrics.record_remove();
                }
                None
            }
        }
    }
}

impl Default for RelationalBranchBasisRegistry {
    fn default() -> Self {
        Self::with_metrics(Arc::new(RelationalBranchBasisRegistryMetrics::default()))
    }
}

impl RelationalBranchBasisRegistryMetrics {
    pub(crate) fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.entries.load(Ordering::Relaxed),
            self.key_lookups.load(Ordering::Relaxed),
            self.mutations.load(Ordering::Relaxed),
        )
    }

    fn record_lookup(&self) {
        self.key_lookups.fetch_add(1, Ordering::Relaxed);
    }

    fn record_insert(&self) {
        self.entries.fetch_add(1, Ordering::Relaxed);
        self.mutations.fetch_add(1, Ordering::Relaxed);
    }

    fn record_remove(&self) {
        self.entries.fetch_sub(1, Ordering::Relaxed);
        self.mutations.fetch_add(1, Ordering::Relaxed);
    }
}

impl Drop for RelationalBasisRegistryLease {
    fn drop(&mut self) {
        let Some(registry) = self.registry.upgrade() else {
            return;
        };
        let mut retained = registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(metrics) = self.metrics.upgrade() else {
            return;
        };
        metrics.record_lookup();
        let matches_registration = retained
            .entries
            .get(&self.key)
            .is_some_and(|entry| entry.registration_id == self.registration_id);
        if matches_registration {
            retained.entries.remove(&self.key);
            metrics.record_remove();
        }
    }
}
