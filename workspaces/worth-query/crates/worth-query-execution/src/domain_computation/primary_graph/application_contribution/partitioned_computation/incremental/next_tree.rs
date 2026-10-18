//! The next tree of an incremental run: every partition carried or computed
//! again, in a full run's order and with a full run's charges.

mod partition_wave;
mod preparation;
mod transport;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_execution::{KeylessPartition, MapKernelContext, MapKernelFailure, PartitionItemId};
use worth_foundational::facade::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

use super::super::super::computation::nested_failure::{
    MemberComputeFailure, NestedFailureTransport,
};
use super::super::super::execution_denial::PartitionRefusal;
use super::super::super::request_execution::{
    kept_tree_bytes, QueryMemoryReservation, QueryRequestExecution,
};
use super::super::gather_memory::GatheredMemory;
use super::super::items::{ItemDigests, Items};
use super::super::plan::GatheredComputationPartition;
use super::super::remaining_work::RemainingWork;
use super::super::routing::ComputationPartitionRouting;
use super::super::{WorthQueryDeterministicReducer, WorthQueryPartitionedComputationDenial};
use super::observed::{observe, observe_tree};
use super::retained::{
    CarriedCalls, CompletedComputationRun, RetainedBasis, RetainedCall, RetainedPartition,
    RetainedPartitions, TypedPrior, WorthQueryPartitionedComputationRunForTest,
};
use super::tree_update::{next_tree, same_bits};
use super::{CompletedComputationRetention, PriorAbsence};
use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial;
use crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge;

/// One marked partition, gathered again for its kernel.
struct Recomputed<Key, Gathered> {
    value: GatheredComputationPartition<Key, Gathered>,
    key: Arc<Key>,
    key_bytes: u64,
    gather: Option<ComputationCallCharge>,
    /// The request memory keeps gathered data held while the wave borrows it.
    memory: GatheredMemory,
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
    partitions: BTreeMap<PartitionIdentity, Recomputed<Key, Gathered>>,
    removed: CanonicalUniqueVec<PartitionIdentity>,
    next: NextPartitioning<Item>,
    basis: RetainedBasis,
    prior: TypedPrior<Key, Item, Reduced>,
    validated_at: Option<worth_relational::facade::snapshots::SnapshotHandle>,
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
    pub(in super::super) fn restrict_compute_ceiling(&mut self, ceiling: u64) {
        self.remaining_work.restrict(ceiling);
    }

