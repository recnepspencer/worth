//! The next tree of an incremental run, built from the retained one.

use std::collections::BTreeMap;

use worth_execution::{ChargedBytes, MapKernelStop, ReductionPlan, ReductionTree};
use worth_foundational::facade::PartitionIdentity;

use super::super::super::request_execution::{QueryMemoryReservation, QueryRequestExecution};
use super::super::super::WorthQueryManagedComputationResourceDenial as Resource;
use super::super::{WorthQueryDeterministicReducer, WorthQueryPartitionedComputationDenial};
use super::retained::RetainedPartitions;
use super::tree_report::{
    rebuild_cause, ReportedTree, WorthQueryPartitionedTreeMetrics as Metrics,
    WorthQueryPartitionedTreeRebuildCause as Cause, WorthQueryPartitionedTreeRun as Run,
};

type Tree<Reduced> = ReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>;

/// The tree over `plan`'s partitions: the retained tree with each partition
/// the run no longer has deleted, and `results` in place of their
/// partitions' leaves, a new partition's inserted.
///
/// When `reduction_work`, the combines a full build of `plan` charges, fits,
/// the tree is the retained one with each changed path recombined, edit by
/// edit in ascending partition order. Otherwise, or when an edit runs out of
/// room, the tree is built again from every leaf the way a full run builds
/// it, so the run fails where and as a full run fails. Every combine is a
/// safe point of the request: an interruption stops the run at that tree
/// node and is never retried as a rebuild.
///
/// `memory` holds the results. Each way of building holds its declared bound
/// beside them before it builds, as a full run holds its tree's, and each
/// edit holds its own bound on the tree it edits before it edits, so no node
/// exists unreserved; the caller then settles the hold to the tree it keeps.
pub(super) fn next_tree<Key, Item, Reduced, Stopped>(
    retained: &RetainedPartitions<Key, Item, Reduced>,
    plan: ReductionPlan,
    reduction_work: Option<u64>,
    mut results: BTreeMap<PartitionIdentity, Reduced>,
    remaining: u64,
    declared_bytes: u64,
    reducer: &WorthQueryDeterministicReducer<Reduced>,
    execution: &QueryRequestExecution<'_>,
    memory: &mut QueryMemoryReservation,
) -> ReportedTree<Tree<Reduced>, Stopped>
where
    Reduced: Clone + ChargedBytes + worth_execution::CanonicalBits,
{
    // A failed edit's transient hold is not part of the rebuild's input
    // results. Preserve their original charge before any path copies exist.
    let results_bytes = memory.bytes();
    let mut actual = Metrics::before_attempts();
    let mut rebuilt = Some(match reduction_work {
        Some(_) => Cause::WorkCeiling,
        None => Cause::WorkCounterOverflow,
    });
    let outcome = (|| {
        if reduction_work.is_some_and(|work| work <= remaining) {
            rebuilt = None;
            match edited(
                retained,
                &plan,
                &results,
                declared_bytes,
                execution,
                memory,
                &mut actual,
            )? {
                EditDisposition::Completed(tree) => return Ok(tree),
                EditDisposition::Rebuild(cause) => rebuilt = Some(cause),
            }
        }
        rebuild(
            retained,
            plan,
            &mut results,
            remaining,
            results_bytes,
            declared_bytes,
            reducer,
            execution,
            memory,
            &mut actual,
        )
    })();
    ReportedTree {
        outcome,
        report: match rebuilt {
            None => Run::Edited(actual),
            Some(cause) => Run::Rebuilt(cause, actual),
        },
    }
}

