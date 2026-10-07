use crate::data::graph::runtime::strategy::{EvaluationStrategy, GcPressure, ObservationLevel};

use super::{EdgeTopology, NodeArena, RuntimeObservation, SignalGraph, TraversalResources};

impl SignalGraph {
    pub fn observe(&self) -> crate::data::graph::runtime::observer::GraphObserver<'_> {
        crate::data::graph::runtime::observer::GraphObserver::new(self)
    }

    pub fn derive_evaluation_strategy(&self) -> EvaluationStrategy {
        let active_nodes = self.active_node_count();
        let tombstone_ratio = self.tombstone_ratio();
        EvaluationStrategy {
            gc_pressure: if tombstone_ratio >= Self::GC_PRESSURE_TOMBSTONE_RATIO
                || self
                    .arena
                    .should_run_compaction_epoch(&self.topology, active_nodes)
            {
                GcPressure::CompactAfterEvaluation
            } else {
                GcPressure::Deferred
            },
            observation_level: if self.installed_runtime_policy().observation_activation()
                == worth_foundational::ObservationActivationProfile::OnDemand
            {
                ObservationLevel::Minimal
            } else {
                ObservationLevel::Full
            },
        }
    }

    pub(crate) fn as_parts_mut(
        &mut self,
    ) -> (
        &mut NodeArena,
        &mut EdgeTopology,
        &mut TraversalResources,
        &mut RuntimeObservation,
    ) {
        (
            &mut self.arena,
            &mut self.topology,
            &mut self.traversal,
            &mut self.observation,
        )
    }

    fn tombstone_ratio(&self) -> f32 {
        let active_nodes = self.active_node_count();
        let total = active_nodes + self.arena.compaction.tombstone_count as usize;
        if total == 0 {
            0.0
        } else {
            self.arena.compaction.tombstone_count as f32 / total as f32
        }
    }
}
