//! Concrete graph-read work for both checks of one prepared dependency cause.
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;
use crate::logic::evaluation::EvaluationWork;
impl SignalGraph {
    pub(crate) fn admit_cause_validation_reads(
        &self,
        node: NodeId,
        scope: Option<&PartitionSubscription>,
        additional_scopes: usize,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        // Each pass reads identity/revision, edges, snapshot, producer version,
        // and ledger. Include admission's own reads and bounded page growth as
        // earlier packet writes make selected nodes unique. No graph scan.
        let steps = [
            self.arena.nodes.lookup_steps(),
            self.arena.hot.lookup_steps(),
            self.arena.warm.lookup_steps(),
            crate::data::retained_storage::ordered_lookup_steps(self.arena.nodes.len()),
            self.topology.dependency_edges.lookup_steps(),
            self.topology.dependency_snapshots.lookup_steps(),
            self.cause_sets.published_output_lookup_steps(),
        ];
        work.reserve(
            steps
                .into_iter()
                .try_fold(64usize, |n, s| n.checked_add(s))
                .and_then(|n| n.checked_mul(64)),
        )?;
        let previous = self
            .warm_ref(node)?
            .aspect_version_overrides
            .lookup_work_bound(scope);
        let bytes = scope.map_or(Some(0), |s| {
            s.partition
                .0
                .len()
                .checked_add(s.detail.as_ref().map_or(0, String::len))
        });
        // The producer evaluation can add at most one partition and one detail
        // entry per region before the second validation.
        let added = bytes
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(8))
            .and_then(|n| n.checked_mul(additional_scopes))
            .and_then(|n| n.checked_mul(2));
        work.reserve(
            previous
                .and_then(|n| added.and_then(|a| n.checked_add(a)))
                .and_then(|n| n.checked_mul(2)),
        )
    }
}
