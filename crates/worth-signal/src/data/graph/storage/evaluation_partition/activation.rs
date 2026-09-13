use crate::data::error::SignalError;
use crate::data::graph::signal_graph::{BranchMutationRecord, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::persistent_ord_map::PersistentOrdMap;

use super::SignalEvaluationPartition;

/// Internal storage custody. The service supplies owner-admitted execution;
/// this mechanism cannot be reached through an audience facade.
struct ActiveEvaluationStorage<'a> {
    graph: &'a mut SignalGraph,
    retained: &'a mut SignalEvaluationPartition,
    mutation_view: PersistentOrdMap<NodeId, BranchMutationRecord>,
    mutation_records: PersistentOrdMap<NodeId, BranchMutationRecord>,
}

impl SignalEvaluationPartition {
    pub(crate) fn execute<R>(
        &mut self,
        graph: &mut SignalGraph,
        execute: impl FnOnce(&mut SignalGraph) -> R,
    ) -> Result<R, SignalError> {
        if graph.traversal.scratch_lease.is_some() || self.traversal.scratch_lease.is_some() {
            return Err(SignalError::invalid_input(
                "evaluation activation requires released traversal scratch",
            ));
        }
        if !self.definitions.matches_storage_lineage(graph) {
            return Err(SignalError::invalid_input(
                "evaluation storage belongs to another storage lineage",
            ));
        }
        if self.evaluation.node_count() != self.definitions.node_count() {
            return Err(SignalError::invalid_input(
                "evaluation storage does not match selected definition shape",
            ));
        }
        self.exchange(graph);
        let active = ActiveEvaluationStorage {
            mutation_view: std::mem::take(&mut graph.observation.branch_mutation_view),
            mutation_records: std::mem::take(&mut graph.observation.branch_mutation_records),
            graph,
            retained: self,
        };
        Ok(execute(active.graph))
    }
}

impl Drop for ActiveEvaluationStorage<'_> {
    fn drop(&mut self) {
        self.retained.exchange(self.graph);
        // Restore authoritative custody before dropping
        // derived mutation records, including during provider unwind.
        std::mem::swap(
            &mut self.mutation_view,
            &mut self.graph.observation.branch_mutation_view,
        );
        std::mem::swap(
            &mut self.mutation_records,
            &mut self.graph.observation.branch_mutation_records,
        );
    }
}
