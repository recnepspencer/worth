//! The next tree of an incremental run: every partition carried or computed
//! again, in a full run's order and with a full run's charges.

mod call_recording;
mod partition_wave;
mod partitioning;
pub(in super::super) use partitioning::ComputedIncremental;
pub(super) use partitioning::NextPartitioning;
use partitioning::Recomputed;
mod preparation;

use crate::domain_computation::primary_graph::{
    application_attempt::ComputationRead, invariant_projection::ComputationCallSequence,
};
use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::{MapKernelContext, MapKernelFailure, PartitionItemId};
use worth_foundational::facade::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

use super::super::super::computation::nested_failure::{
    MemberComputeFailure, NestedFailureTransport,
};
use super::super::super::execution_denial::PartitionRefusal;
use super::super::super::request_execution::QueryRequestExecution;
use super::super::items::{ItemDigests, Items};
use super::super::plan::GatheredComputationPartition;
use super::super::preparation_work::PreparationWork;
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

/// An incremental run after `prepare`: every partition carried or gathered
/// again, the preparation receipt, and the prior run it carries from.
pub(in super::super) struct PreparedIncremental<Key, Item, Reduced, Gathered> {
    preparation_work: PreparationWork,
    compute_ceiling: Option<u64>,
    partitions: BTreeMap<PartitionIdentity, Recomputed<Key, Gathered>>,
    removed: CanonicalUniqueVec<PartitionIdentity>,
    next: NextPartitioning<Item>,
    basis: RetainedBasis,
    prior: TypedPrior<Key, Item, Reduced>,
    validated_at: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    transition: Option<
        Arc<crate::domain_computation::primary_graph::invariant_projection::CallShapeTransition>,
    >,
    rekeyed: worth_execution::ExecutionOrderedMap<PartitionItemId, ()>,
    ranges: Option<
        Arc<crate::domain_computation::primary_graph::application_attempt::NativeCallCarriage>,
    >,
}

