use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::shared_read_pins::WorthQuerySharedReadGenerationId;
use crate::runtime::WorthQueryDerivedMaterializationTarget;

use super::{
    WorthQueryPublishedArtifactCounters, WorthQueryPublishedArtifactDiagnostics,
    WorthQueryPublishedArtifactEntry, WorthQueryPublishedArtifactGenerationDiagnostic,
    WorthQueryPublishedArtifactResolution,
};

#[derive(Clone, Debug, Default)]
pub(in crate::runtime) struct WorthQueryPublishedArtifactRegistry {
    generations: Arc<ArcSwap<WorthQueryPublishedArtifactGenerations>>,
    counters: WorthQueryPublishedArtifactCounters,
}

impl WorthQueryPublishedArtifactRegistry {
    pub(in crate::runtime) fn publish_generation(
        &self,
        generation: &WorthQuerySharedReadGenerationId,
        entries: BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryPublishedArtifactEntry>,
    ) {
        let mut generations = self.generations.load_full().as_ref().clone();
        generations.publish_generation(generation, entries);
        self.generations.store(Arc::new(generations));
    }

    pub(in crate::runtime) fn retain_generations(&self, retained_ordinals: &BTreeSet<u64>) {
        let mut generations = self.generations.load_full().as_ref().clone();
        let dropped = generations.retain_generations(retained_ordinals);
        self.counters.record_dropped_generations(dropped);
        self.generations.store(Arc::new(generations));
    }

    pub(in crate::runtime) fn resolve(
        &self,
        generation: &WorthQuerySharedReadGenerationId,
        target: &WorthQueryDerivedMaterializationTarget,
    ) -> WorthQueryPublishedArtifactResolution {
        let resolution = self.generations.load().resolve(generation, target);
        if matches!(
            resolution,
            WorthQueryPublishedArtifactResolution::Published { .. }
        ) {
            self.counters.record_registry_lease();
        }
        resolution
    }

    pub(in crate::runtime) fn counters(&self) -> WorthQueryPublishedArtifactCounters {
        self.counters.clone()
    }

    pub(in crate::runtime) fn diagnostics(&self) -> WorthQueryPublishedArtifactDiagnostics {
        WorthQueryPublishedArtifactDiagnostics::new(
            self.counters.snapshot(),
            self.generations.load().generation_diagnostics(),
        )
    }
}

#[derive(Clone, Debug, Default)]
struct WorthQueryPublishedArtifactGenerations {
    generations: BTreeMap<u64, WorthQueryPublishedArtifactGeneration>,
}

impl WorthQueryPublishedArtifactGenerations {
    fn publish_generation(
        &mut self,
        generation: &WorthQuerySharedReadGenerationId,
        entries: BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryPublishedArtifactEntry>,
    ) {
        self.generations.insert(
            generation.ordinal(),
            WorthQueryPublishedArtifactGeneration::new(
                generation.ordinal(),
                generation.snapshot_identity().clone(),
                entries,
            ),
        );
    }

    fn retain_generations(&mut self, retained_ordinals: &BTreeSet<u64>) -> usize {
        let before = self.generations.len();
        self.generations
            .retain(|ordinal, _| retained_ordinals.contains(ordinal));
        before.saturating_sub(self.generations.len())
    }

    fn resolve(
        &self,
        generation: &WorthQuerySharedReadGenerationId,
        target: &WorthQueryDerivedMaterializationTarget,
    ) -> WorthQueryPublishedArtifactResolution {
        let Some(published_generation) = self.generations.get(&generation.ordinal()) else {
            return WorthQueryPublishedArtifactResolution::MissingGeneration;
        };
        published_generation.resolve(generation, target)
    }

    fn generation_diagnostics(&self) -> Vec<WorthQueryPublishedArtifactGenerationDiagnostic> {
        self.generations
            .values()
            .map(WorthQueryPublishedArtifactGeneration::diagnostic)
            .collect()
    }
}

#[derive(Clone, Debug)]
struct WorthQueryPublishedArtifactGeneration {
    ordinal: u64,
    snapshot_identity: WorthQuerySnapshotIdentity,
    entries: BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryPublishedArtifactEntry>,
}

impl WorthQueryPublishedArtifactGeneration {
    fn new(
        ordinal: u64,
        snapshot_identity: WorthQuerySnapshotIdentity,
        entries: BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryPublishedArtifactEntry>,
    ) -> Self {
        Self {
            ordinal,
            snapshot_identity,
            entries,
        }
    }

    fn resolve(
        &self,
        generation: &WorthQuerySharedReadGenerationId,
        target: &WorthQueryDerivedMaterializationTarget,
    ) -> WorthQueryPublishedArtifactResolution {
        if self.ordinal != generation.ordinal()
            || !self
                .snapshot_identity
                .is_same_current_identity_as(generation.snapshot_identity())
        {
            return WorthQueryPublishedArtifactResolution::MissingGeneration;
        }
        let Some(entry) = self.entries.get(target) else {
            return WorthQueryPublishedArtifactResolution::MissingView;
        };
        if entry.target() != target {
            return WorthQueryPublishedArtifactResolution::MissingView;
        }
        match entry.published_binding() {
            Some(binding) => WorthQueryPublishedArtifactResolution::Published {
                binding,
                async_result_state: entry.async_result_state(),
            },
            None => WorthQueryPublishedArtifactResolution::Unpublished {
                async_result_state: entry
                    .async_result_state()
                    .expect("unpublished artifact entries carry async posture"),
            },
        }
    }

    fn diagnostic(&self) -> WorthQueryPublishedArtifactGenerationDiagnostic {
        WorthQueryPublishedArtifactGenerationDiagnostic::new(
            self.ordinal,
            self.snapshot_identity.clone(),
            self.entries.len(),
        )
    }
}
#[cfg(test)]
#[path = "registry_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
