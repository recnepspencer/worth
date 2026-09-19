mod fork_growth;
use crate::data::graph::signal_graph::{EdgeTopology, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::node::{NodeColdData, NodeHotData, NodeWarmData};
use crate::data::output::PartitionInterner;
use crate::data::persistent_ord_map::PersistentOrdMap;
use crate::data::persistent_paged_vector::PersistentPagedVector;
use crate::diagnostics::state::DiagnosticsState;
mod fork_admission;
mod mutable_charge;
mod mutable_fork;
#[cfg(test)]
mod retained_charge;

use super::super::invalidation_causes::CanonicalCauseSetStore;

/// Derived evaluation roots shared by immutable seeds and mutable slots.
/// Only immutable seeds use Clone; mutable slots fork each persistent owner.
#[derive(Debug, Clone)]
pub(in crate::data::graph) struct SignalEvaluationStorage {
    hot: PersistentPagedVector<Option<NodeHotData>>,
    warm: PersistentPagedVector<NodeWarmData>,
    cold: PersistentPagedVector<Option<Box<NodeColdData>>>,
    retained_node_ledger:
        Option<std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionLedger>>,
    retained_node_custody: Option<
        std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    >,
    topology: EdgeTopology,
    compaction: crate::data::graph::compaction::CompactionState,
    causes: CanonicalCauseSetStore,
    cause_readmission_required: bool,
    conditional_versions: PersistentOrdMap<
        NodeId,
        crate::data::conditional_execution::SignalConditionalVersionObservation,
    >,
    conditional_versions_custody: Option<
        std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    >,
    repeated_admissions: PersistentOrdMap<NodeId, u64>,
    partitions: PartitionInterner,
    diagnostics: DiagnosticsState,
    // Drop custody only after every retained payload owned by this root.
    retained_seed_custody: Option<
        std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    >,
    // Inline candidate custody stays with this storage value, not the activated graph.
    fork_custody: Option<
        std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionReservation>,
    >,
}

impl SignalEvaluationStorage {
    pub(in crate::data::graph) fn prepare_persistent_fork_readiness(
        &mut self,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> Result<(), crate::data::retained_storage::RetainedStoragePreparationDenial> {
        use crate::data::retained_storage::RetainedStorageForkPreparation;

        self.hot.prepare_fork_charge(work)?;
        self.warm.prepare_fork_charge(work)?;
        self.cold.prepare_fork_charge(work)?;
        self.topology.prepare_fork_charge(work)?;
        self.causes.prepare_fork_charge(work)?;
        self.conditional_versions.prepare_fork_charge(work)?;
        self.repeated_admissions.prepare_fork_charge(work)?;
        self.partitions.prepare_fork_charge(work)?;
        self.diagnostics.prepare_retained_heap_charge(work)?;
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn retained_node_observation(
        &self,
        node: NodeId,
        aspect: crate::data::aspect::Aspect,
    ) -> Option<(crate::data::node::NodeState, u64, bool)> {
        let hot = self.hot.get(node.index() as usize)?.as_ref()?;
        Some((
            hot.state,
            hot.aspect_version_header.global().get(aspect),
            hot.dirty_aspects
                .contains(crate::data::aspect::AspectMask::from_aspect(aspect)),
        ))
    }

    /// Initial immutable seeds contain only the fixed diagnostic branch carrier.
    /// Prepare its cloned catalog during slot admission, never on a warm attempt.
    pub(super) fn prepare_seed_diagnostics(
        &mut self,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> Result<(), crate::data::retained_storage::RetainedStoragePreparationDenial> {
        self.diagnostics.prepare_retained_heap_charge(work)?;
        Ok(())
    }

    pub(super) fn retention_ledger(
        &self,
    ) -> Option<&std::sync::Arc<crate::data::retained_storage::SignalConditionalRetentionLedger>>
    {
        self.retained_node_ledger.as_ref()
    }

    pub(in crate::data::graph) fn has_retention_ledger(&self) -> bool {
        self.retained_node_ledger.is_some()
    }

    pub(in crate::data::graph) fn retain_seed_custody(
        &mut self,
        custody: std::sync::Arc<
            crate::data::retained_storage::SignalConditionalRetentionReservation,
        >,
    ) {
        self.retained_seed_custody = Some(custody);
    }

    pub(in crate::data::graph) fn preserve_issuance_from(&mut self, performed: &Self) {
        self.diagnostics
            .preserve_issuance_from(&performed.diagnostics);
        self.causes.preserve_output_issuance_from(&performed.causes);
    }

    pub(super) fn capture(
        graph: &mut SignalGraph,
        resources: &mut crate::data::retained_storage::SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            hot: graph.arena.hot.fork_reserved(resources),
            warm: graph.arena.warm.fork_reserved(resources),
            cold: graph.arena.cold.fork_reserved(resources),
            retained_node_ledger: Some(resources.ledger()),
            retained_node_custody: graph.arena.retained_node_custody.clone(),
            retained_seed_custody: graph.arena.retained_seed_custody.clone(),
            fork_custody: None,
            topology: graph.topology.fork_reserved(resources),
            compaction: graph.arena.compaction.clone(),
            causes: graph.cause_sets.fork_reserved(resources),
            cause_readmission_required: graph.cause_readmission_required,
            conditional_versions: graph
                .conditional_dependency_versions
                .fork_reserved(resources),
            conditional_versions_custody: graph.conditional_dependency_versions_custody.clone(),
            repeated_admissions: graph
                .pending_repeated_invalidation_admissions
                .fork_reserved(resources),
            partitions: graph
                .observation
                .partition_interner
                .fork_reserved(resources),
            diagnostics: graph.observation.diagnostics.fork_branch_carrier(),
        }
    }

    pub(in crate::data::graph) fn node_count(&self) -> usize {
        self.hot.len()
    }

    #[cfg(test)]
    pub(in crate::data::graph) fn diagnostics_for_test(&self) -> serde_json::Value {
        serde_json::to_value(&self.diagnostics).unwrap()
    }

    #[cfg(test)]
    pub(in crate::data::graph) fn retained_artifact_for_test(
        &self,
        node: NodeId,
    ) -> Option<&crate::data::trace::RetainedDiagnosticArtifact> {
        self.cold
            .get(node.index() as usize)?
            .as_deref()?
            .retained_artifact
            .as_ref()
    }

    pub(in crate::data::graph) fn exchange(&mut self, graph: &mut SignalGraph) {
        std::mem::swap(&mut self.hot, &mut graph.arena.hot);
        std::mem::swap(&mut self.warm, &mut graph.arena.warm);
        std::mem::swap(&mut self.cold, &mut graph.arena.cold);
        std::mem::swap(
            &mut self.retained_node_ledger,
            &mut graph.arena.retained_node_ledger,
        );
        std::mem::swap(
            &mut self.retained_node_custody,
            &mut graph.arena.retained_node_custody,
        );
        std::mem::swap(
            &mut self.retained_seed_custody,
            &mut graph.arena.retained_seed_custody,
        );
        std::mem::swap(&mut self.topology, &mut graph.topology);
        std::mem::swap(&mut self.compaction, &mut graph.arena.compaction);
        std::mem::swap(&mut self.causes, &mut graph.cause_sets);
        std::mem::swap(
            &mut self.cause_readmission_required,
            &mut graph.cause_readmission_required,
        );
        std::mem::swap(
            &mut self.conditional_versions,
            &mut graph.conditional_dependency_versions,
        );
        std::mem::swap(
            &mut self.conditional_versions_custody,
            &mut graph.conditional_dependency_versions_custody,
        );
        std::mem::swap(
            &mut self.repeated_admissions,
            &mut graph.pending_repeated_invalidation_admissions,
        );
        std::mem::swap(
            &mut self.partitions,
            &mut graph.observation.partition_interner,
        );
        std::mem::swap(&mut self.diagnostics, &mut graph.observation.diagnostics);
    }
}
