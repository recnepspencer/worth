//! Bounded source-root blob facts for a selected release's predecessor root.

use super::super::blob_walk::{BlobRecordWalk, HistoricalBlobSource};
use super::super::families::root_manifest::OfflineRootManifestFacts;
use super::super::{
    OfflineArtifactObservation, OfflineIntegrityObservationCounters,
    OfflineIntegrityOutcome as Outcome,
};
use super::{ChildExpectation, TraversalOrigin};
use worth_foundational::PhysicalArtifactGeneration;

pub(super) struct HistoricalBlobCollector {
    generation: Option<u64>,
    walk: Option<BlobRecordWalk>,
}

impl HistoricalBlobCollector {
    pub(super) fn new(
        roots: &[OfflineRootManifestFacts],
        current: Option<u64>,
        maximum_entries: u64,
        checkpoint: super::super::journal_walk::SelectedCheckpointEvidence,
    ) -> Self {
        let generation = current
            .and_then(|value| value.checked_sub(1))
            .filter(|prior| roots.iter().any(|root| root.generation == *prior));
        let walk = generation.map(|_| BlobRecordWalk::new(maximum_entries, checkpoint));
        Self { generation, walk }
    }

    fn matches(&self, origin: TraversalOrigin, generation: u64) -> bool {
        origin == TraversalOrigin::RootManifest && self.generation == Some(generation)
    }

    pub(super) fn note(
        &mut self,
        origin: TraversalOrigin,
        generation: u64,
        expected: &ChildExpectation,
        outcome: &Outcome,
    ) {
        if self.matches(origin, generation) {
            if let Some(walk) = self.walk.as_mut() {
                walk.note_extent_chunk_outcome(expected, outcome);
            }
        }
    }

    pub(super) fn observe(
        &mut self,
        origin: TraversalOrigin,
        generation: u64,
        expected: &ChildExpectation,
        bytes: &[u8],
        store: Option<[u8; 16]>,
        counters: &mut OfflineIntegrityObservationCounters,
    ) {
        if self.matches(origin, generation) {
            if let Some(walk) = self.walk.as_mut() {
                walk.observe_valid_extent_chunk(expected, bytes, store, counters);
            }
        }
    }

    pub(super) fn finish(
        self,
        arena_observations: &[OfflineArtifactObservation],
    ) -> Option<HistoricalBlobSource> {
        let generation = self.generation?;
        let encoded = PhysicalArtifactGeneration::encoded(generation)?;
        let routes_intact = arena_observations
            .iter()
            .filter(|observation| observation.generation() == encoded)
            .all(|observation| matches!(observation.outcome(), Outcome::Intact));
        Some(self.walk?.into_historical_source(generation, routes_intact))
    }
}
