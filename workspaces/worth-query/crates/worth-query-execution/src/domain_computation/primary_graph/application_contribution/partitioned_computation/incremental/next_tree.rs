//! The next tree of an incremental run: every partition carried or computed
//! again, in a full run's order and with a full run's charges.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_execution::{
    ExecutionMap, KeylessPartition, MapKernelContext, MapKernelFailure, MapOutcome,
    PartitionItemId, ReductionPlan,
};
use worth_foundational::facade::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

use super::super::super::execution_denial::PartitionRefusal;
use super::super::super::request_execution::{
    kept_tree_bytes, QueryMemoryReservation, QueryRequestExecution,
};
use super::super::gather_memory::GatheredMemory;
use super::super::items::{ItemDigests, Items};
use super::super::plan::GatheredComputationPartition;
use super::super::remaining_work::RemainingWork;
use super::super::routing::ComputationPartitionRouting;
use super::super::{
    WorthQueryComputationPartitionStop, WorthQueryDeterministicReducer,
    WorthQueryPartitionedComputationDenial,
};
use super::observed::{observe, observe_tree};
use super::retained::{
    CarriedCalls, CompletedComputationRun, RetainedBasis, RetainedCall, RetainedPartition,
    RetainedPartitions, TypedPrior, WorthQueryPartitionedComputationRun,
};
use super::tree_update::{next_tree, same_bits};
use super::{CompletedComputationRetention, PriorAbsence};
use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;

/// One marked partition, gathered again for its kernel.
struct Recomputed<Key, Gathered> {
    map: ExecutionMap<GatheredComputationPartition<Key, Gathered>, u64>,
    key: Arc<Key>,
    key_bytes: u64,
    gather: Option<ComputationCallCharge>,
    /// The request memory its gathering holds until its map's admission
    /// takes it over.
    memory: GatheredMemory,
}

/// How one partition of the run is had: carried from the retained run, or
/// gathered again for its kernel.
enum Disposition<Key, Gathered> {
    Carried(Arc<RetainedPartition<Key>>),
    Recomputed(Recomputed<Key, Gathered>),
}

/// The run's items, keys and routes, carried or made again.
pub(super) struct NextPartitioning<Item> {
    pub(super) items: Items<Item>,
    pub(super) digests: ItemDigests,
    /// `None` when the membership or an item's key was made again and its
    /// charge did not measure: the run cannot be carried again.
    pub(super) membership: Option<RetainedCall>,
    pub(super) item_keys: BTreeMap<PartitionItemId, RetainedCall>,
    pub(super) routing: Arc<ComputationPartitionRouting>,
    /// The request memory a routing the run built holds.
    pub(super) routing_memory: Option<QueryMemoryReservation>,
    pub(super) carried_membership: bool,
    pub(super) carried_items: BTreeSet<PartitionItemId>,
}

/// An incremental run after `prepare`: every partition carried or gathered
/// again, what is left of the declared work, and the prior run it carries
/// from.
pub(in super::super) struct PreparedIncremental<Key, Item, Reduced, Gathered> {
    remaining_work: RemainingWork,
    declared_bytes: u64,
    partitions: BTreeMap<PartitionIdentity, Disposition<Key, Gathered>>,
    next: NextPartitioning<Item>,
    basis: RetainedBasis,
    prior: TypedPrior<Key, Item, Reduced>,
}