/// A full build over carried and newly computed leaves, including a stopped
/// build's metrics. Earlier edit attempts remain in `actual`.
fn rebuild<Key, Item, Reduced, Stopped>(
    retained: &RetainedPartitions<Key, Item, Reduced>,
    plan: ReductionPlan,
    results: &mut BTreeMap<PartitionIdentity, Reduced>,
    remaining: u64,
    results_bytes: u64,
    declared_bytes: u64,
    reducer: &WorthQueryDeterministicReducer<Reduced>,
    execution: &QueryRequestExecution<'_>,
    memory: &mut QueryMemoryReservation,
    actual: &mut Metrics,
) -> Result<Tree<Reduced>, WorthQueryPartitionedComputationDenial<Stopped>>
where
    Reduced: Clone + ChargedBytes + worth_execution::CanonicalBits,
{
    let overflow = || WorthQueryPartitionedComputationDenial::Resource(Resource::CapacityOverflow);
    // A full run's build over every leaf, beside the leaves: the results
    // the hold already counts, and a copy of each retained leaf.
    let count = plan.identities().len();
    let copies = plan
        .identities()
        .iter()
        .filter(|identity| !results.contains_key(identity))
        .try_fold(0_u64, |sum, identity| {
            sum.checked_add(
                retained
                    .tree
                    .leaf(*identity)
                    .map_or(0, ChargedBytes::additional_charged_bytes),
            )
        });
    let held = Tree::<Reduced>::checked_build_memory_bound(count, declared_bytes)
        .and_then(|build| {
            build.checked_add(
                inline::<(PartitionIdentity, Reduced)>()?
                    .checked_mul(u64::try_from(count).ok()?)?,
            )
        })
        .and_then(|bound| bound.checked_add(copies?))
        .and_then(|bound| results_bytes.checked_add(bound))
        .ok_or_else(overflow)?;
    memory
        .resize(held)
        .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
    let leaves = plan
        .identities()
        .iter()
        .map(|identity| {
            let leaf = results
                .remove(identity)
                .or_else(|| retained.tree.leaf(*identity).cloned());
            (
                *identity,
                leaf.expect("every partition has a result or a retained leaf"),
            )
        })
        .collect::<Vec<_>>();
    let mut left = remaining;
    let outcome = ReductionTree::try_from_declared_checked(
        plan,
        leaves,
        (reducer.identity)(),
        reducer.combine,
        declared_bytes,
        || {
            execution.checkpoint()?;
            left = left.checked_sub(1).ok_or(MapKernelStop::WorkCeiling)?;
            Ok(())
        },
    );
    match outcome {
        Ok((tree, metrics)) => {
            actual.include(metrics);
            Ok(tree)
        }
        Err(failure) => {
            actual.include(failure.metrics);
            Err(WorthQueryPartitionedComputationDenial::from_reduction(
                failure,
            ))
        }
    }
}

/// A completed edit batch, or its exhaustive reason to rebuild.
enum EditDisposition<Tree> {
    Completed(Tree),
    Rebuild(Cause),
}

/// Edits only changed root paths. Every returned attempt contributes its
/// existing metrics before success, interruption, or a rebuild is selected.
fn edited<Key, Item, Reduced, Stopped>(
    retained: &RetainedPartitions<Key, Item, Reduced>,
    plan: &ReductionPlan,
    results: &BTreeMap<PartitionIdentity, Reduced>,
    declared_bytes: u64,
    execution: &QueryRequestExecution<'_>,
    memory: &mut QueryMemoryReservation,
    actual: &mut Metrics,
) -> Result<EditDisposition<Tree<Reduced>>, WorthQueryPartitionedComputationDenial<Stopped>>
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
    // The copy shares the retained tree; each edited path is new, and so is
    // each result's copy in its leaf.
    let Some(mut held) =
        inline::<Tree<Reduced>>().and_then(|copy| memory.bytes().checked_add(copy))
    else {
        return Ok(EditDisposition::Rebuild(Cause::EditCapacityOverflow));
    };
    let mut tree = retained.tree.clone();
    for (identity, edit) in edits {
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
            return Ok(EditDisposition::Rebuild(Cause::EditCapacityOverflow));
        };
        held = next_hold;
        // An edit that finds no room falls to the rebuild, which refuses as
        // a full run's build refuses.
        if memory.resize(held).is_err() {
            return Ok(EditDisposition::Rebuild(Cause::EditMemory));
        }
        let checkpoint = || execution.checkpoint();
        let outcome = match edit {
            None => tree.delete_checked(identity, declared_bytes, checkpoint),
            Some(value) if present => {
                tree.update_checked(identity, value.clone(), declared_bytes, checkpoint)
            }
            Some(value) => tree.insert_checked(identity, value.clone(), declared_bytes, checkpoint),
        };
        match outcome {
            Ok(metrics) => actual.include(metrics),
            Err(failure) => {
                actual.include(failure.metrics);
                return match rebuild_cause(failure.reason) {
                    Err(stop) => Err(WorthQueryPartitionedComputationDenial::from_kernel_stop(
                        stop,
                    )),
                    Ok(cause) => Ok(EditDisposition::Rebuild(cause)),
                };
            }
        }
    }
    Ok(EditDisposition::Completed(tree))
}

/// The bytes a value of `T` takes in place.
fn inline<T>() -> Option<u64> {
    u64::try_from(size_of::<T>()).ok()
}

/// Whether two results have the same canonical bits. A result whose bits do
/// not encode is never the same.
pub(super) fn same_bits<Value: worth_execution::CanonicalBits>(
    left: &Value,
    right: &Value,
) -> bool {
    fn bits<Value: worth_execution::CanonicalBits>(value: &Value) -> Option<Vec<u8>> {
        let mut out = Vec::with_capacity(value.canonical_len()?);
        value
            .visit_canonical_bits(&mut |chunk| {
                out.extend_from_slice(chunk);
                true
            })
            .then_some(out)
    }
    matches!((bits(left), bits(right)), (Some(left), Some(right)) if left == right)
}
