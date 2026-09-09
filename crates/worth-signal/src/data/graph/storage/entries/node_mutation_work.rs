//! Existing-slot container writes in an exclusive output publication packet.
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::logic::evaluation::EvaluationWork;

impl SignalGraph {
    pub(crate) fn admit_effect_node_mutation_work(
        &self,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        // Producer: version, artifact, cause release/cache, clean/deferred
        // state, snapshot ID and final waiter projection. Cold storage also
        // permits causality and both empty-companion trims. These bounds cover
        // every verdict, including absent artifacts and pending cause sets.
        self.admit_node_container_writes(8, 8, 4, work)
    }

    pub(crate) fn admit_downstream_node_mutation_work(
        &self,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        // At most one cause replacement (handle/cache/state) and one waiter
        // projection per node in the deduplicated prepared resolution map.
        self.admit_node_container_writes(4, 2, 0, work)
    }

    fn admit_node_container_writes(
        &self,
        hot: usize,
        warm: usize,
        cold: usize,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        if matches!(work, EvaluationWork::Ordinary) {
            return Ok(());
        }
        let writes = hot.checked_add(warm).and_then(|n| n.checked_add(cold));
        // Each mutator validates its NodeId. Include the admission's own
        // selected-node reads before payload-copy admission starts.
        let handles = writes
            .and_then(|n| n.checked_add(16))
            .and_then(|n| n.checked_mul(self.arena.nodes.lookup_steps() + 2));
        let bounds = [
            handles,
            self.arena.hot.indexed_mutation_work_bound(hot),
            self.arena.warm.indexed_mutation_work_bound(warm),
            self.arena.cold.indexed_mutation_work_bound(cold),
        ];
        work.reserve(
            bounds
                .into_iter()
                .try_fold(0usize, |n, b| n.checked_add(b?)),
        )
    }
}
