use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};

use crate::history::data::BranchId;
use crate::runtime::RelationalRuntimeConfig;
use crate::visibility::snapshot_states::{SnapshotState, VisibilitySnapshotStateKey};

#[derive(Debug)]
pub(crate) struct VisibilityCache {
    pub(crate) states: RwLock<HashMap<VisibilitySnapshotStateKey, SnapshotState>>,
    pub(crate) residency: RwLock<HashMap<VisibilitySnapshotStateKey, VisibilityResidency>>,
    branch_head_states: RwLock<HashMap<BranchId, VisibilitySnapshotStateKey>>,
    pub(crate) recent_policy: Mutex<DeterministicVersionWindowPolicy>,
    residency_key_lookups: AtomicU64,
    residency_mutations: AtomicU64,
    branch_head_population_scans: AtomicU64,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct VisibilityCacheCostCounters {
    pub(crate) residency_entries: u64,
    pub(crate) residency_key_lookups: u64,
    pub(crate) residency_mutations: u64,
    pub(crate) branch_head_population_scans: u64,
}

impl VisibilityCache {
    pub(crate) fn new(config: &RelationalRuntimeConfig) -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            residency: RwLock::new(HashMap::new()),
            branch_head_states: RwLock::new(HashMap::new()),
            recent_policy: Mutex::new(DeterministicVersionWindowPolicy {
                recent_version_window: config.visibility.cache_policy.recent_version_window,
                order: VecDeque::new(),
                resident_count: 0,
            }),
            residency_key_lookups: AtomicU64::new(0),
            residency_mutations: AtomicU64::new(0),
            branch_head_population_scans: AtomicU64::new(0),
        }
    }

    pub(crate) fn fork(&self) -> Self {
        let recent_version_window = self
            .recent_policy
            .lock()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "recent visibility policy lock poisoned")
            .recent_version_window;
        Self {
            states: RwLock::new(HashMap::new()),
            residency: RwLock::new(HashMap::new()),
            branch_head_states: RwLock::new(HashMap::new()),
            recent_policy: Mutex::new(DeterministicVersionWindowPolicy {
                recent_version_window,
                order: VecDeque::new(),
                resident_count: 0,
            }),
            residency_key_lookups: AtomicU64::new(0),
            residency_mutations: AtomicU64::new(0),
            branch_head_population_scans: AtomicU64::new(0),
        }
    }

    pub(crate) fn clear(&self) {
        self.states
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility state lock poisoned")
            .clear();
        self.residency
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility residency lock poisoned")
            .clear();
        self.branch_head_states
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "branch-head visibility state lock poisoned")
            .clear();
        let mut recent_policy = self
            .recent_policy
            .lock()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "recent visibility policy lock poisoned");
        recent_policy.order.clear();
        recent_policy.resident_count = 0;
    }

    pub(crate) fn cached_version_count(&self) -> usize {
        self.states
            .read()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility state lock poisoned")
            .len()
    }

    pub(crate) fn protected_state_keys(
        &self,
        protect_active_snapshots: bool,
    ) -> Vec<VisibilitySnapshotStateKey> {
        let mut keys = self
            .residency
            .read()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility residency lock poisoned")
            .iter()
            .filter_map(|(key, entry)| {
                (entry.branch_head_refs > 0
                    || entry.replay_refs > 0
                    || (protect_active_snapshots && entry.active_snapshot_refs > 0))
                    .then_some(key.clone())
            })
            .collect::<Vec<_>>();
        keys.sort();
        keys
    }

    pub(crate) fn recent_visibility_count(&self) -> usize {
        self.recent_policy
            .lock()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "recent visibility policy lock poisoned")
            .resident_count
    }

    pub(crate) fn tracked_branch_head_states(&self) -> Vec<VisibilitySnapshotStateKey> {
        self.branch_head_population_scans
            .fetch_add(1, Ordering::Relaxed);
        let mut keys = self
            .branch_head_states
            .read()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "branch-head visibility state lock poisoned")
            .values()
            .cloned()
            .collect::<Vec<_>>();
        keys.sort();
        keys
    }

    pub(crate) fn track_branch_head_state(&self, key: &VisibilitySnapshotStateKey) {
        self.branch_head_states
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "branch-head visibility state lock poisoned")
            .insert(key.branch_id().clone(), key.clone());
    }

    pub(crate) fn untrack_branch_head_state(
        &self,
        branch_id: &BranchId,
    ) -> Option<VisibilitySnapshotStateKey> {
        self.branch_head_states
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "branch-head visibility state lock poisoned")
            .remove(branch_id)
    }

    pub(crate) fn state(&self, key: &VisibilitySnapshotStateKey) -> Option<SnapshotState> {
        self.states
            .read()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility state lock poisoned")
            .get(key)
            .cloned()
    }

    pub(crate) fn insert_state(&self, state: SnapshotState) {
        let key = state.basis.key();
        self.states
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility state lock poisoned")
            .insert(key, state);
    }

    pub(crate) fn remove_state(&self, key: &VisibilitySnapshotStateKey) {
        self.states
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility state lock poisoned")
            .remove(key);
    }

    pub(crate) fn residency(&self, key: &VisibilitySnapshotStateKey) -> VisibilityResidency {
        self.residency_key_lookups.fetch_add(1, Ordering::Relaxed);
        self.residency
            .read()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility residency lock poisoned")
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn update_residency(
        &self,
        key: &VisibilitySnapshotStateKey,
        update: impl FnOnce(&mut VisibilityResidency),
    ) {
        let mut residency = self
            .residency
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility residency lock poisoned");
        self.residency_key_lookups.fetch_add(1, Ordering::Relaxed);
        let entry = residency.entry(key.clone()).or_default();
        update(entry);
        self.residency_mutations.fetch_add(1, Ordering::Relaxed);
        if entry.branch_head_refs == 0
            && entry.replay_refs == 0
            && entry.active_snapshot_refs == 0
            && !entry.recent_resident
        {
            self.residency_key_lookups.fetch_add(1, Ordering::Relaxed);
            self.residency_mutations.fetch_add(1, Ordering::Relaxed);
            residency.remove(key);
        }
    }

    #[cfg(test)]
    pub(crate) fn cost_counters(&self) -> VisibilityCacheCostCounters {
        VisibilityCacheCostCounters {
            residency_entries: self
                .residency
                .read()
                .expect("visibility residency lock poisoned")
                .len() as u64,
            residency_key_lookups: self.residency_key_lookups.load(Ordering::Relaxed),
            residency_mutations: self.residency_mutations.load(Ordering::Relaxed),
            branch_head_population_scans: self.branch_head_population_scans.load(Ordering::Relaxed),
        }
    }

    pub(crate) fn recent_window(&self) -> usize {
        self.recent_policy
            .lock()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "recent visibility policy lock poisoned")
            .recent_version_window
    }

    pub(crate) fn resident_recent_count(&self) -> usize {
        self.recent_policy
            .lock()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "recent visibility policy lock poisoned")
            .resident_count
    }

    pub(crate) fn recent_candidate_count(&self) -> usize {
        self.recent_policy
            .lock()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "recent visibility policy lock poisoned")
            .order
            .len()
    }

    pub(crate) fn enqueue_recent_candidate(&self, key: VisibilitySnapshotStateKey) {
        self.recent_policy
            .lock()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "recent visibility policy lock poisoned")
            .order
            .push_back(key);
    }

    pub(crate) fn pop_oldest_recent_candidate(&self) -> Option<VisibilitySnapshotStateKey> {
        self.recent_policy
            .lock()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "recent visibility policy lock poisoned")
            .order
            .pop_front()
    }

    pub(crate) fn mark_recent_resident(&self, key: &VisibilitySnapshotStateKey) -> bool {
        {
            let mut residency = self
                .residency
                .write()
                .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility residency lock poisoned");
            let entry = residency.entry(key.clone()).or_default();
            if entry.recent_resident {
                return false;
            }
            entry.recent_resident = true;
        }
        let mut recent_policy = self
            .recent_policy
            .lock()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "recent visibility policy lock poisoned");
        recent_policy.order.push_back(key.clone());
        recent_policy.resident_count += 1;
        true
    }

    pub(crate) fn evict_recent_resident_if_unprotected(
        &self,
        key: &VisibilitySnapshotStateKey,
    ) -> bool {
        let mut residency = self
            .residency
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility residency lock poisoned");
        let Some(entry) = residency.get_mut(key) else {
            return false;
        };
        if !entry.recent_resident {
            return false;
        }
        if entry.branch_head_refs > 0 || entry.replay_refs > 0 || entry.active_snapshot_refs > 0 {
            return false;
        }
        entry.recent_resident = false;
        self.recent_policy
            .lock()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "recent visibility policy lock poisoned")
            .resident_count -= 1;
        if entry.branch_head_refs == 0 && entry.replay_refs == 0 && entry.active_snapshot_refs == 0
        {
            residency.remove(key);
        }
        true
    }

    pub(crate) fn clear_branch_head_residency(
        &self,
        tracked_states: &[VisibilitySnapshotStateKey],
    ) {
        self.branch_head_states
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "branch-head visibility state lock poisoned")
            .clear();
        let mut residency = self
            .residency
            .write()
            .expect(/* Proof: Visibility locks protect closed native maps/policy rows only; callbacks are owner-local reference-count updates, with no external code or I/O while a guard is held. */ "visibility residency lock poisoned");
        for key in tracked_states {
            if let Some(entry) = residency.get_mut(key) {
                entry.branch_head_refs = 0;
                if entry.replay_refs == 0
                    && entry.active_snapshot_refs == 0
                    && !entry.recent_resident
                {
                    residency.remove(key);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct VisibilityResidency {
    pub(crate) branch_head_refs: u32,
    pub(crate) replay_refs: u32,
    pub(crate) active_snapshot_refs: u32,
    pub(crate) recent_resident: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DeterministicVersionWindowPolicy {
    pub(crate) recent_version_window: usize,
    pub(crate) order: VecDeque<VisibilitySnapshotStateKey>,
    pub(crate) resident_count: usize,
}
