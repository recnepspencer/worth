//! The next tree of an incremental run: every partition carried or computed
//! again, in a full run's order and with a full run's charges.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::{
    ExecutionMap, ExecutionWorkCeiling, MapKernelContext, MapKernelFailure, MapKernelStop,
    MapOutcome, MapPartition, ReductionPlan, ReductionTree,
};
use worth_foundational::facade::PartitionIdentity;

use super::super::plan::GatheredComputationPartition;
use super::super::remaining_work::RemainingWork;
use super::super::{WorthQueryDeterministicReducer, WorthQueryPartitionedComputationDenial};
use super::retained::{
    observe, CarriedPartitions, CompletedComputationRun, RetainedBasis, RetainedPartition,
    RetainedPartitions, TypedPrior, WorthQueryPartitionedComputationRun,
};
use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;

/// One marked partition, gathered again for its kernel.
struct Recomputed<Key, Gathered> {
    map: ExecutionMap<GatheredComputationPartition<Key, Gathered>, u64>,
    gather: Option<ComputationCallCharge>,
}

/// An incremental run after `prepare`: the marked partitions gathered again,
/// what is left of the declared work, and the prior run it carries from.
pub(in super::super) struct PreparedIncremental<Key, Item, Reduced, Gathered> {
    remaining_work: RemainingWork,
    declared_bytes: u64,
    recomputed: BTreeMap<PartitionIdentity, Recomputed<Key, Gathered>>,
    basis: RetainedBasis,
    prior: TypedPrior<Key, Item, Reduced>,
}

/// What computing an incremental run produced.
pub(in super::super) struct ComputedIncremental<Reduced> {
    pub(in super::super) reduced: Reduced,
    /// The kernels' and the combines' work, as a full run charges it.
    pub(in super::super) computed_work: u64,
    pub(in super::super) completed: CompletedComputationRun,
}

impl<Key, Item, Reduced, Gathered> PreparedIncremental<Key, Item, Reduced, Gathered>
where
    Key: Send + Sync + 'static,
    Gathered: Send + Sync + worth_execution::ChargedBytes,
{
    pub(super) fn new(
        remaining_work: RemainingWork,
        declared_bytes: u64,
        basis: RetainedBasis,
        prior: TypedPrior<Key, Item, Reduced>,
    ) -> Self {
        Self {
            remaining_work,
            declared_bytes,
            recomputed: BTreeMap::new(),
            basis,
            prior,
        }
    }

    pub(in super::super) const fn remaining_work(&self) -> RemainingWork {
        self.remaining_work
    }

    /// Readies one marked partition's kernel.
    pub(super) fn gathered<Stopped>(
        &mut self,
        identity: PartitionIdentity,
        partition: &RetainedPartition<Key>,
        gathered: Gathered,
        gather: Option<ComputationCallCharge>,
    ) -> Result<(), WorthQueryPartitionedComputationDenial<Stopped>> {
        let map = ExecutionMap::try_from_declared_partitions(
            vec![identity],
            vec![MapPartition {
                identity,
                value: GatheredComputationPartition {
                    identity,
                    key: Arc::clone(&partition.key),
                    items: Arc::clone(&partition.members),
                    gathered,
                },
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 0,
                max_result_bytes: self.declared_bytes,
            }],
        )
        .map_err(WorthQueryPartitionedComputationDenial::from_map)?;
        self.recomputed.insert(identity, Recomputed { map, gather });
        Ok(())
    }

    /// Computes the marked partitions and builds the next tree from the
    /// retained one. A carried kernel is charged its retained work in its
    /// place among the partitions; a recomputed result with the retained
    /// result's canonical bits replaces nothing. A recomputed kernel's work
    /// is execution's report of its one-partition map.
    pub(in super::super) fn compute<Stopped, Kernel>(
        self,
        kernel: Kernel,
        reducer: &WorthQueryDeterministicReducer<Reduced>,
    ) -> Result<ComputedIncremental<Reduced>, WorthQueryPartitionedComputationDenial<Stopped>>
    where
        Item: Send + Sync + worth_execution::ChargedBytes + 'static,
        Reduced: Clone
            + Send
            + Sync
            + worth_execution::ChargedBytes
            + worth_execution::CanonicalBits
            + 'static,
        Stopped: Send + worth_execution::ChargedBytes,
        Kernel: Fn(
                &GatheredComputationPartition<Key, Gathered>,
                &mut MapKernelContext<'_, '_>,
            ) -> Result<Reduced, MapKernelFailure<Stopped>>
            + Sync,
    {
        let Self {
            remaining_work,
            declared_bytes,
            mut recomputed,
            basis,
            prior,
        } = self;
        let retained = &*prior.typed;
        let exhausted = |partition| WorthQueryPartitionedComputationDenial::Partition {
            partition,
            cause: super::super::WorthQueryComputationPartitionStop::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            ),
        };
        let mut remaining = remaining_work;
        let mut partitions = BTreeMap::new();
        let mut results = Vec::new();
        let skipped = retained
            .partitions
            .keys()
            .filter(|identity| !recomputed.contains_key(identity))
            .copied()
            .collect();
        for (identity, carried) in &retained.partitions {
            let Some(marked) = recomputed.remove(identity) else {
                remaining
                    .spend(Some(carried.kernel_units))
                    .map_err(|_| exhausted(*identity))?;
                partitions.insert(*identity, Arc::clone(carried));
                continue;
            };
            let (outcome, _) = ExecutionWorkCeiling::new(remaining.remaining())
                .run(None, || marked.map.run(None, &kernel))
                .map_err(WorthQueryPartitionedComputationDenial::from_work_ceiling)?;
            let (value, units) = match outcome {
                MapOutcome::Complete { mut values, report } => (
                    values.pop().expect("one partition yields one result"),
                    report.charged_work(),
                ),
                MapOutcome::Stopped { reason, .. } => {
                    return Err(WorthQueryPartitionedComputationDenial::from_map_stop(
                        reason,
                    ));
                }
            };
            remaining
                .spend(Some(units))
                .map_err(|_| exhausted(*identity))?;
            let replaced = retained
                .tree
                .leaf(*identity)
                .is_none_or(|leaf| !same_bits(leaf, &value));
            if replaced {
                results.push((*identity, value));
            }
            if let Some(gather) = marked.gather {
                partitions.insert(
                    *identity,
                    Arc::new(RetainedPartition {
                        key: Arc::clone(&carried.key),
                        key_bytes: carried.key_bytes,
                        members: Arc::clone(&carried.members),
                        gather,
                        kernel_units: units,
                    }),
                );
            }
        }
        let tree = next_tree(
            retained,
            results,
            remaining.remaining(),
            declared_bytes,
            reducer,
        )?;
        let computed_work = remaining
            .spent_since(remaining_work)
            .and_then(|kernels| kernels.checked_add(retained.reduction_work))
            .ok_or(WorthQueryPartitionedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            ))?;
        let reduced = tree.result().clone();
        // A recomputed gathering whose charge did not measure cannot be
        // carried again, so the run retains nothing.
        let typed = (partitions.len() == retained.partitions.len()).then(|| RetainedPartitions {
            items: Arc::clone(&retained.items),
            membership: retained.membership.clone(),
            item_keys: Arc::clone(&retained.item_keys),
            partitions,
            tree,
            reduction_work: retained.reduction_work,
        });
        let typed_bytes = typed.as_ref().and_then(RetainedPartitions::charged_bytes);
        let typed = typed.map(|typed| Arc::new(typed) as Arc<dyn std::any::Any + Send + Sync>);
        observe(WorthQueryPartitionedComputationRun::Incremental, None);
        Ok(ComputedIncremental {
            reduced,
            computed_work,
            completed: CompletedComputationRun {
                basis,
                typed,
                typed_bytes,
                carried: Some(CarriedPartitions {
                    prior: prior.state,
                    skipped,
                }),
            },
        })
    }
}

