//! Native retained-path edits consume accumulated work on every exit.
use super::super::super::request_execution::QueryRequestExecution;
use super::super::WorthQueryPartitionedComputationDenial as Denial;
use super::tree_attempt::{EditDisposition, TreeAttempt};
use std::collections::BTreeMap;
use worth_execution::{
    ExecutionResourceLease, MapKernelStop, NativeReductionTree, ReductionRunFailure,
};
use worth_foundational::facade::PartitionIdentity;

type Tree<Reduced> = NativeReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>;

pub(super) fn apply<Reduced, Stopped>(
    mut attempt: TreeAttempt<Tree<Reduced>, ReductionRunFailure<MapKernelStop>>,
    removed: &worth_proof::CanonicalUniqueVec<PartitionIdentity>,
    results: &BTreeMap<PartitionIdentity, Reduced>,
    execution: &QueryRequestExecution<'_>,
    lease: &ExecutionResourceLease<'_>,
) -> TreeAttempt<EditDisposition<Tree<Reduced>>, Denial<Stopped>>
where
    Reduced: Clone + worth_execution::CanonicalBits,
{
    // Merge the two canonical streams without allocating another edit map.
    // A supplied result takes precedence over removal, as in the original map.
    let mut removals = removed.as_slice().iter().copied().peekable();
    let mut changes = results.iter().peekable();
    loop {
        let edit = match (removals.peek().copied(), changes.peek().copied()) {
            (None, None) => break,
            (Some(id), Some((&next, value))) if id >= next => {
                changes.next();
                if id == next {
                    removals.next();
                }
                (next, Some(value))
            }
            (Some(id), _) => {
                removals.next();
                (id, None)
            }
            (None, Some((&id, value))) => {
                changes.next();
                (id, Some(value))
            }
        };
        let (identity, value) = edit;
        attempt = attempt.and_then(|mut tree| {
            let checkpoint = || execution.checkpoint();
            let outcome = match value {
                None => tree.delete_native(identity, Some(lease), checkpoint),
                Some(value) if tree.leaf(identity).is_some() => {
                    tree.update_native(identity, value.clone(), Some(lease), checkpoint)
                }
                Some(value) => tree.insert_native(identity, value.clone(), Some(lease), checkpoint),
            };
            TreeAttempt::native(outcome.map(|metrics| (tree, metrics)))
        });
        if attempt.outcome().is_err() {
            return attempt.finish_edits();
        }
    }
    attempt.finish_edits()
}
