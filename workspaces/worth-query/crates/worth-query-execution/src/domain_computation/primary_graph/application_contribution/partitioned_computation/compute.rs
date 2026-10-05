//! The compute and complete phases of a prepared partitioned computation.

use worth_execution::{
    ExecutionMap, ExecutionWorkCeiling, MapKernelContext, MapKernelFailure, MapKernelStop,
};
use worth_foundational::facade::ExecutionReport;
use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationDenial,
    WorthQueryManagedComputationExecution, WorthQueryManagedComputationInterruption,
    WorthQueryManagedComputationResourceDenial,
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
type Gathered<Schema, Feature, Computation, Owner> = GatheredComputationPartition<
    <Computation as ApplicationManagedComputation<Schema, Feature>>::Partition,
    <Owner as WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>>::Gathered,
>;

pub(super) fn units(declared: usize) -> u64 {
    u64::try_from(declared).unwrap_or(u64::MAX)
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
    pub(super) map: ExecutionMap<Gathered<Schema, Feature, Computation, Owner>, u64>,
    pub(super) prepared_work: u64,
    pub(super) remaining_work: u64,
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryPreparedPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    /// Computes every partition through the execution map and reduces the
    /// results over the canonical tree, inside what `prepare` left of the
    /// computation's declared work.
    ///
    /// No lease reaches a managed computation yet, so the serial backend runs
    /// every partition on the calling thread, in the order and with the
    /// charges any worker count would settle on.
    pub fn compute(
        self,
        execution: WorthQueryManagedComputationExecution<'_>,
    ) -> Result<
        WorthQueryCompletedPartitionedComputation<Schema, Feature, Computation, Owner>,
        Denial<Schema, Feature, Computation, Owner>,
    > {
        if let Some(interruption) = execution.interruption() {
            return Err(WorthQueryPartitionedComputationDenial::Interrupted(
                interruption,
            ));
        }
        let owner = &*self.installed.owner;
        let execution = &execution;
        let reducer = owner.reducer();
        let declared_bytes = units(Computation::RESOURCES.maximum_retained_bytes());
        let kernel = |partition: &Gathered<Schema, Feature, Computation, Owner>,
                      context: &mut MapKernelContext<'_, '_>| {
            let mut charge = |work| context.checkpoint(work);
            let mut checkpoint =
                WorthQueryManagedComputationCheckpoint::for_partition(&mut charge, execution);
            let computed = owner.compute_partition(
                WorthQueryComputationPartitionView::new(partition),
                &mut checkpoint,
            );
            match checkpoint.refused() {
                Some(refused) => Err(kernel_failure(refused.into())),
                None => computed.map_err(kernel_failure),
            }
        };
        let (reduced, _) = ExecutionWorkCeiling::new(self.remaining_work)
            .run(None, || {
                self.map.run_reduce(
                    None,
                    kernel,
                    (reducer.identity)(),
                    reducer.combine,
                    declared_bytes,
                    0,
                )
            })
            .map_err(WorthQueryPartitionedComputationDenial::from_work_ceiling)?;
        let (tree, report, _) =
            reduced.map_err(WorthQueryPartitionedComputationDenial::from_reduce)?;
        Ok(WorthQueryCompletedPartitionedComputation {
            installed: self.installed,
            reduced: tree.result().clone(),
            prepared_work: self.prepared_work,
            report,
        })
    }
}

/// What a partition's refusal is to the execution kernel.
fn kernel_failure<Stopped>(
    denial: WorthQueryManagedComputationDenial<Stopped>,
) -> MapKernelFailure<Stopped> {
    match denial {
        WorthQueryManagedComputationDenial::Owner(stopped) => MapKernelFailure::Domain(stopped),
        WorthQueryManagedComputationDenial::Resource(
            WorthQueryManagedComputationResourceDenial::WorkExhausted,
        ) => MapKernelFailure::Stop(MapKernelStop::WorkCeiling),
        WorthQueryManagedComputationDenial::Resource(
            WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
        ) => MapKernelFailure::ResultCapacityExceeded,
        WorthQueryManagedComputationDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled,
        ) => MapKernelFailure::Stop(MapKernelStop::Cancelled),
        WorthQueryManagedComputationDenial::Interrupted(
            WorthQueryManagedComputationInterruption::DeadlineExceeded,
        ) => MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed),
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
    prepared_work: u64,
    report: ExecutionReport,
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryCompletedPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    /// All the work the run was charged: deriving and routing the partition
    /// keys, every partition's kernel and every combine.
    pub fn charged_work(&self) -> u64 {
        self.prepared_work
            .saturating_add(self.report.charged_work())
    }

    /// What execution reports about the partitions and the reduction: their
    /// work and span, the resolved posture and why a run was serial.
    pub const fn execution_report(&self) -> ExecutionReport {
        self.report
    }

    /// Completes the reduced result on the calling thread.
    pub fn complete(self) -> Result<Owner::Output, Denial<Schema, Feature, Computation, Owner>> {
        self.installed
            .owner
            .complete(self.reduced)
            .map_err(WorthQueryPartitionedComputationDenial::Owner)
    }
}
