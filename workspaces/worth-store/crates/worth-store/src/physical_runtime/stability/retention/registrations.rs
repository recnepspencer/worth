use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use worth_store_physical_format::{DurablePhysicalRootManifest, RootPublicationCell};

use super::super::{
    PhysicalProtectedRootObservation, PhysicalReadProtectionDenial, PhysicalReadProtectionPolicy,
};
use super::{PhysicalReadProtectionObservation, PhysicalRootReadLease};
use crate::physical_runtime::{
    lifecycle::{LifecycleState, ObservedLifecyclePhase},
    RuntimeIdentity,
};

pub(in crate::physical_runtime) struct RootProtectionRegistry {
    runtime: RuntimeIdentity,
    lifecycle: Arc<LifecycleState>,
    policy: PhysicalReadProtectionPolicy,
    state: Mutex<Registrations>,
}

struct Registrations {
    revoked: bool,
    slots: Vec<RegistrationSlot>,
    free: Option<usize>,
    roots: HashMap<u64, ProtectedRoot>,
    live: u32,
    acquisitions: u64,
    releases: u64,
    index_probes: u64,
}

struct RegistrationSlot {
    root: Option<RootPublicationCell>,
    owners: u64,
    next_free: Option<usize>,
}

struct ProtectedRoot {
    manifest: DurablePhysicalRootManifest,
    acquisitions: u32,
}

impl RootProtectionRegistry {
    pub(in crate::physical_runtime::stability) fn admit(
        policy: PhysicalReadProtectionPolicy,
        runtime: RuntimeIdentity,
        lifecycle: Arc<LifecycleState>,
    ) -> Result<Self, PhysicalReadProtectionDenial> {
        let capacity = usize::try_from(policy.acquisitions().get())
            .map_err(|_| PhysicalReadProtectionDenial::MetadataUnavailable)?;
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(capacity)
            .map_err(|_| PhysicalReadProtectionDenial::MetadataUnavailable)?;
        for index in 0..capacity {
            slots.push(RegistrationSlot {
                root: None,
                owners: 0,
                next_free: (index + 1 < capacity).then_some(index + 1),
            });
        }
        let mut roots = HashMap::new();
        roots
            .try_reserve(policy.roots().get().min(policy.acquisitions().get()) as usize)
            .map_err(|_| PhysicalReadProtectionDenial::MetadataUnavailable)?;
        Ok(Self {
            runtime,
            lifecycle,
            policy,
            state: Mutex::new(Registrations {
                revoked: false,
                slots,
                free: Some(0),
                roots,
                live: 0,
                acquisitions: 0,
                releases: 0,
                index_probes: 0,
            }),
        })
    }

    /// Called only while the authoritative current-root mutex is held.
    pub(in crate::physical_runtime) fn capture(
        self: &Arc<Self>,
        root: &DurablePhysicalRootManifest,
    ) -> Result<PhysicalRootReadLease, PhysicalReadProtectionDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let lifecycle = self.lifecycle.snapshot();
        if state.revoked || lifecycle.phase != ObservedLifecyclePhase::RecordServing {
            return Err(PhysicalReadProtectionDenial::Revoked);
        }
        let slot = state
            .free
            .ok_or(PhysicalReadProtectionDenial::ProtectionLimit)?;
        let key = root.root_cell();
        state.index_probes = state.index_probes.saturating_add(1);
        if !state.roots.contains_key(&root.generation())
            && state.roots.len() >= self.policy.roots().get() as usize
        {
            return Err(PhysicalReadProtectionDenial::RetainedRootLimit);
        }
        state.index_probes = state.index_probes.saturating_add(1);
        let protected = state
            .roots
            .entry(root.generation())
            .or_insert_with(|| ProtectedRoot {
                manifest: root.clone(),
                acquisitions: 0,
            });
        assert_eq!(
            &protected.manifest, root,
            "one root cell cannot name competing root contents"
        );
        protected.acquisitions += 1;
        state.free = state.slots[slot].next_free;
        state.slots[slot] = RegistrationSlot {
            root: Some(key),
            owners: 1,
            next_free: None,
        };
        state.live += 1;
        state.acquisitions = state.acquisitions.saturating_add(1);
        Ok(PhysicalRootReadLease::new(
            Arc::clone(self),
            slot,
            PhysicalProtectedRootObservation::new(self.runtime, lifecycle.generation, key),
        ))
    }

    pub(super) fn share(&self, slot: usize, root: RootPublicationCell) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = &mut state.slots[slot];
        assert_eq!(
            entry.root,
            Some(root),
            "a live lease keeps its registration slot"
        );
        entry.owners = entry
            .owners
            .checked_add(1)
            .expect("read lease owner count exhausted");
    }

    pub(super) fn release(&self, slot: usize, root: RootPublicationCell) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = &mut state.slots[slot];
        assert_eq!(
            entry.root,
            Some(root),
            "only the slot's live lease may release it"
        );
        entry.owners -= 1;
        if entry.owners != 0 {
            return;
        }
        state.index_probes = state.index_probes.saturating_add(1);
        let protected = state
            .roots
            .get_mut(&root.generation().get())
            .expect("live registration has indexed root");
        protected.acquisitions -= 1;
        if protected.acquisitions == 0 {
            state.index_probes = state.index_probes.saturating_add(1);
            state.roots.remove(&root.generation().get());
        }
        state.slots[slot] = RegistrationSlot {
            root: None,
            owners: 0,
            next_free: state.free,
        };
        state.free = Some(slot);
        state.live -= 1;
        state.releases = state.releases.saturating_add(1);
    }

    pub(super) fn require_live(
        &self,
        binding: PhysicalProtectedRootObservation,
    ) -> Result<(), PhysicalReadProtectionDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let lifecycle = self.lifecycle.snapshot();
        if state.revoked
            || lifecycle.phase != ObservedLifecyclePhase::RecordServing
            || lifecycle.generation != binding.lifecycle()
        {
            Err(PhysicalReadProtectionDenial::Revoked)
        } else {
            Ok(())
        }
    }

    pub(in crate::physical_runtime::stability) fn revoke(&self) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .revoked = true;
    }

    pub(super) fn observation(&self) -> PhysicalReadProtectionObservation {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        PhysicalReadProtectionObservation::new(
            self.policy,
            state.live,
            state.roots.len() as u32,
            state.acquisitions,
            state.releases,
            state.index_probes,
            state.revoked
                || self.lifecycle.snapshot().phase != ObservedLifecyclePhase::RecordServing,
        )
    }

    pub(super) fn root_acquisitions(&self, binding: PhysicalProtectedRootObservation) -> u32 {
        if binding.runtime() != self.runtime {
            return 0;
        }
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state
            .roots
            .get(&binding.root().generation().get())
            .filter(|root| root.manifest.root_cell() == binding.root())
            .map_or(0, |root| root.acquisitions)
    }
}