impl<Key, Item, Reduced, Gathered> PreparedIncremental<Key, Item, Reduced, Gathered>
where
    Key: Send + Sync + 'static,
    Gathered: Send + Sync,
{
    pub(in super::super) fn restrict_compute_ceiling(&mut self, ceiling: u64) {
        self.compute_ceiling = Some(self.compute_ceiling.map_or(ceiling, |old| old.min(ceiling)));
    }

    /// Advances the genuine native run with changed gathers and removals.
    /// Native settlement owns full logical charges and first-stop prefixes.
    /// Only changed results reach tree maintenance; the native request
    /// spends the admitted native kernel and tree receipts.
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
        Item: Send + Sync + 'static,
        Reduced: Clone + Send + Sync + worth_execution::CanonicalBits + 'static,
        Stopped: Send,
        Kernel: Fn(
                &GatheredComputationPartition<Key, Gathered>,
                &mut MapKernelContext<'_, '_>,
            ) -> Result<Reduced, MapKernelFailure<PartitionRefusal<Stopped>>>
            + Sync,
    {
        let Self {
            preparation_work,
            compute_ceiling,
            partitions: recomputed,
            removed,
            next,
            basis,
            prior,
            validated_at,
            ranges,
            rekeyed,
            transition,
        } = self;
        let retained = prior.typed();
        let mut work_receipt = preparation_work;
        let identities_unchanged = removed.as_slice().is_empty()
            && recomputed
                .keys()
                .all(|id| retained.native_run.result(*id).is_some());
        let fresh = recomputed.keys().copied().collect();
        let carried =
            super::carried_partitions::CarriedPartitions::new(Arc::clone(&next.routing), fresh);
        let mut gathers_measured = true;
        let mut results = BTreeMap::new();
        let settled_wave = partition_wave::run(
            &recomputed,
            &retained.native_run,
            removed.clone(),
            execution,
            compute_ceiling,
            &kernel,
            transport,
        )?;
        work_receipt
            .spend(settled_wave.accounting.map(|(work, _)| work))
            .map_err(|_| {
                WorthQueryPartitionedComputationDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkCounterOverflow,
                )
            })?;
        let native_run = settled_wave.native_run;
        let patched_calls = if ranges.is_some() {
            let changes = rekeyed
                .keys()
                .map(|item| {
                    let call = &next.item_keys[item];
                    (
                        ComputationRead::ItemKey(*item),
                        &call.charge,
                        call.declared_units,
                    )
                })
                .chain(recomputed.iter().filter_map(|(id, marked)| {
                    marked.gather.as_ref().map(|charge| {
                        (
                            ComputationRead::Partition(*id),
                            charge,
                            execution.has_accounting().then_some(0),
                        )
                    })
                }));
            Some(
                match &transition {
                    Some(transition) => retained
                        .call_sequence
                        .edited(transition, changes, execution),
                    None => retained.call_sequence.patched(changes, execution),
                }
                .map_err(WorthQueryPartitionedComputationDenial::from_call_sequence)?,
            )
        } else {
            None
        };
        let allocation_policy = execution
            .allocation_policy()
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        let mut partitions = retained.partitions.fork();
        for &id in removed.as_slice() {
            partitions
                .remove(id, allocation_policy)
                .map_err(WorthQueryPartitionedComputationDenial::from_ordered)?;
        }

        for (identity, marked) in recomputed {
            super::super::super::super::fact_verification_observation::computation_result_receiver(
            );
            let kernel_result = native_run
                .result(identity)
                .expect("changed native partition settled");
            let value = kernel_result.value();
            // A partition the retained tree does not hold is a new leaf.
            let replaced = retained
                .tree
                .leaf(identity)
                .is_none_or(|leaf| !same_bits(leaf, value));
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
                        gather,
                    };
                    partitions
                        .put(identity, partition, allocation_policy)
                        .map_err(WorthQueryPartitionedComputationDenial::from_ordered)?;
                }
                None => gathers_measured = false,
            }
        }
        // The combines a full build of this run's tree charges, from its
        // shape alone, by the reduce's own rule.
        let target = super::tree_update::TreeTarget::new(
            retained,
            Arc::clone(&next.routing),
            removed,
            identities_unchanged,
            &results,
            execution,
        )?;
        let reduction_work = if execution.has_accounting() {
            target.full_work()
        } else {
            None
        };
        let (tree, tree_bill) = transport.call(|| {
            execution
                .dispatch()
                .map_err(WorthQueryPartitionedComputationDenial::Resource)?
                .tree_stage(compute_ceiling, |context| {
                    let tree = next_tree(
                        retained,
                        target,
                        reduction_work,
                        results,
                        context.remaining_work(),
                        reducer,
                        execution,
                    );
                    observe_tree(tree.report);
                    if !context.has_accounting() {
                        context
                            .poll()
                            .map_err(WorthQueryPartitionedComputationDenial::from_kernel_stop)?;
                        return tree.outcome;
                    }
                    let (work, span) = if let Ok(completed) = &tree.outcome {
                        completed
                            .checked_full_build_cost()
                            .expect("a completed build has bounded cost")
                    } else {
                        let metrics = tree
                            .report
                            .metrics()
                            .accounted_metrics()
                            .expect("accounted tree stage retains its accounting");
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
                        && context.remaining_work() == Some(0)
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
        work_receipt
            .spend(tree_bill.accounted_cost().map(|(work, _)| work))
            .map_err(|_| {
                WorthQueryPartitionedComputationDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkCounterOverflow,
                )
            })?;
        let computed_work = work_receipt.spent_since(preparation_work);
        // Like Full, only a successfully reduced computation settles a receipt.
        if let Some((observer, (work, span))) = observer.zip(settled_wave.accounting) {
            observer.record(work, span);
        }
        let reduced = tree.result().clone();
        // A call made again whose charge did not measure cannot be carried
        // again, so the run retains nothing.
        let call_sequence = call_recording::complete(
            &next,
            &partitions,
            patched_calls,
            gathers_measured,
            execution,
        )?;
        let typed = next
            .membership
            .filter(|_| gathers_measured)
            .map(|membership| RetainedPartitions {
                items: next.items,
                digests: next.digests,
                membership,
                item_keys: next.item_keys,
                call_sequence: Arc::clone(call_sequence.as_ref().unwrap()),
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
                    call_sequence: Arc::clone(call_sequence.as_ref().unwrap()),
                    typed: Arc::new(typed),
                    carried: Some(CarriedCalls {
                        validated_at,
                        prior: prior.into_state(),
                        membership: next.carried_membership,
                        items: next.carried_items,
                        partitions: carried,
                        ranges,
                    }),
                    attribution_memory: execution
                        .has_accounting()
                        .then(|| {
                            execution
                                .reserve(0)
                                .map_err(WorthQueryPartitionedComputationDenial::Resource)
                        })
                        .transpose()?,
                }),
            },
        })
    }
}
