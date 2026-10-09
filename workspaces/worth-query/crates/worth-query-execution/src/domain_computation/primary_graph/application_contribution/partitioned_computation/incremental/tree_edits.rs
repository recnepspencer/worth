//! Ongoing retained-path edits consume their accumulated work on every exit.
use super::super::super::request_execution::{QueryMemoryReservation, QueryRequestExecution};
use super::super::WorthQueryPartitionedComputationDenial as Denial;
use super::retained::RetainedPartitions;
use super::tree_attempt::{EditDisposition, TreeAttempt};
use super::tree_report::WorthQueryPartitionedTreeRebuildCause as Cause;
use std::collections::BTreeMap;
use worth_execution::{
    ChargedBytes, MapKernelStop, ReductionPlan, ReductionRunFailure, ReductionTree,
};
use worth_foundational::facade::PartitionIdentity;

type Tree<Reduced> = ReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>;
pub(super) fn apply<Key, Item, Reduced, Stopped>(
    mut attempt: TreeAttempt<Tree<Reduced>, ReductionRunFailure<MapKernelStop>>,
    retained: &RetainedPartitions<Key, Item, Reduced>,
    plan: &ReductionPlan,
    results: &BTreeMap<PartitionIdentity, Reduced>,
    declared_bytes: u64,
    mut held: u64,
    execution: &QueryRequestExecution<'_>,
    memory: &mut QueryMemoryReservation,
) -> TreeAttempt<EditDisposition<Tree<Reduced>>, Denial<Stopped>>
where
    Reduced: Clone + ChargedBytes + worth_execution::CanonicalBits,
{
    // `None` deletes the partition; a result puts it.
    let mut edits = retained
        .partitions
        .keys()
        .filter(|identity| plan.identities().binary_search(identity).is_err())
        .map(|identity| (*identity, None))
        .collect::<BTreeMap<_, _>>();
    edits.extend(
        results
            .iter()
            .map(|(identity, value)| (*identity, Some(value))),
    );
    for (identity, edit) in edits {
        let tree = attempt.outcome().as_ref().expect("prior edits completed");
        let present = tree.leaf(identity).is_some();
        let bound = match edit {
            None => tree.checked_delete_memory_bound(identity, declared_bytes),
            Some(value) => if present {
                tree.checked_update_memory_bound(identity, declared_bytes)
            } else {
                tree.checked_insert_memory_bound(identity, declared_bytes)
            }
            .and_then(|path| path.checked_add(value.additional_charged_bytes())),
        };
        let Some(next_hold) = bound.and_then(|bound| held.checked_add(bound)) else {
            return attempt
                .map_result(|_| Ok(EditDisposition::Rebuild(Cause::EditCapacityOverflow)));
        };
        held = next_hold;
        // An edit that finds no room falls to the rebuild, which refuses as
        // a full run's build refuses.
        if memory.resize(held).is_err() {
            return attempt.map_result(|_| Ok(EditDisposition::Rebuild(Cause::EditMemory)));
        }
        attempt = attempt.and_then(|mut tree| {
            let checkpoint = || execution.checkpoint();
            let outcome = match edit {
                None => tree.delete_checked(identity, declared_bytes, checkpoint),
                Some(value) if present => {
                    tree.update_checked(identity, value.clone(), declared_bytes, checkpoint)
                }
                Some(value) => {
                    tree.insert_checked(identity, value.clone(), declared_bytes, checkpoint)
                }
            };
            TreeAttempt::native(outcome.map(|metrics| (tree, metrics)))
        });
        if attempt.outcome().is_err() {
            return attempt.finish_edits();
        }
    }
    attempt.finish_edits()
}