/// What computing an incremental run produced.
pub(in super::super) struct ComputedIncremental<Reduced> {
    pub(in super::super) reduced: Reduced,
    /// The kernels' and the combines' work, as a full run charges it.
    pub(in super::super) computed_work: u64,
    pub(in super::super) completed: CompletedComputationRetention,
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
        next: NextPartitioning<Item>,
    ) -> Self {
        Self {
            remaining_work,
            declared_bytes,
            partitions: BTreeMap::new(),
            next,
            basis,
            prior,
        }
    }

    pub(in super::super) const fn remaining_work(&self) -> RemainingWork {
        self.remaining_work
    }

    /// Carries one partition's gathering and kernel from the retained run.
    pub(super) fn carried(
        &mut self,
        identity: PartitionIdentity,
        partition: &Arc<RetainedPartition<Key>>,
    ) {
        self.partitions
            .insert(identity, Disposition::Carried(Arc::clone(partition)));
    }

    /// Readies one marked partition's kernel.
    pub(super) fn gathered<Stopped>(
        &mut self,
        identity: PartitionIdentity,
        key: Arc<Key>,
        key_bytes: u64,
        members: Arc<[PartitionItemId]>,
        gathered: Gathered,
        gather: Option<ComputationCallCharge>,
        mut memory: GatheredMemory,
    ) -> Result<(), WorthQueryPartitionedComputationDenial<Stopped>> {
        let value = GatheredComputationPartition {
            identity,
            key: Arc::clone(&key),
            items: members,
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
        self.partitions.insert(
            identity,
            Disposition::Recomputed(Recomputed {
                map,
                key,
                key_bytes,
                gather,
                memory,
            }),
        );
        Ok(())
    }

    /// Computes the marked partitions and builds the next tree from the
    /// retained one. A carried kernel is charged its retained work in its
    /// place among this run's partitions; a recomputed result with the
    /// retained result's canonical bits replaces nothing, a new partition's
    /// is a new leaf, and a partition the run no longer has leaves the
    /// tree. A recomputed kernel's work
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
            partitions: dispositions,
            next,
            basis,
            prior,
        } = self;
        let retained = prior.typed();
        let exhausted = |partition| WorthQueryPartitionedComputationDenial::Partition {
            partition,
            cause: WorthQueryComputationPartitionStop::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            ),
        };
        let mut remaining = remaining_work;
        let mut partitions = BTreeMap::new();
        let mut carried = BTreeSet::new();
        let mut gathers_measured = true;
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
        let plan = ReductionPlan::from_canonical(CanonicalUniqueVec::from_btree_set(
            dispositions.keys().copied().collect(),
        ));
        for (identity, disposition) in dispositions {
            let marked = match disposition {
                Disposition::Carried(partition) => {
                    remaining
                        .spend(Some(partition.kernel_units))
                        .map_err(|_| exhausted(identity))?;
                    partitions.insert(identity, partition);
                    carried.insert(identity);
                    continue;
                }
                Disposition::Recomputed(marked) => marked,
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
                            partition: identity,
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
                .map_err(|_| exhausted(identity))?;
            // A partition the retained tree does not hold is a new leaf.
            let replaced = retained
                .tree
                .leaf(identity)
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
                results.insert(identity, value);
            }
            match marked.gather {
                Some(gather) => {
                    let partition = RetainedPartition {
                        key: marked.key,
                        key_bytes: marked.key_bytes,
                        gather,
                        kernel_units: units,
                    };
                    partitions.insert(identity, Arc::new(partition));
                }
                None => gathers_measured = false,
            }
        }
        // The combines a full build of this run's tree charges, from its
        // shape alone, by the reduce's own rule.
        let reduction_work = plan.checked_build_work();
        let tree = next_tree(
            retained,
            plan,
            reduction_work,
            results,
            remaining.remaining(),
            declared_bytes,
            reducer,
            execution,
            &mut results_memory,
        );
        observe_tree(tree.report);
        let tree = tree.outcome?;
        // The results are the tree's now: the hold settles to what the tree
        // keeps, as a full run's tree is handed over when its run ends.
        kept_tree_bytes(&tree)
            .ok_or(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
            .and_then(|bytes| results_memory.resize(bytes))
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        let computed_work = remaining
            .spent_since(remaining_work)
            .zip(reduction_work)
            .and_then(|(kernels, combines)| kernels.checked_add(combines))
            .ok_or(WorthQueryPartitionedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            ))?;
        let reduced = tree.result().clone();
        // A call made again whose charge did not measure cannot be carried
        // again, so the run retains nothing.
        let typed = next
            .membership
            .filter(|_| gathers_measured)
            .map(|membership| RetainedPartitions {
                items: next.items,
                digests: next.digests,
                membership,
                item_keys: Arc::new(next.item_keys),
                routing: next.routing,
                partitions,
                tree,
            });

        observe(WorthQueryPartitionedComputationRun::Incremental, None);
        Ok(ComputedIncremental {
            reduced,
            computed_work,
            completed: match typed {
                None => CompletedComputationRetention::Absent(PriorAbsence::Unmeasured),
                Some(typed) => CompletedComputationRetention::Produced(CompletedComputationRun {
                    basis,
                    typed_bytes: typed.charged_bytes(),
                    typed: Arc::new(typed),
                    carried: Some(CarriedCalls {
                        prior: prior.into_state(),
                        membership: next.carried_membership,
                        items: next.carried_items,
                        partitions: carried,
                    }),
                    tree_memory: results_memory,
                    routing_memory: next.routing_memory,
                }),
            },
        })
    }
}
