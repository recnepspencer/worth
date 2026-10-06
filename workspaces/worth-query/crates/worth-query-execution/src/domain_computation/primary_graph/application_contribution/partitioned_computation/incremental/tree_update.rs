//! The next tree of an incremental run, built from the retained one.

use std::collections::BTreeMap;

use worth_execution::{
    ChargedBytes, MapKernelStop, ReductionPlan, ReductionRunStop, ReductionTree,
};
use worth_foundational::facade::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

use super::super::super::request_execution::{QueryMemoryReservation, QueryRequestExecution};
use super::super::super::WorthQueryManagedComputationResourceDenial as Resource;
use super::super::{WorthQueryDeterministicReducer, WorthQueryPartitionedComputationDenial};
use super::retained::RetainedPartitions;

type Tree<Reduced> = ReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>;

/// The retained tree with `results` in place of their partitions' leaves.
///
/// When the combines a full build charges fit, the tree is the retained one
/// with each changed path recombined. Otherwise, or when recombining a path
/// runs out of room, the tree is built again from every leaf the way a full
/// run builds it, so the run fails where and as a full run fails. Every
/// combine is a safe point of the request: an interruption stops the run at
/// that tree node and is never retried as a rebuild.
///
/// `memory` holds the results. Each way of building holds its declared bound
/// beside them before it builds, as a full run holds its tree's, so no node
/// exists unreserved; the caller then settles the hold to the tree it keeps.
pub(super) fn next_tree<Key, Item, Reduced, Stopped>(
    retained: &RetainedPartitions<Key, Item, Reduced>,
    mut results: BTreeMap<PartitionIdentity, Reduced>,
    remaining: u64,
    declared_bytes: u64,
    reducer: &WorthQueryDeterministicReducer<Reduced>,
    execution: &QueryRequestExecution<'_>,
    memory: &mut QueryMemoryReservation,
) -> Result<Tree<Reduced>, WorthQueryPartitionedComputationDenial<Stopped>>
where
    Reduced: Clone + ChargedBytes + worth_execution::CanonicalBits,
{
    let results_bytes = memory.bytes();
    let overflow = || WorthQueryPartitionedComputationDenial::Resource(Resource::CapacityOverflow);
    if retained.reduction_work <= remaining {
        // The copy shares the retained tree; each changed path is new, and
        // so is each result's copy in its leaf.
        let copy = inline::<Tree<Reduced>>().ok_or_else(overflow)?;
        let paths = results.iter().try_fold(copy, |sum, (identity, value)| {
            sum.checked_add(
                retained
                    .tree
                    .checked_update_memory_bound(*identity, declared_bytes)?,
            )?
            .checked_add(value.additional_charged_bytes())
        });
        let held = paths
            .and_then(|paths| results_bytes.checked_add(paths))
            .ok_or_else(overflow)?;
        // A path that finds no room falls to the rebuild, which refuses as a
        // full run's build refuses.
        if memory.resize(held).is_ok() {
            let mut tree = retained.tree.clone();
            let mut recombined = true;
            for (identity, value) in &results {
                match tree.update_checked(*identity, value.clone(), declared_bytes, || {
                    execution.checkpoint()
                }) {
                    Ok(_) => {}
                    Err(failure) => match failure.reason {
                        ReductionRunStop::Hook(stop) => {
                            return Err(WorthQueryPartitionedComputationDenial::from_kernel_stop(
                                stop,
                            ));
                        }
                        ReductionRunStop::Panic
                        | ReductionRunStop::Denial(_)
                        | ReductionRunStop::ResultCapacityExceeded
                        | ReductionRunStop::WorkCounterOverflow => {
                            recombined = false;
                            break;
                        }
                    },
                }
            }
            if recombined {
                return Ok(tree);
            }
        }
    }
    // A full run's build over every leaf, beside the leaves: the results
    // the hold already counts, and a copy of each retained leaf.
    let count = retained.partitions.len();
    let copies = retained
        .partitions
        .keys()
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
    let leaves = retained
        .partitions
        .keys()
        .map(|identity| {
            let leaf = results
                .remove(identity)
                .or_else(|| retained.tree.leaf(*identity).cloned());
            (
                *identity,
                leaf.expect("every retained partition has a leaf"),
            )
        })
        .collect::<Vec<_>>();
    let plan = ReductionPlan::from_canonical(CanonicalUniqueVec::from_btree_set(
        retained.partitions.keys().copied().collect(),
    ));
    let mut left = remaining;
    ReductionTree::try_from_declared_checked(
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
    )
    .map(|(tree, _)| tree)
    .map_err(WorthQueryPartitionedComputationDenial::from_reduction)
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
