use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::memory_workspace::WorthQuerySnapshotIdentity;

use super::WorthQuerySharedReadCounters;
use super::{
    collect_retired_zero_pin_generations, WorthQuerySharedReadCurrentGeneration,
    WorthQuerySharedReadGenerationDiagnostic, WorthQuerySharedReadGenerationEntry,
    WorthQuerySharedReadGenerationId, WorthQuerySharedReadGenerationLease,
    WorthQuerySharedReadHotPathMeasurement, WorthQuerySharedReadPinnedSnapshot,
    WorthQuerySharedReadPinningDiagnostics,
};

#[derive(Clone, Debug)]
pub(in crate::runtime) struct WorthQuerySharedReadPinRegistry {
    state: Arc<Mutex<WorthQuerySharedReadPinRegistryState>>,
    current_generation: Arc<WorthQuerySharedReadCurrentGeneration>,
    hot_path_measurement: WorthQuerySharedReadHotPathMeasurement,
}

impl Default for WorthQuerySharedReadPinRegistry {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(WorthQuerySharedReadPinRegistryState::default())),
            current_generation: Arc::new(WorthQuerySharedReadCurrentGeneration::default()),
            hot_path_measurement: WorthQuerySharedReadHotPathMeasurement::default(),
        }
    }
}

impl WorthQuerySharedReadPinRegistry {
    pub(in crate::runtime) fn capture_committed_snapshot(
        &self,
        snapshot_identity: WorthQuerySnapshotIdentity,
    ) -> WorthQuerySharedReadGenerationId {
        let mut state = self.state.lock().expect("shared-read pin registry lock");
        state.capture_committed_snapshot(snapshot_identity, &self.current_generation)
    }

    pub(in crate::runtime) fn pin_current_generation(
        &self,
    ) -> Option<WorthQuerySharedReadGenerationLease> {
        loop {
            let entry = self.current_generation.load()?;
            entry.pin();
            if !entry.is_invalidated() {
                return Some(WorthQuerySharedReadGenerationLease::new(
                    self.clone(),
                    entry,
                ));
            }
            let remaining_pin_count = entry.release_pin();
            if entry.is_retired() && remaining_pin_count == 0 {
                self.drain_retired_generation();
            }
        }
    }

    pub(in crate::runtime) fn has_current_generation(&self) -> bool {
        self.current_generation
            .load()
            .is_some_and(|entry| !entry.is_retired())
    }

    pub(in crate::runtime) fn release_generation(
        &self,
        entry: Arc<WorthQuerySharedReadGenerationEntry>,
    ) {
        let remaining_pin_count = entry.release_pin();
        if !(entry.is_retired() && remaining_pin_count == 0) {
            return;
        }
        self.drain_retired_generation();
    }

    pub(in crate::runtime) fn drain_retired_generation(&self) {
        let mut state = self.state.lock().expect("shared-read pin registry lock");
        state.collect_retired_zero_pin_generations();
    }

    pub(in crate::runtime) fn counters(&self) -> WorthQuerySharedReadCounters {
        let state = self.state.lock().expect("shared-read pin registry lock");
        state.counters(
            self.hot_path_measurement
                .committed_read_hot_path_lock_count(),
        )
    }

    pub(in crate::runtime) fn diagnostics(&self) -> WorthQuerySharedReadPinningDiagnostics {
        let state = self.state.lock().expect("shared-read pin registry lock");
        let counters = state.counters(
            self.hot_path_measurement
                .committed_read_hot_path_lock_count(),
        );
        WorthQuerySharedReadPinningDiagnostics::new(counters, state.generation_diagnostics())
    }

    pub(in crate::runtime) fn retained_generation_ordinals(&self) -> BTreeSet<u64> {
        let state = self.state.lock().expect("shared-read pin registry lock");
        state.retained_generation_ordinals()
    }

    pub(in crate::runtime) fn record_committed_read_hot_path_lock_for_certification(&self) {
        self.hot_path_measurement
            .record_committed_read_hot_path_lock();
    }

    pub(in crate::runtime) fn force_retire_snapshot_identity(
        &self,
        snapshot_identity: &WorthQuerySnapshotIdentity,
    ) {
        let mut state = self.state.lock().expect("shared-read pin registry lock");
        state.force_retire_snapshot_identity(snapshot_identity, &self.current_generation);
    }
}

