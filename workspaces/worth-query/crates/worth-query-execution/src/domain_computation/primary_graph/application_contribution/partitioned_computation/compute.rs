//! The compute and complete phases of a prepared partitioned computation.

mod completion;
mod decision_plan;
pub(super) mod kernel_observation;
mod transport;

use worth_execution::{ExecutionMap, MapKernelContext};
use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::computation::nested_failure::{MemberComputeFailure, NestedFailureTransport};
use super::super::execution_denial::kernel_failure;
use super::super::{
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationExecution,
    WorthQueryManagedComputationResourceDenial,
};
use super::gather_memory::GatheredMemory;
use super::incremental::{
    observe_unretained, CompletedComputationRetention, ComputationDeposit, ComputedIncremental,
    FullRecording, PreparedIncremental, PriorAbsence, Suppression,
    WorthQueryPartitionedComputationFullCause,
};
use super::plan::GatheredComputationPartition;
use super::{
    WorthQueryComputationPartitionView, WorthQueryInstalledPartitionedComputation,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
};

pub(super) type Denial<Schema, Feature, Computation, Owner> =
    WorthQueryPartitionedComputationDenial<
        <Owner as WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>>::Stopped,
    >;

/// One gathered partition of `Owner`'s computation.
pub(super) type Gathered<Schema, Feature, Computation, Owner> = GatheredComputationPartition<
    <Computation as ApplicationManagedComputation<Schema, Feature>>::Partition,
    <Owner as WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>>::Gathered,
>;

/// A declared amount as execution units, refused as `refused` when it does
/// not fit.
pub(super) fn units(
    declared: usize,
    refused: WorthQueryManagedComputationResourceDenial,
) -> Result<u64, WorthQueryManagedComputationResourceDenial> {
    u64::try_from(declared).map_err(|_| refused)
}

pub struct WorthQueryPreparedPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    pub(super) installed:
        WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>,
    /// The declared work the input digest and the item keys spent.
    pub(super) prepared_work: u64,
    pub(super) run: PreparedRun<Schema, Feature, Computation, Owner>,
    /// Where the completed run is left for seal, when a producer runs it.
    pub(super) deposit: ComputationDeposit,
    pub(super) kernel_observation_for_test:
        Option<kernel_observation::KernelCostObservationForTest>,
}

