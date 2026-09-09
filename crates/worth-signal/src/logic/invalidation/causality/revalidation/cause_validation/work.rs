//! Admit the successful pre-write check and its post-write consistency check.
use super::*;
use crate::logic::evaluation::EvaluationWork;
impl SignalGraph {
    pub(super) fn admit_prepared_cause_validation(
        &self,
        consumer: NodeId,
        cause: &ResolvedDependencyCause,
        prepared: &ProducedAspectDelta,
        regions: usize,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        if matches!(work, EvaluationWork::Ordinary) {
            return Ok(());
        }
        let scope = cause.key.edge_scope.as_ref();
        self.admit_cause_validation_reads(
            cause.key.producer,
            scope,
            if cause.key.producer == prepared.producer {
                regions
            } else {
                0
            },
            work,
        )?;
        let bytes = scope.map_or(Some(0), |s| {
            s.partition
                .0
                .len()
                .checked_add(s.detail.as_ref().map_or(0, String::len))
        });
        let comparison = bytes
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(64));
        let edges = self.current_runtime_dependencies_of(consumer)?.len();
        let snapshots = self.get_dep_snapshot(consumer)?.entries().len();
        // Identity scope equality, edge search, snapshot search, final scope
        // equality and fixed-axis checks, each before and after publication.
        work.reserve(
            edges
                .checked_add(snapshots)
                .and_then(|n| n.checked_add(8))
                .and_then(|n| comparison.and_then(|c| n.checked_mul(c)))
                .and_then(|n| n.checked_mul(2)),
        )?;
        let ordinal = cause.binding_axes.output_commit_ordinal;
        let delta = if ordinal == prepared.output_commit_ordinal {
            Some(prepared)
        } else {
            self.cause_sets.published_output_commit(ordinal)
        };
        if let Some(delta) = delta {
            work.reserve(delta.changes.as_slice().len().checked_mul(2))?;
            for change in delta.changes.as_slice() {
                work.reserve(
                    change
                        .changed_scopes
                        .len()
                        .checked_add(1)
                        .and_then(|n| comparison.and_then(|c| n.checked_mul(c)))
                        .and_then(|n| n.checked_mul(2)),
                )?;
            }
        }
        Ok(())
    }
}
