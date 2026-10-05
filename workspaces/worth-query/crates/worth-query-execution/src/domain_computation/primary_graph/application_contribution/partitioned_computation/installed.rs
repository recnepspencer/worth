//! The installed partitioned owner and its prepare phase.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_execution::{ExecutionMap, MapPartition};
use worth_query_declaration::facade::application_operation::{
    application_computation_partition_identity, ApplicationComputationPartitionIdentityDenial,
    ApplicationMutationBinding, CanonicalEncodingCharge,
};
use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryManagedComputationResourceDenial;
use super::compute::{units, Denial, WorthQueryPreparedPartitionedComputation};
use super::plan::{GatheredComputationPartition, PlanShape};
use super::routing::ComputationPartitionRouting;
use super::{
    InputValue, WorthQueryComputationPartitionMembers, WorthQueryComputationReader,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
};
use crate::domain_computation::primary_graph::application_attempt::ComputationRead;
use crate::domain_computation::primary_graph::{
    DecisionReader, WorthQueryApplicationOperationInvariantProjectionReader,
};

pub struct WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner> {
    pub(super) owner: Arc<Owner>,
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

    /// Plans and gathers the input's partitions on the calling thread, reading
    /// through the handler's `reader`: names the items, derives every item's
    /// partition identity from its key, routes it and gathers each partition.
    /// Key derivation and routing are charged against the computation's
    /// declared work. The owner's reads are the operation's decision reads.
    ///
    /// The items are met in item identity order and the partitions in
    /// partition identity order, so everything planning decides, a denial
    /// included, follows from the items and not from the order the input
    /// holds them in.
    pub fn prepare<Binding>(
        &self,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
        input: &InputValue<Schema, Feature, Computation>,
    ) -> Result<
        WorthQueryPreparedPartitionedComputation<Schema, Feature, Computation, Owner>,
        Denial<Schema, Feature, Computation, Owner>,
    >
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Owner::Operation>,
    {
        self.prepare_through(reader.operation_reader(), input)
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_through(
        &self,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Schema,
            Owner::Operation,
        >,
        input: &InputValue<Schema, Feature, Computation>,
    ) -> Result<
        WorthQueryPreparedPartitionedComputation<Schema, Feature, Computation, Owner>,
        Denial<Schema, Feature, Computation, Owner>,
    > {
        let owner = &*self.owner;
        reader.begin_computation_reads();
        let PlanShape::Keyed(mut entries) = reader
            .attributed(ComputationRead::Membership, |reader| {
                owner.partitions(&mut WorthQueryComputationReader::lend(reader), input)
            })?
            .shape;
        let declared_work = units(Computation::RESOURCES.maximum_work());
        let declared_bytes = units(Computation::RESOURCES.maximum_retained_bytes());
        let mut remaining_work = declared_work;
        let mut routing = ComputationPartitionRouting::default();
        let mut items = BTreeMap::new();
        let mut keys = BTreeMap::new();
        entries.sort_by_key(|(item, _)| *item);
        for (item, value) in entries {
            let key = reader.attributed(ComputationRead::ItemKey(item), |reader| {
                owner.partition_key(
                    &mut WorthQueryComputationReader::lend(reader),
                    input,
                    &value,
                )
            })?;
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
            items.insert(item, value);
            keys.entry(partition).or_insert(key);
        }
        let identities = keys.keys().copied().collect();
        let mut partitions = Vec::with_capacity(keys.len());
        for (identity, key) in keys {
            let members = routing.members(identity).collect::<Vec<_>>();
            let gathered = reader
                .attributed(ComputationRead::Partition(identity), |reader| {
                    owner.gather(
                        &mut WorthQueryComputationReader::lend(reader),
                        input,
                        WorthQueryComputationPartitionMembers::new(
                            identity, &key, &members, &items,
                        ),
                    )
                })
                .map_err(|denial| {
                    WorthQueryPartitionedComputationDenial::gathering(identity, denial)
                })?;
            partitions.push(MapPartition {
                identity,
                value: GatheredComputationPartition {
                    identity,
                    key,
                    items: members,
                    gathered,
                },
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 0,
                max_result_bytes: declared_bytes,
            });
        }
        let map = ExecutionMap::try_from_declared_partitions(identities, partitions)
            .map_err(WorthQueryPartitionedComputationDenial::from_map)?;
        Ok(WorthQueryPreparedPartitionedComputation {
            installed: self.clone(),
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
