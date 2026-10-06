//! The next tree of an incremental run: every partition carried or computed
//! again, in a full run's order and with a full run's charges.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::{
    ExecutionMap, KeylessPartition, MapKernelContext, MapKernelFailure, MapOutcome,
};
use worth_foundational::facade::PartitionIdentity;

use super::super::super::execution_denial::PartitionRefusal;
use super::super::super::request_execution::{kept_tree_bytes, QueryRequestExecution};
use super::super::gather_memory::GatheredMemory;
use super::super::plan::GatheredComputationPartition;
use super::super::remaining_work::RemainingWork;
use super::super::{
    WorthQueryComputationPartitionStop, WorthQueryDeterministicReducer,
    WorthQueryPartitionedComputationDenial,
};
use super::retained::{
    observe, CarriedPartitions, CompletedComputationRun, RetainedBasis, RetainedPartition,
    RetainedPartitions, TypedPrior, WorthQueryPartitionedComputationRun,
};
use super::tree_update::{next_tree, same_bits};
use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;

/// One marked partition, gathered again for its kernel.
struct Recomputed<Key, Gathered> {
    map: ExecutionMap<GatheredComputationPartition<Key, Gathered>, u64>,
    gather: Option<ComputationCallCharge>,
    /// The request memory its gathering holds until its map's admission
    /// takes it over.
    memory: GatheredMemory,
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
        mut memory: GatheredMemory,
    ) -> Result<(), WorthQueryPartitionedComputationDenial<Stopped>> {
        let value = GatheredComputationPartition {
            identity,
            key: Arc::clone(&partition.key),
            items: Arc::clone(&partition.members),
            gathered,
        };
        memory.after_gather(&value)?;
        let map = ExecutionMap::from_keyless_partitions(BTreeMap::from([(
            identity,
            KeylessPartition {
                value,
                kernel_scratch_bytes: 0,
                max_result_bytes: self.declared_bytes,
            },
        )]))
        .map_err(WorthQueryPartitionedComputationDenial::from_map_overflow)?;
        self.recomputed.insert(
            identity,
            Recomputed {
                map,
                gather,
                memory,
            },
        );
        Ok(())
    }

    /// Computes the marked partitions and builds the next tree from the
    /// retained one. A carried kernel is charged its retained work in its
    /// place among the partitions; a recomputed result with the retained
    /// result's canonical bits replaces nothing. A recomputed kernel's work
    /// is execution's report of its one-partition map, run on its own
    /// dispatch of the request's execution.
    pub(in super::super) fn compute<Stopped, Kernel>(
        self,
        execution: &QueryRequestExecution<'_>,
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
            ) -> Result<Reduced, MapKernelFailure<PartitionRefusal<Stopped>>>
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
            cause: WorthQueryComputationPartitionStop::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            ),
        };
        let mut remaining = remaining_work;
        let mut partitions = BTreeMap::new();
        let mut results = BTreeMap::new();
        let mut results_memory = execution
            .reserve(0)
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        // What one recomputed result can hold once it leaves its map: its
        // entry, and at most the declared bytes its map admits it.
        let result_bound = u64::try_from(size_of::<(PartitionIdentity, Reduced)>())
            .ok()
            .and_then(|inline| inline.checked_add(declared_bytes))
            .ok_or(WorthQueryPartitionedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::CapacityOverflow,
            ))?;
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
            let dispatch = execution
                .dispatch()
                .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
            // The result's bound is held before its map runs, so the result
            // never exists unreserved; it settles to the result's own bytes.
            let settled = results_memory.bytes();
            results_memory
                .grow(result_bound)
                .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
            // The map's admission takes over the gathered partition's hold.
            let outcome = dispatch
                .map(
                    remaining.remaining(),
                    &marked.map,
                    marked.memory.into_held(),
                    &kernel,
                )
                .map_err(|denial| {
                    // Only the map runs under this ceiling, and it contains
                    // its kernel's panics: what is left is the kernel's.
                    WorthQueryPartitionedComputationDenial::from_work_ceiling(
                        denial,
                        WorthQueryPartitionedComputationDenial::Partition {
                            partition: *identity,
                            cause: WorthQueryComputationPartitionStop::Panicked,
                        },
                    )
                })?;
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
            let kept = if replaced {
                u64::try_from(size_of::<(PartitionIdentity, Reduced)>())
                    .ok()
                    .and_then(|inline| inline.checked_add(value.additional_charged_bytes()))
                    .and_then(|bytes| settled.checked_add(bytes))
                    .ok_or(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
            } else {
                Ok(settled)
            };
            kept.and_then(|bytes| results_memory.resize(bytes))
                .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
            if replaced {
                results.insert(*identity, value);
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
            execution,
            &mut results_memory,
        )?;
        // The results are the tree's now: the hold settles to what the tree
        // keeps, as a full run's tree is handed over when its run ends.
        kept_tree_bytes(&tree)
            .ok_or(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
            .and_then(|bytes| results_memory.resize(bytes))
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
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
                tree_memory: results_memory,
            },
        })
    }
}
