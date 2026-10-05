//! The compute and complete phases of a prepared partitioned computation.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use worth_execution::{
    ExecutionMap, ExecutionWorkCeiling, MapKernelContext, MapKernelFailure, MapKernelStop,
    PartitionItemId,
};
use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationDenial,
    WorthQueryManagedComputationExecution, WorthQueryManagedComputationInterruption,
    WorthQueryManagedComputationResourceDenial,
};
use super::incremental::{
    observe_unretained, ComputationDeposit, ComputedIncremental, FullRecording,
    PreparedIncremental, WorthQueryPartitionedComputationFullCause,
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
    pub(super) deposit: Option<ComputationDeposit>,
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
        remaining_work: u64,
        items: Arc<BTreeMap<PartitionItemId, Owner::Item>>,
        recording: Option<FullRecording<Computation::Partition>>,
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
        let declared_bytes = units(
            Computation::RESOURCES.maximum_retained_bytes(),
            WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
        )
        .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        let kernel_units = Mutex::new(BTreeMap::new());
        let kernel = |partition: &Gathered<Schema, Feature, Computation, Owner>,
                      context: &mut MapKernelContext<'_, '_>| {
            let mut spent = 0_u64;
            let (computed, refused) = {
                let mut charge = |work| {
                    context.checkpoint(work)?;
                    spent = spent
                        .checked_add(work)
                        .ok_or(MapKernelStop::WorkCounterOverflow)?;
                    Ok(())
                };
                let mut checkpoint =
                    WorthQueryManagedComputationCheckpoint::for_partition(&mut charge, execution);
                let computed = owner.compute_partition(
                    WorthQueryComputationPartitionView::new(partition),
                    &mut checkpoint,
                );
                (computed, checkpoint.refused())
            };
            kernel_units
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(partition.identity, spent);
            match refused {
                Some(refused) => Err(kernel_failure(refused.into())),
                None => computed.map_err(kernel_failure),
            }
        };
        let exhausted = || {
            WorthQueryPartitionedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            )
        };
        let (reduced, computed_work, completed) = match self.run {
            PreparedRun::Full {
                map,
                remaining_work,
                items,
                recording,
                cause,
            } => {
                let (reduced, _) = ExecutionWorkCeiling::new(remaining_work)
                    .run(None, || {
                        map.run_reduce(
                            None,
                            kernel,
                            (reducer.identity)(),
                            reducer.combine,
                            declared_bytes,
                            0,
                        )
                    })
                    .map_err(WorthQueryPartitionedComputationDenial::from_work_ceiling)?;
                let (tree, report, metrics) =
                    reduced.map_err(WorthQueryPartitionedComputationDenial::from_reduce)?;
                let reduced = tree.result().clone();
                let completed = match recording {
                    Some(recording) => Some(
                        recording.complete(
                            items,
                            kernel_units
                                .into_inner()
                                .unwrap_or_else(std::sync::PoisonError::into_inner),
                            tree,
                            metrics.charged_work,
                            cause,
                            report,
                        ),
                    ),
                    None => {
                        observe_unretained(cause, report);
                        None
                    }
                };
                (reduced, report.charged_work(), completed)
            }
            PreparedRun::Incremental(prepared) => {
                let ComputedIncremental {
                    reduced,
                    computed_work,
                    completed,
                } = prepared.compute(kernel, &reducer)?;
                (reduced, computed_work, Some(completed))
            }
        };
        let charged_work = self
            .prepared_work
            .checked_add(computed_work)
            .ok_or_else(exhausted)?;
        if let (Some(deposit), Some(completed)) = (self.deposit, completed) {
            *deposit
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(completed);
        }
        Ok(WorthQueryCompletedPartitionedComputation {
            installed: self.installed,
            reduced,
            charged_work,
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
    charged_work: u64,
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryCompletedPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    /// All the work the run was charged: digesting the input, deriving and
    /// routing the partition keys, every partition's kernel and every combine,
    /// whether a partition was computed again or carried from the last run.
    pub const fn charged_work(&self) -> u64 {
        self.charged_work
    }

    /// Completes the reduced result on the calling thread.
    pub fn complete(self) -> Result<Owner::Output, Denial<Schema, Feature, Computation, Owner>> {
        self.installed
            .owner
            .complete(self.reduced)
            .map_err(WorthQueryPartitionedComputationDenial::Owner)
    }
}