/// The retained tree with `results` in place of their partitions' leaves.
///
/// When the combines a full build charges fit, the tree is the retained one
/// with each changed path recombined. Otherwise, or when recombining a path
/// fails, the tree is built again from every leaf the way a full run builds
/// it, so the run fails where and as a full run fails.
fn next_tree<Key, Item, Reduced, Stopped>(
    retained: &RetainedPartitions<Key, Item, Reduced>,
    results: Vec<(PartitionIdentity, Reduced)>,
    remaining: u64,
    declared_bytes: u64,
    reducer: &WorthQueryDeterministicReducer<Reduced>,
) -> Result<
    ReductionTree<Reduced, fn(&Reduced, &Reduced) -> Reduced>,
    WorthQueryPartitionedComputationDenial<Stopped>,
>
where
    Reduced: Clone + worth_execution::ChargedBytes + worth_execution::CanonicalBits,
{
    if retained.reduction_work <= remaining {
        let mut tree = retained.tree.clone();
        let recombined = results.iter().all(|(identity, value)| {
            tree.update_checked(
                *identity,
                value.clone(),
                declared_bytes,
                || Ok::<(), ()>(()),
            )
            .is_ok()
        });
        if recombined {
            return Ok(tree);
        }
    }
    let mut changed = results.into_iter().collect::<BTreeMap<_, _>>();
    let leaves = retained
        .partitions
        .keys()
        .map(|identity| {
            let leaf = changed
                .remove(identity)
                .or_else(|| retained.tree.leaf(*identity).cloned());
            (
                *identity,
                leaf.expect("every retained partition has a leaf"),
            )
        })
        .collect::<Vec<_>>();
    let plan = ReductionPlan::try_from_sorted_unique(
        leaves.iter().map(|(identity, _)| *identity).collect(),
    )
    .expect("retained partitions are unique and in identity order");
    let mut left = remaining;
    ReductionTree::try_from_declared_checked(
        plan,
        leaves,
        (reducer.identity)(),
        reducer.combine,
        declared_bytes,
        || {
            left = left.checked_sub(1).ok_or(MapKernelStop::WorkCeiling)?;
            Ok(())
        },
    )
    .map(|(tree, _)| tree)
    .map_err(WorthQueryPartitionedComputationDenial::from_reduction)
}

/// Whether two results have the same canonical bits. A result whose bits do
/// not encode is never the same.
fn same_bits<Value: worth_execution::CanonicalBits>(left: &Value, right: &Value) -> bool {
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
