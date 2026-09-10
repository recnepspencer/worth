//! Work for dependency capture copies, comparisons and selected version reads.
use crate::data::dependency::{DependencyEdge, DependencySnapshotEntry};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::output::PartitionSubscription;
use crate::logic::evaluation::EvaluationWork;

fn scope_bytes(scope: Option<&PartitionSubscription>) -> Option<usize> {
    scope.map_or(Some(0), |s| {
        s.partition
            .0
            .len()
            .checked_add(s.detail.as_ref().map_or(0, String::len))
    })
}

pub(super) fn scope_copy(
    scope: Option<&PartitionSubscription>,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(scope_bytes(scope).and_then(|n| n.checked_add(8)))
}

pub(super) fn scope_comparison(
    scope: Option<&PartitionSubscription>,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(
        scope_bytes(scope)
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(16)),
    )
}

pub(super) fn dependencies(
    dependencies: &[DependencyEdge],
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(
        dependencies
            .len()
            .checked_mul(std::mem::size_of::<DependencyEdge>() + 1)
            .filter(|n| *n <= isize::MAX as usize),
    )?;
    for dependency in dependencies {
        scope_copy(dependency.scope_ref(), work)?;
    }
    Ok(())
}

pub(super) fn version(
    graph: &SignalGraph,
    dependency: &DependencyEdge,
    work: &mut EvaluationWork<'_>,
) -> Result<u64, SignalError> {
    match work {
        EvaluationWork::Ordinary => graph.node_version_for_scope(
            dependency.source(),
            dependency.aspect(),
            dependency.scope_ref(),
        ),
        EvaluationWork::Conditional(work) => graph.conditional_node_version_for_scope(
            dependency.source(),
            dependency.aspect(),
            dependency.scope_ref(),
            work,
        ),
    }
}

pub(super) fn snapshot_comparison(
    left: &[DependencySnapshotEntry],
    right: &[DependencySnapshotEntry],
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    let count = left.len().checked_add(right.len());
    work.reserve(count)?;
    let mut largest = 0;
    for entry in left.iter().chain(right) {
        let bytes = scope_bytes(entry.scope.as_ref());
        work.reserve(bytes.map(|_| 0))?;
        largest = largest.max(bytes.expect("checked scope size"));
    }
    // A merge comparison advances at least one cursor; canonical validation
    // compares adjacent entries. Bound both operand byte reads per comparison.
    work.reserve(count.and_then(|n| n.checked_mul(largest.checked_mul(2)?.checked_add(16)?)))
}
