use std::collections::{BTreeMap, BTreeSet};

use worth_relational::facade::identity::{EntityId, VersionId};

use super::{
    WorkflowInstanceProgress, WorkflowTransitionReplayProjection, WorkflowTransitionReplayRetention,
};

#[path = "retention/counters.rs"]
mod counters;
pub use counters::WorthQueryWorkflowInstanceProgressCounters;

pub(super) const SHARD_COUNT: usize = 16;
const TOTAL_RETAINED_CHARGE_BUDGET: usize = 16 * 1024 * 1024;

pub(in crate::domain_computation::primary_graph) fn default_progress_retention_shards(
) -> std::sync::Arc<[std::sync::Mutex<WorkflowInstanceProgressRetention>]> {
    (0..SHARD_COUNT)
        .map(|_| std::sync::Mutex::new(WorkflowInstanceProgressRetention::default()))
        .collect::<Vec<_>>()
        .into()
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WorkflowInstanceProgressKey {
    branch_occurrence: u64,
    instance: EntityId,
    definition: EntityId,
}

impl WorkflowInstanceProgressKey {
    pub(in crate::domain_computation::primary_graph) const fn new(
        branch_occurrence: u64,
        instance: EntityId,
        definition: EntityId,
    ) -> Self {
        Self {
            branch_occurrence,
            instance,
            definition,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn shard_index(self) -> usize {
        let mixed = self.branch_occurrence.wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ self.instance.local_slot_value()
            ^ self.definition.local_slot_value().rotate_left(23);
        mixed as usize % SHARD_COUNT
    }
}

#[derive(Clone)]
struct RetainedWorkflowInstanceProgress {
    revision: Option<VersionId>,
    progress: WorkflowInstanceProgress,
    replays: WorkflowTransitionReplayRetention,
    retained_charge_bytes: usize,
    last_use: u64,
}

pub(in crate::domain_computation::primary_graph) struct RetainedWorkflowInstanceProgressProjection {
    pub(in crate::domain_computation::primary_graph) progress: WorkflowInstanceProgress,
    pub(in crate::domain_computation::primary_graph) replays: WorkflowTransitionReplayRetention,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorkflowInstanceProgressRetentionDenial {
    RevisionCollision,
    ContinuityUnavailable,
    ByteBudgetExceeded,
}

pub(in crate::domain_computation::primary_graph) struct WorkflowInstanceProgressRetention {
    maximum_retained_charge_bytes: usize,
    retained_charge_bytes: usize,
    use_sequence: u64,
    entries: BTreeMap<WorkflowInstanceProgressKey, RetainedWorkflowInstanceProgress>,
    least_recently_used: BTreeSet<(u64, WorkflowInstanceProgressKey)>,
    counters: WorthQueryWorkflowInstanceProgressCounters,
}

impl WorkflowInstanceProgressRetention {
    #[cfg(test)]
    fn new(maximum_retained_charge_bytes: usize) -> Self {
        Self::with_budget(maximum_retained_charge_bytes)
    }

    fn with_budget(maximum_retained_charge_bytes: usize) -> Self {
        Self {
            maximum_retained_charge_bytes,
            retained_charge_bytes: 0,
            use_sequence: 0,
            entries: BTreeMap::new(),
            least_recently_used: BTreeSet::new(),
            counters: WorthQueryWorkflowInstanceProgressCounters {
                maximum_retained_charge_bytes,
                ..Default::default()
            },
        }
    }

    pub(in crate::domain_computation::primary_graph) fn reuse(
        &mut self,
        key: WorkflowInstanceProgressKey,
        revision: Option<VersionId>,
    ) -> Option<RetainedWorkflowInstanceProgressProjection> {
        let matches = self
            .entries
            .get(&key)
            .is_some_and(|entry| entry.revision == revision);
        if !matches {
            self.counters.cold_misses = self.counters.cold_misses.saturating_add(1);
            return None;
        }
        let last_use = self.next_use_sequence();
        let retained = self
            .entries
            .get_mut(&key)
            .expect("matching retained workflow progress remains present");
        self.least_recently_used.remove(&(retained.last_use, key));
        retained.last_use = last_use;
        self.least_recently_used.insert((last_use, key));
        self.counters.warm_hits = self.counters.warm_hits.saturating_add(1);
        Some(RetainedWorkflowInstanceProgressProjection {
            progress: retained.progress.clone(),
            replays: retained.replays.clone(),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn retain(
        &mut self,
        key: WorkflowInstanceProgressKey,
        revision: Option<VersionId>,
        progress: WorkflowInstanceProgress,
        replays: WorkflowTransitionReplayRetention,
        reconstruction_transition_visits: usize,
    ) -> Result<(), WorkflowInstanceProgressRetentionDenial> {
        if self
            .entries
            .get(&key)
            .is_some_and(|retained| retained.revision == revision && retained.progress != progress)
        {
            self.counters.denials = self.counters.denials.saturating_add(1);
            return Err(WorkflowInstanceProgressRetentionDenial::RevisionCollision);
        }
        if self
            .entries
            .get(&key)
            .is_some_and(|retained| retained.revision > revision)
        {
            self.counters.denials = self.counters.denials.saturating_add(1);
            return Err(WorkflowInstanceProgressRetentionDenial::RevisionCollision);
        }
        self.store(key, revision, progress, replays)?;
        self.counters.cold_retains = self.counters.cold_retains.saturating_add(1);
        self.counters.cold_reconstruction_transition_visits = self
            .counters
            .cold_reconstruction_transition_visits
            .saturating_add(reconstruction_transition_visits);
        self.counters.retained_charge_bytes = self.retained_charge_bytes;
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn observe_warm_core(&mut self) {
        self.counters.warm_core_hits = self.counters.warm_core_hits.saturating_add(1);
    }

    pub(in crate::domain_computation::primary_graph) fn observe_warm_history(
        &mut self,
        transition_visits: usize,
    ) {
        self.counters.warm_history_transition_visits = self
            .counters
            .warm_history_transition_visits
            .saturating_add(transition_visits);
    }

    pub(in crate::domain_computation::primary_graph) fn advance(
        &mut self,
        key: WorkflowInstanceProgressKey,
        source_revision: Option<VersionId>,
        source: &WorkflowInstanceProgress,
        committed_revision: Option<VersionId>,
        advanced: WorkflowInstanceProgress,
        replay: WorkflowTransitionReplayProjection,
    ) -> Result<(), WorkflowInstanceProgressRetentionDenial> {
        let Some(retained) = self.entries.get(&key) else {
            self.counters.incremental_misses = self.counters.incremental_misses.saturating_add(1);
            return Err(WorkflowInstanceProgressRetentionDenial::ContinuityUnavailable);
        };
        if retained.revision == committed_revision && retained.progress == advanced {
            self.touch(key);
            self.counters.incremental_replays = self.counters.incremental_replays.saturating_add(1);
            return Ok(());
        }
        if retained.revision != source_revision || &retained.progress != source {
            self.counters.denials = self.counters.denials.saturating_add(1);
            return Err(WorkflowInstanceProgressRetentionDenial::RevisionCollision);
        }
        if committed_revision <= source_revision {
            self.counters.denials = self.counters.denials.saturating_add(1);
            return Err(WorkflowInstanceProgressRetentionDenial::RevisionCollision);
        }
        let advanced_replays = retained.replays.append(replay);
        let advanced_charge = retained_charge_bytes(&advanced, &advanced_replays);
        if advanced_charge > self.maximum_retained_charge_bytes {
            self.counters.denials = self.counters.denials.saturating_add(1);
            return Err(WorkflowInstanceProgressRetentionDenial::ByteBudgetExceeded);
        }
        let retained = self
            .entries
            .remove(&key)
            .expect("continuous retained workflow progress remains present");
        self.least_recently_used.remove(&(retained.last_use, key));
        self.retained_charge_bytes = self
            .retained_charge_bytes
            .saturating_sub(retained.retained_charge_bytes);
        self.store(key, committed_revision, advanced, advanced_replays)?;
        self.counters.incremental_advances = self.counters.incremental_advances.saturating_add(1);
        Ok(())
    }

    fn store(
        &mut self,
        key: WorkflowInstanceProgressKey,
        revision: Option<VersionId>,
        progress: WorkflowInstanceProgress,
        replays: WorkflowTransitionReplayRetention,
    ) -> Result<(), WorkflowInstanceProgressRetentionDenial> {
        let retained_charge_bytes = retained_charge_bytes(&progress, &replays);
        if retained_charge_bytes > self.maximum_retained_charge_bytes {
            self.counters.denials = self.counters.denials.saturating_add(1);
            return Err(WorkflowInstanceProgressRetentionDenial::ByteBudgetExceeded);
        }
        if let Some(displaced) = self.entries.remove(&key) {
            self.least_recently_used.remove(&(displaced.last_use, key));
            self.retained_charge_bytes = self
                .retained_charge_bytes
                .saturating_sub(displaced.retained_charge_bytes);
        }
        self.evict_until_available(retained_charge_bytes);
        let last_use = self.next_use_sequence();
        self.entries.insert(
            key,
            RetainedWorkflowInstanceProgress {
                revision,
                progress,
                replays,
                retained_charge_bytes,
                last_use,
            },
        );
        self.least_recently_used.insert((last_use, key));
        self.retained_charge_bytes = self
            .retained_charge_bytes
            .saturating_add(retained_charge_bytes);
        self.counters.retained_charge_bytes = self.retained_charge_bytes;
        Ok(())
    }

    fn touch(&mut self, key: WorkflowInstanceProgressKey) {
        let last_use = self.next_use_sequence();
        let retained = self
            .entries
            .get_mut(&key)
            .expect("retained workflow progress remains present");
        self.least_recently_used.remove(&(retained.last_use, key));
        retained.last_use = last_use;
        self.least_recently_used.insert((last_use, key));
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn replays(
        &mut self,
        key: WorkflowInstanceProgressKey,
        revision: Option<VersionId>,
    ) -> Option<WorkflowTransitionReplayRetention> {
        let retained = self.entries.get(&key)?;
        if retained.revision != revision {
            return None;
        }
        Some(retained.replays.clone())
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(in crate::domain_computation::primary_graph) fn release_all(&mut self) {
        self.counters.releases = self.counters.releases.saturating_add(self.entries.len());
        self.entries.clear();
        self.least_recently_used.clear();
        self.retained_charge_bytes = 0;
        self.counters.retained_charge_bytes = 0;
    }

    pub(in crate::domain_computation::primary_graph) fn release_branch(
        &mut self,
        branch_occurrence: u64,
    ) {
        let keys = self
            .entries
            .keys()
            .copied()
            .filter(|key| key.branch_occurrence == branch_occurrence)
            .collect::<Vec<_>>();
        let released = keys.len();
        for key in keys {
            let removed = self
                .entries
                .remove(&key)
                .expect("selected branch progress remains present");
            self.least_recently_used.remove(&(removed.last_use, key));
            self.retained_charge_bytes = self
                .retained_charge_bytes
                .saturating_sub(removed.retained_charge_bytes);
        }
        self.counters.releases = self.counters.releases.saturating_add(released);
        self.counters.retained_charge_bytes = self.retained_charge_bytes;
    }

    fn evict_until_available(&mut self, required: usize) {
        while self.retained_charge_bytes.saturating_add(required)
            > self.maximum_retained_charge_bytes
        {
            let Some((last_use, key)) = self.least_recently_used.pop_first() else {
                break;
            };
            let removed = self
                .entries
                .remove(&key)
                .expect("selected retained progress remains present");
            debug_assert_eq!(removed.last_use, last_use);
            self.retained_charge_bytes = self
                .retained_charge_bytes
                .saturating_sub(removed.retained_charge_bytes);
            self.counters.evictions = self.counters.evictions.saturating_add(1);
        }
    }

    fn next_use_sequence(&mut self) -> u64 {
        self.use_sequence = self.use_sequence.saturating_add(1);
        self.use_sequence
    }

    pub(in crate::domain_computation::primary_graph) const fn counters(
        &self,
    ) -> WorthQueryWorkflowInstanceProgressCounters {
        self.counters
    }
}

fn retained_charge_bytes(
    progress: &WorkflowInstanceProgress,
    replays: &WorkflowTransitionReplayRetention,
) -> usize {
    std::mem::size_of::<WorkflowInstanceProgressKey>()
        .saturating_add(std::mem::size_of::<RetainedWorkflowInstanceProgress>())
        .saturating_add(progress.retained_charge_bytes())
        .saturating_add(replays.retained_charge_bytes())
}

impl Default for WorkflowInstanceProgressRetention {
    fn default() -> Self {
        Self::with_budget(TOTAL_RETAINED_CHARGE_BUDGET / SHARD_COUNT)
    }
}

#[cfg(test)]
#[path = "retention/tests.rs"]
mod tests;
