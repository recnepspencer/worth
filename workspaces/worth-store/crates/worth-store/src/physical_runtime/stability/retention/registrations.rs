use std::sync::{Arc, Mutex};

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
    roots: CountedMap<u64, ProtectedRoot>,
    inline_tails: CountedMap<(u64, u64), u32>,
    live: u32,
    acquisitions: u64,
    releases: u64,
    index_probes: u64,
    examined_entries: u64,
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
        let root_bound = policy.roots().get().min(policy.acquisitions().get()) as usize;
        let roots = CountedMap::with_capacity(root_bound)?;
        let inline_tails = CountedMap::with_capacity(root_bound)?;
        Ok(Self {
            runtime,
            lifecycle,
            policy,
            state: Mutex::new(Registrations {
                revoked: false,
                slots,
                free: Some(0),
                roots,
                inline_tails,
                live: 0,
                acquisitions: 0,
                releases: 0,
                index_probes: 0,
                examined_entries: 0,
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
        let (visited, present) = state.roots.contains(&root.generation());
        state.examined_entries = state.examined_entries.saturating_add(visited);
        let fresh = !present;
        if fresh && state.roots.len() >= self.policy.roots().get() as usize {
            return Err(PhysicalReadProtectionDenial::RetainedRootLimit);
        }
        state.index_probes = state.index_probes.saturating_add(1);
        let tail = fresh.then(|| inline_tail(root)).flatten();
        {
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
        }
        if let Some(tail) = tail {
            *state.inline_tails.entry(tail).or_insert(0) += 1;
        }
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
        let generation = root.generation().get();
        let exhausted = {
            let protected = state
                .roots
                .get_mut(&generation)
                .expect("live registration has indexed root");
            protected.acquisitions -= 1;
            (protected.acquisitions == 0).then(|| inline_tail(&protected.manifest))
        };
        if let Some(tail) = exhausted {
            state.index_probes = state.index_probes.saturating_add(1);
            if let Some(tail) = tail {
                let (visited, count) = state.inline_tails.consult(&tail);
                let count = count.copied();
                state.examined_entries = state.examined_entries.saturating_add(visited);
                let remaining =
                    count.expect("protected tail stays indexed while its root is live") - 1;
                if remaining == 0 {
                    state.inline_tails.remove(&tail);
                } else {
                    state.inline_tails.insert(tail, remaining);
                }
            }
            state.roots.remove(&generation);
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
            state.examined_entries,
            state.revoked
                || self.lifecycle.snapshot().phase != ObservedLifecyclePhase::RecordServing,
        )
    }

    pub(in crate::physical_runtime) fn protects_root(&self, generation: u64) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.index_probes = state.index_probes.saturating_add(1);
        let (visited, protected) = state.roots.contains(&generation);
        state.examined_entries = state.examined_entries.saturating_add(visited);
        protected
    }

    /// True when a live reader holds any root at or below generation.
    ///
    /// A displaced extent generation is readable from every root between the
    /// one that placed it and the rewrite's source root. No index names that
    /// span, so the scan is counted and bounded by the retained-root limit.
    pub(in crate::physical_runtime) fn protects_root_at_or_below(&self, generation: u64) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.index_probes = state.index_probes.saturating_add(1);
        let (visited, protected) = state.roots.any_key(|root| *root <= generation);
        state.examined_entries = state.examined_entries.saturating_add(visited);
        protected
    }

    /// True when a live reader still names this inline segment as its tail.
    ///
    /// The lookup is the tail index. It does not walk protected roots or the
    /// published graph.
    pub(in crate::physical_runtime) fn protects_inline_segment(
        &self,
        segment_id: u64,
        generation: u64,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.index_probes = state.index_probes.saturating_add(1);
        let (visited, count) = state.inline_tails.consult(&(segment_id, generation));
        let protected = count.is_some_and(|roots| *roots > 0);
        state.examined_entries = state.examined_entries.saturating_add(visited);
        protected
    }

    pub(super) fn root_acquisitions(&self, binding: PhysicalProtectedRootObservation) -> u32 {
        if binding.runtime() != self.runtime {
            return 0;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (visited, root) = state.roots.consult(&binding.root().generation().get());
        let acquisitions = root
            .filter(|root| root.manifest.root_cell() == binding.root())
            .map_or(0, |root| root.acquisitions);
        state.examined_entries = state.examined_entries.saturating_add(visited);
        acquisitions
    }
}

fn inline_tail(root: &DurablePhysicalRootManifest) -> Option<(u64, u64)> {
    root.last_inline_segment()
        .map(|cell| (cell.segment_id().get(), cell.generation().get()))
}

mod counted {
    use std::collections::HashMap;

    use super::PhysicalReadProtectionDenial;

    /// Map whose reads count each consulted key. The stored entries are private,
    /// so a caller cannot walk every key without a counting scan.
    pub(super) struct CountedMap<K, V> {
        entries: HashMap<K, V>,
    }

    impl<K: Eq + std::hash::Hash, V> CountedMap<K, V> {
        pub(super) fn with_capacity(capacity: usize) -> Result<Self, PhysicalReadProtectionDenial> {
            let mut entries = HashMap::new();
            entries
                .try_reserve(capacity)
                .map_err(|_| PhysicalReadProtectionDenial::MetadataUnavailable)?;
            Ok(Self { entries })
        }

        pub(super) fn consult<'a>(&'a self, key: &K) -> (u64, Option<&'a V>) {
            (1, self.entries.get(key))
        }

        pub(super) fn contains(&self, key: &K) -> (u64, bool) {
            let (visited, value) = self.consult(key);
            (visited, value.is_some())
        }

        /// Charges every stored key as visited, whether or not the scan stops early.
        pub(super) fn any_key(&self, matches: impl Fn(&K) -> bool) -> (u64, bool) {
            let visited = self.entries.len() as u64;
            (visited, self.entries.keys().any(matches))
        }

        pub(super) fn len(&self) -> usize {
            self.entries.len()
        }

        pub(super) fn entry(&mut self, key: K) -> std::collections::hash_map::Entry<'_, K, V> {
            self.entries.entry(key)
        }

        pub(super) fn get_mut(&mut self, key: &K) -> Option<&mut V> {
            self.entries.get_mut(key)
        }

        pub(super) fn insert(&mut self, key: K, value: V) -> Option<V> {
            self.entries.insert(key, value)
        }

        pub(super) fn remove(&mut self, key: &K) -> Option<V> {
            self.entries.remove(key)
        }
    }
}

use counted::CountedMap;
