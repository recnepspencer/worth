//! The compute and complete phases of a prepared partitioned computation.

use std::collections::BTreeMap;
use std::sync::Mutex;

use worth_execution::{ExecutionMap, MapKernelContext, MapKernelStop};
use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

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
                memory,
                remaining_work,
                recording,
                cause,
            } => {
                let dispatch = execution
                    .request
                    .dispatch()
                    .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
                // The map's admission takes over the gathered partitions' hold,
                // and the tree is handed to `tree_memory` as the run ends.
                let mut tree_memory = execution
                    .request
                    .reserve(0)
                    .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
                let reduced = dispatch
                    .reduce(
                        remaining_work,
                        &map,
                        memory.into_held(),
                        &mut tree_memory,
                        kernel,
                        (reducer.identity)(),
                        reducer.combine,
                        declared_bytes,
                    )
                    .map_err(|denial| {
                        // The reduce pattern contains its kernels' panics:
                        // what is left is the reducer's.
                        WorthQueryPartitionedComputationDenial::from_work_ceiling(
                            denial,
                            WorthQueryPartitionedComputationDenial::ReducerPanicked,
                        )
                    })?;
                let (tree, report, metrics) =
                    reduced.map_err(WorthQueryPartitionedComputationDenial::from_reduce)?;
                let reduced = tree.result().clone();
                let completed = match recording {
                    Ok(recording) => recording.complete(
                        kernel_units
                            .into_inner()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                        tree,
                        tree_memory,
                        metrics.charged_work,
                        cause,
                        report,
                    ),
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
                } = prepared.compute(execution.request, kernel, &reducer)?;
                (reduced, computed_work, completed)
            }
        };
        let charged_work = self
            .prepared_work
            .checked_add(computed_work)
            .ok_or_else(exhausted)?;
        self.deposit.write(completed);
        Ok(WorthQueryCompletedPartitionedComputation {
            installed: self.installed,
            reduced,
            charged_work,
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