    /// Settles every partition through one independent map, computing only
    /// marked partitions. Carried kernels contribute retained work and span;
    /// a recomputed result with the
    /// retained result's canonical bits replaces nothing, a new partition's
    /// is a new leaf, and a partition the run no longer has leaves the
    /// tree. A recomputed kernel's work
    /// is execution's admitted receipt in that same wave. The private declared
    /// budget spends that receipt, which also settles on the member meter.
    pub(in super::super) fn compute<Stopped, Kernel>(
        self,
        execution: &QueryRequestExecution<'_>,
        kernel: Kernel,
        observer: Option<&super::super::compute::kernel_observation::KernelCostObservationForTest>,
        reducer: &WorthQueryDeterministicReducer<Reduced>,
        transport: &mut NestedFailureTransport<'_>,
    ) -> Result<
        ComputedIncremental<Reduced>,
        MemberComputeFailure<WorthQueryPartitionedComputationDenial<Stopped>>,
    >
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
            partitions: recomputed,
            removed,
            next,
            basis,
            prior,
            validated_at,
        } = self;
        let retained = prior.typed();
        let mut remaining = remaining_work;
        let identities_unchanged = removed.as_slice().is_empty()
            && recomputed
                .keys()
                .all(|id| retained.native_run.result(*id).is_some());
        let fresh = recomputed.keys().copied().collect();
        let carried =
            super::carried_partitions::CarriedPartitions::new(Arc::clone(&next.routing), fresh);
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
        let all_results = u64::try_from(recomputed.len())
            .ok()
            .and_then(|count| count.checked_mul(result_bound))
            .ok_or(WorthQueryPartitionedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::CapacityOverflow,
            ))?;
        results_memory
            .grow(all_results)
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        let settled_wave = partition_wave::run(
            &recomputed,
            &retained.native_run,
            removed.clone(),
            execution,
            remaining.remaining(),
            declared_bytes,
            &kernel,
            transport,
        )?;
        remaining.spend(Some(settled_wave.work)).map_err(|_| {
            WorthQueryPartitionedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            )
        })?;
        let native_run = settled_wave.native_run;
        let mut partitions = retained
            .partitions
            .fork(execution)
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        for &id in removed.as_slice() {
            partitions
                .remove(id)
                .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        }
        let mut results_held = 0_u64;
        for (identity, marked) in recomputed {
            super::super::super::super::fact_verification_observation::computation_result_receiver(
            );
            let kernel_result = native_run
                .result(identity)
                .expect("changed native partition settled");
            let value = kernel_result.value();
            let settled = results_held;
            // A partition the retained tree does not hold is a new leaf.
            let replaced = retained
                .tree
                .leaf(identity)
                .is_none_or(|leaf| !same_bits(leaf, value));
            let kept = if replaced {
                u64::try_from(size_of::<(PartitionIdentity, Reduced)>())
                    .ok()
                    .and_then(|inline| inline.checked_add(value.additional_charged_bytes()))
                    .and_then(|bytes| settled.checked_add(bytes))
                    .ok_or(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
            } else {
                Ok(settled)
            };
            results_held = kept.map_err(WorthQueryPartitionedComputationDenial::Resource)?;
            if replaced {
                results.insert(identity, value.clone());
            }
            match marked
                .gather
                .filter(|_| kernel_result.native_reuse_available())
            {
                Some(gather) => {
                    let partition = RetainedPartition {
                        key: marked.key,
                        key_bytes: marked.key_bytes,
                        gather,
                    };
                    partitions
                        .put(identity, partition)
                        .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
                }
                None => gathers_measured = false,
            }
        }
        results_memory
            .resize(results_held)
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        // The combines a full build of this run's tree charges, from its
        // shape alone, by the reduce's own rule.
        let target = super::tree_update::TreeTarget::new(
            retained,
            Arc::clone(&next.routing),
            removed,
            identities_unchanged,
        );
        let reduction_work = target.full_work();
        let (tree, tree_bill) = transport.call(|| {
            execution
                .dispatch()
                .map_err(WorthQueryPartitionedComputationDenial::Resource)?
                .tree_stage(remaining.remaining(), |context| {
                    let tree = next_tree(
                        retained,
                        target,
                        reduction_work,
                        results,
                        context.remaining_work().min(remaining.remaining()),
                        declared_bytes,
                        reducer,
                        execution,
                        &mut results_memory,
                    );
                    observe_tree(tree.report);
                    let (work, span) = if let Ok(completed) = &tree.outcome {
                        completed
                            .checked_full_build_cost()
                            .expect("a completed build has bounded cost")
                    } else {
                        let metrics = tree.report.metrics();
                        (
                            u64::try_from(metrics.charged_work).unwrap_or(u64::MAX),
                            u64::try_from(metrics.charged_span).unwrap_or(u64::MAX),
                        )
                    };
                    context
                        .checkpoint_cost(work, span)
                        .map_err(WorthQueryPartitionedComputationDenial::from_kernel_stop)?;
                    if work == context.charged_cost().0
                        && tree.outcome.is_err()
                        && context.remaining_work() == 0
                    {
                        let _ = context.checkpoint(1);
                    }
                    tree.outcome
                })
                .map_err(|denial| {
                    WorthQueryPartitionedComputationDenial::from_work_ceiling(
                        denial,
                        WorthQueryPartitionedComputationDenial::ReducerPanicked,
                    )
                })
                .and_then(|(outcome, report)| outcome.map(|tree| (tree, report)))
        })?;
        remaining
            .spend(Some(tree_bill.charged_work()))
            .map_err(|_| {
                WorthQueryPartitionedComputationDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted,
                )
            })?;
        // The results are the tree's now: the hold settles to what the tree
        // keeps, as a full run's tree is handed over when its run ends.
        kept_tree_bytes(&tree)
            .ok_or(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
            .and_then(|bytes| results_memory.resize(bytes))
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        let computed_work = remaining.spent_since(remaining_work).ok_or(
            WorthQueryPartitionedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            ),
        )?;
        // Like Full, only a successfully reduced computation settles a receipt.
        if let Some(observer) = observer {
            observer.record(settled_wave.work, settled_wave.span);
        }
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
                native_run,
                tree,
            });

        observe(
            WorthQueryPartitionedComputationRunForTest::Incremental,
            None,
        );
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
                        validated_at,
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
