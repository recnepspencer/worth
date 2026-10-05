//! The installed partitioned owner and its prepare, compute, complete phases.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_execution::{
    ExecutionMap, ExecutionWorkCeiling, MapKernelContext, MapKernelFailure, MapKernelStop,
    MapPartition,
};
use worth_foundational::facade::ExecutionReport;
use worth_query_declaration::facade::application_operation::{
    application_computation_partition_identity, ApplicationComputationPartitionIdentityDenial,
    CanonicalEncodingCharge,
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
use super::plan::{ComputationPartitionMembers, PlanShape};
use super::routing::ComputationPartitionRouting;
use super::{
    InputValue, WorthQueryComputationPartitionItem, WorthQueryComputationPartitionView,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
};

type Denial<Schema, Feature, Computation, Owner> = WorthQueryPartitionedComputationDenial<
    <Owner as WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>>::Stopped,
>;

fn units(declared: usize) -> u64 {
    u64::try_from(declared).unwrap_or(u64::MAX)
}

pub struct WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner> {
    owner: Arc<Owner>,
    marker: PhantomData<fn() -> (Schema, Feature, Computation)>,
}

impl<Schema, Feature, Computation, Owner> Clone
    for WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>
{
    fn clone(&self) -> Self {
        Self {
            owner: Arc::clone(&self.owner),
            marker: PhantomData,
        }
    }
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    pub(in crate::domain_computation::primary_graph) fn new(owner: Owner) -> Self {
        Self {
            owner: Arc::new(owner),
            marker: PhantomData,
        }
    }

    /// Plans the input's partitions on the calling thread: derives every
    /// item's partition identity from its key, routes it and gathers each
    /// partition's members. Key derivation and routing are charged against
    /// the computation's declared work.
    ///
    /// The items are met in item identity order, so everything planning
    /// decides, a denial included, follows from the items and not from the
    /// order the input holds them in.
    pub fn prepare<'input>(
        &self,
        input: &'input InputValue<Schema, Feature, Computation>,
    ) -> Result<
        WorthQueryPreparedPartitionedComputation<'input, Schema, Feature, Computation, Owner>,
        Denial<Schema, Feature, Computation, Owner>,
    > {
        let PlanShape::Keyed(entries) = self.owner.partitions(input).shape;
        let declared_work = units(Computation::RESOURCES.maximum_work());
        let declared_bytes = units(Computation::RESOURCES.maximum_retained_bytes());
        let mut remaining_work = declared_work;
        let mut routing = ComputationPartitionRouting::default();
        let mut positions = BTreeMap::new();
        let mut partitions = BTreeMap::new();
        let mut entries = entries.into_iter().enumerate().collect::<Vec<_>>();
        entries.sort_by_key(|(_, (item, _))| *item);
        for (position, (item, key)) in entries {
            let mut scratch = 0_u64;
            let derived =
                application_computation_partition_identity(&key, &mut |charge| match charge {
                    CanonicalEncodingCharge::Work(work) => spend(&mut remaining_work, Some(work)),
                    CanonicalEncodingCharge::Scratch(bytes) => {
                        scratch = scratch
                            .checked_add(bytes)
                            .filter(|held| *held <= declared_bytes)
                            .ok_or(
                                WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
                            )?;
                        Ok(())
                    }
                })
                .map_err(|denial| match denial {
                    ApplicationComputationPartitionIdentityDenial::Key(denial) => {
                        WorthQueryPartitionedComputationDenial::KeyNotEncodable { item, denial }
                    }
                    ApplicationComputationPartitionIdentityDenial::Admission(denial) => {
                        WorthQueryPartitionedComputationDenial::Resource(denial)
                    }
                    ApplicationComputationPartitionIdentityDenial::CapacityOverflow
                    | ApplicationComputationPartitionIdentityDenial::Allocation => {
                        WorthQueryPartitionedComputationDenial::Resource(
                            WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
                        )
                    }
                })?;
            let (partition, routed) = routing.route(item, *derived.digest())?;
            spend(&mut remaining_work, routed.units())
                .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
            positions.insert(item, position);
            partitions
                .entry(partition)
                .or_insert_with(|| ComputationPartitionMembers {
                    identity: partition,
                    key,
                    items: Vec::new(),
                });
        }
        let identities = partitions.keys().copied().collect();
        let partitions = partitions
            .into_values()
            .map(|mut members| {
                members.items = routing
                    .members(members.identity)
                    .map(|item| WorthQueryComputationPartitionItem::new(item, positions[&item]))
                    .collect();
                MapPartition {
                    identity: members.identity,
                    value: members,
                    read_keys: Vec::new(),
                    write_keys: Vec::new(),
                    kernel_scratch_bytes: 0,
                    max_result_bytes: declared_bytes,
                }
            })
            .collect();
        let map = ExecutionMap::try_from_declared_partitions(identities, partitions)
            .map_err(WorthQueryPartitionedComputationDenial::from_map)?;
        Ok(WorthQueryPreparedPartitionedComputation {
            installed: self.clone(),
            input,
            map,
            prepared_work: declared_work - remaining_work,
            remaining_work,
        })
    }
}

/// Spends declared work, refusing what does not fit. `None` is a count that
/// overflowed, which fits no ceiling.
fn spend(
    remaining: &mut u64,
    work: Option<u64>,
) -> Result<(), WorthQueryManagedComputationResourceDenial> {
    *remaining = work
        .and_then(|work| remaining.checked_sub(work))
        .ok_or(WorthQueryManagedComputationResourceDenial::WorkExhausted)?;
    Ok(())
}

pub struct WorthQueryPreparedPartitionedComputation<'input, Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    installed: WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>,
    input: &'input InputValue<Schema, Feature, Computation>,
    map: ExecutionMap<ComputationPartitionMembers<Computation::Partition>, u64>,
    prepared_work: u64,
    remaining_work: u64,
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryPreparedPartitionedComputation<'_, Schema, Feature, Computation, Owner>
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
        let input = self.input;
        let execution = &execution;
        let reducer = owner.reducer();
        let declared_bytes = units(Computation::RESOURCES.maximum_retained_bytes());
        let kernel = |members: &ComputationPartitionMembers<Computation::Partition>,
                      context: &mut MapKernelContext<'_, '_>| {
            let mut charge = |work| context.checkpoint(work);
            let mut checkpoint =
                WorthQueryManagedComputationCheckpoint::for_partition(&mut charge, execution);
            let computed = owner.compute_partition(
                WorthQueryComputationPartitionView::new(members, input),
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