#[derive(Debug, Default)]
struct WorthQuerySharedReadPinRegistryState {
    next_generation_ordinal: u64,
    current_generation_ordinal: Option<u64>,
    current_snapshot_identity: Option<WorthQuerySnapshotIdentity>,
    generations: BTreeMap<u64, Arc<WorthQuerySharedReadGenerationEntry>>,
}

impl WorthQuerySharedReadPinRegistryState {
    fn capture_committed_snapshot(
        &mut self,
        snapshot_identity: WorthQuerySnapshotIdentity,
        current_generation: &WorthQuerySharedReadCurrentGeneration,
    ) -> WorthQuerySharedReadGenerationId {
        if self
            .current_snapshot_identity
            .as_ref()
            .is_some_and(|current| current.is_same_current_identity_as(&snapshot_identity))
        {
            let ordinal = self
                .current_generation_ordinal
                .expect("current snapshot identity requires current generation");
            return self
                .generations
                .get(&ordinal)
                .expect("current generation should exist")
                .snapshot()
                .generation()
                .clone();
        }

        if let Some(current) = self.current_generation_ordinal
            && let Some(entry) = self.generations.get(&current)
        {
            entry.retire();
        }

        self.next_generation_ordinal += 1;
        let generation =
            WorthQuerySharedReadGenerationId::new(self.next_generation_ordinal, snapshot_identity);
        let ordinal = generation.ordinal();
        let snapshot = WorthQuerySharedReadPinnedSnapshot::new(generation.clone());
        self.current_snapshot_identity = Some(snapshot.generation().snapshot_identity().clone());
        self.current_generation_ordinal = Some(ordinal);
        let entry = Arc::new(WorthQuerySharedReadGenerationEntry::new(snapshot));
        current_generation.publish(Arc::clone(&entry));
        self.generations.insert(ordinal, entry);
        self.collect_retired_zero_pin_generations();
        generation
    }

    fn collect_retired_zero_pin_generations(&mut self) {
        collect_retired_zero_pin_generations(&mut self.generations);
    }

    fn counters(&self, committed_read_hot_path_lock_count: usize) -> WorthQuerySharedReadCounters {
        let orphaned_generation_count = self
            .generations
            .values()
            .filter(|entry| entry.is_retired() && entry.pin_count() == 0)
            .count();
        let unretired_pin_count = self
            .generations
            .values()
            .map(|entry| entry.pin_count())
            .sum();
        WorthQuerySharedReadCounters::new(
            committed_read_hot_path_lock_count,
            orphaned_generation_count,
            unretired_pin_count,
        )
    }

    fn retained_generation_ordinals(&self) -> BTreeSet<u64> {
        self.generations.keys().copied().collect()
    }

    fn generation_diagnostics(&self) -> Vec<WorthQuerySharedReadGenerationDiagnostic> {
        self.generations
            .iter()
            .map(|(ordinal, entry)| {
                WorthQuerySharedReadGenerationDiagnostic::new(
                    *ordinal,
                    entry.snapshot().generation().snapshot_identity().clone(),
                    self.current_generation_ordinal == Some(*ordinal),
                    entry.is_retired(),
                    entry.is_invalidated(),
                    entry.pin_count(),
                )
            })
            .collect()
    }

    fn force_retire_snapshot_identity(
        &mut self,
        snapshot_identity: &WorthQuerySnapshotIdentity,
        current_generation: &WorthQuerySharedReadCurrentGeneration,
    ) {
        let affected = self
            .generations
            .iter()
            .filter_map(|(ordinal, entry)| {
                entry
                    .snapshot()
                    .generation()
                    .snapshot_identity()
                    .is_same_current_identity_as(snapshot_identity)
                    .then_some(*ordinal)
            })
            .collect::<Vec<_>>();
        for ordinal in affected {
            let Some(entry) = self.generations.get(&ordinal) else {
                continue;
            };
            entry.invalidate();
            if self.current_generation_ordinal == Some(ordinal) {
                current_generation.clear_if_generation(entry);
                self.current_generation_ordinal = None;
                self.current_snapshot_identity = None;
            }
        }
        self.collect_retired_zero_pin_generations();
    }
}