/// Every partition gathered, or only the marked ones.
pub(super) enum PreparedRun<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    Full {
        map: ExecutionMap<Gathered<Schema, Feature, Computation, Owner>, u64>,
        /// The request memory the gathered partitions hold until the map's
        /// admission takes it over.
        memory: GatheredMemory,
        remaining_work: u64,
        recording: Result<FullRecording<Computation::Partition, Owner::Item>, Suppression>,
        cause: WorthQueryPartitionedComputationFullCause,
    },
    Incremental(
        PreparedIncremental<
            Computation::Partition,
            Owner::Item,
            Owner::PartitionResult,
            Owner::Gathered,
        >,
    ),
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryPreparedPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    /// Restrict this owned prepared run's compute allowance for certification.
    #[doc(hidden)]
    pub fn with_compute_ceiling_for_test(mut self, ceiling: u64) -> Self {
        crate::domain_computation::execution_observation::owned_control_scope::current_id();
        match &mut self.run {
            PreparedRun::Full { remaining_work, .. } => {
                *remaining_work = (*remaining_work).min(ceiling)
            }
            PreparedRun::Incremental(prepared) => prepared.restrict_compute_ceiling(ceiling),
        }
        self
    }

    /// Observe this receiver's full-equivalent map bill without a thread recorder.
    #[doc(hidden)]
    pub fn observe_kernel_cost_for_test(
        mut self,
        report: impl Fn(u64, u64) + Send + Sync + 'static,
    ) -> Self {
        self.kernel_observation_for_test = Some(
            kernel_observation::KernelCostObservationForTest::new(report),
        );
        self
    }

    /// Computes every partition through the execution map and reduces the
    /// results over the canonical tree, inside what `prepare` left of the
    /// computation's declared work.
    ///
    /// The run is placed by the request's execution: on a child of the
    /// request's lease, or serially within the request's policy. Either way
    /// the partitions settle in identity order with the charges any worker
    /// count would settle on.
    pub fn compute(
        self,
        execution: WorthQueryManagedComputationExecution<'_>,
    ) -> Result<
        WorthQueryCompletedPartitionedComputation<Schema, Feature, Computation, Owner>,
        Denial<Schema, Feature, Computation, Owner>,
    > {
        // A standalone compute has no wave kernel's latch to acknowledge.
        let mut take = || None;
        self.compute_with_transport(execution, &mut NestedFailureTransport::new(&mut take))
            .map_err(MemberComputeFailure::into_failure)
    }

    fn compute_with_transport(
        self,
        execution: WorthQueryManagedComputationExecution<'_>,
        transport: &mut NestedFailureTransport<'_>,
    ) -> Result<
        WorthQueryCompletedPartitionedComputation<Schema, Feature, Computation, Owner>,
        MemberComputeFailure<Denial<Schema, Feature, Computation, Owner>>,
    > {
        if let Some(interruption) = execution.interruption() {
            return Err(WorthQueryPartitionedComputationDenial::Interrupted(interruption).into());
        }
        let owner = &*self.installed.owner;
        let execution = &execution;
        let reducer = owner.reducer();
        let declared_bytes = units(
            Computation::RESOURCES.maximum_retained_bytes(),
            WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
        )
        .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        let compute_kernel = |partition: &Gathered<Schema, Feature, Computation, Owner>,
                              context: &mut MapKernelContext<'_, '_>,
                              #[cfg(test)] native: Option<
            &worth_execution::ExecutionResourceLease<'_>,
        >| {
            let (computed, refused) = {
                let mut charge = |work| {
                    context.checkpoint(work)?;
                    Ok(())
                };
                let checkpoint =
                    WorthQueryManagedComputationCheckpoint::for_partition(&mut charge, execution);
                #[cfg(test)]
                let checkpoint = checkpoint.with_native_lease_for_test(native);
                let mut checkpoint = checkpoint;
                let computed = owner.compute_partition(
                    WorthQueryComputationPartitionView::new(partition),
                    &mut checkpoint,
                );
                (computed, checkpoint.refused())
            };
            match refused {
                Some(refused) => Err(kernel_failure(refused.into())),
                None => computed.map_err(kernel_failure),
            }
        };
        let kernel = |partition: &Gathered<Schema, Feature, Computation, Owner>,
                      context: &mut MapKernelContext<'_, '_>| {
            compute_kernel(
                partition,
                context,
                #[cfg(test)]
                None,
            )
        };
        // Certification reruns the same owner without touching request retention
        // or its memory reservations. The original native kernel alone issues receipts.
        let diagnostic_kernel =
            |partition: &Gathered<Schema, Feature, Computation, Owner>,
             context: &mut MapKernelContext<'_, '_>| {
                compute_kernel(
                    partition,
                    context,
                    #[cfg(test)]
                    None,
                )
            };
        let exhausted = || {
            WorthQueryPartitionedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            )
        };
        let (reduced, computed_work, completed) = match self.run {
            PreparedRun::Full {
                map,
                memory,
                remaining_work,
                recording,
                cause,
            } => {
                let dispatch = execution
                    .request
                    .dispatch()
                    .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
                #[cfg(test)]
                let kernel = |partition: &Gathered<Schema, Feature, Computation, Owner>,
                              context: &mut MapKernelContext<'_, '_>| {
                    compute_kernel(partition, context, Some(dispatch.native_lease_for_test()))
                };
                // The map's admission takes over the gathered partitions' hold,
                // and the tree is handed to `tree_memory` as the run ends.
                let mut tree_memory = execution
                    .request
                    .reserve(0)
                    .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
                let (tree, report, metrics, partition_results) = transport.call(|| {
                    let reduced = dispatch
                        .reduce_retained(
                            remaining_work,
                            &map,
                            memory.into_held(),
                            &mut tree_memory,
                            (kernel, diagnostic_kernel),
                            (reducer.identity)(),
                            reducer.combine,
                            declared_bytes,
                            cause,
                        )
                        .map_err(WorthQueryPartitionedComputationDenial::from_retained_reduce)?;
                    Ok((reduced.tree, reduced.report, reduced.metrics, reduced.run))
                })?;
                if let Some(observer) = &self.kernel_observation_for_test {
                    observer.record(
                        report
                            .charged_work()
                            .checked_sub(metrics.charged_work)
                            .expect("tree work is included"),
                        report
                            .charged_span()
                            .checked_sub(metrics.charged_span)
                            .expect("tree span is included"),
                    );
                }
                let reduced = tree.result().clone();
                let completed = match recording {
                    Ok(recording) => recording.complete(
                        partition_results,
                        tree,
                        tree_memory,
                        execution.request,
                        cause,
                        report,
                    )?,
                    Err(reason) => {
                        observe_unretained(cause, report);
                        CompletedComputationRetention::Absent(PriorAbsence::Suppressed(reason))
                    }
                };
                (reduced, report.charged_work(), completed)
            }
            PreparedRun::Incremental(prepared) => {
                let ComputedIncremental {
                    reduced,
                    computed_work,
                    completed,
                } = prepared.compute(
                    execution.request,
                    kernel,
                    self.kernel_observation_for_test.as_ref(),
                    &reducer,
                    transport,
                )?;
                (reduced, computed_work, completed)
            }
        };
        let charged_work = self
            .prepared_work
            .checked_add(computed_work)
            .ok_or_else(exhausted)?;
        Ok(WorthQueryCompletedPartitionedComputation {
            installed: self.installed,
            reduced,
            charged_work,
            completed,
            deposit: self.deposit,
        })
    }
}

pub struct WorthQueryCompletedPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    installed: WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>,
    reduced: Owner::PartitionResult,
    charged_work: u64,
    completed: CompletedComputationRetention,
    deposit: ComputationDeposit,
}
