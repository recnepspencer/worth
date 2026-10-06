//! The installed partitioned owner and its prepare phase.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_execution::{ExecutionMap, KeylessPartition};
use worth_query_declaration::facade::application_operation::{
    application_computation_input_digest, application_computation_partition_identity,
    ApplicationComputationPartitionIdentityDenial, ApplicationMutationBinding,
};
use worth_query_declaration::facade::application_program::{
    ApplicationComputationInput, ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::request_execution::QueryRequestExecution;
use super::super::WorthQueryManagedComputationResourceDenial;
use super::compute::{units, Denial, PreparedRun, WorthQueryPreparedPartitionedComputation};
use super::gather_memory::GatheredMemory;
use super::incremental::{
    self, Begun, ComputationInstallation, FullRecording, WorthQueryPartitionedComputationFullCause,
};
use super::plan::{GatheredComputationPartition, PlanShape};
use super::remaining_work::{EncodingMeter, RemainingWork};
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
    retention: ComputationRetention,
    installation: ComputationInstallation,
    marker: PhantomData<fn() -> (Schema, Feature, Computation)>,
}

/// Whether a run can be retained, fixed at installation: only a producer
/// hands a run the prior it compares with, so only a run of a producer's
/// operation digests its input. No other run does that work or pays for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum ComputationRetention {
    /// A declared producer runs the owner's operation.
    ProducerOperation,
    /// No producer runs it, so no run is ever compared.
    Unretained,
}

impl<Schema, Feature, Computation, Owner> Clone
    for WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>
{
    fn clone(&self) -> Self {
        Self {
            owner: Arc::clone(&self.owner),
            retention: self.retention,
            installation: self.installation.clone(),
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
    pub(in crate::domain_computation::primary_graph) fn new(
        owner: Owner,
        retention: ComputationRetention,
    ) -> Self {
        Self {
            owner: Arc::new(owner),
            retention,
            installation: ComputationInstallation::new(),
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
        let execution = reader.execution();
        self.prepare_through(reader.operation_reader(), execution, input)
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_through(
        &self,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Schema,
            Owner::Operation,
        >,
        execution: &QueryRequestExecution<'_>,
        input: &InputValue<Schema, Feature, Computation>,
    ) -> Result<
        WorthQueryPreparedPartitionedComputation<Schema, Feature, Computation, Owner>,
        Denial<Schema, Feature, Computation, Owner>,
    > {
        let owner = &*self.owner;
        reader.begin_computation_reads();
        let mut remaining_work = RemainingWork::declared(
            units(
                Computation::RESOURCES.maximum_work(),
                WorthQueryManagedComputationResourceDenial::WorkExhausted,
            )
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?,
        );
        let declared_bytes = units(
            Computation::RESOURCES.maximum_retained_bytes(),
            WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
        )
        .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        let begun = match self.retention {
            ComputationRetention::Unretained => Begun::Full {
                basis: None,
                cause: WorthQueryPartitionedComputationFullCause::NoPriorRecord,
            },
            ComputationRetention::ProducerOperation => {
                let input_digest = input_digest::<Computation::Input, _>(
                    input,
                    &mut remaining_work,
                    declared_bytes,
                )?;
                incremental::begin::<
                    Computation::Partition,
                    Owner::Item,
                    Owner::PartitionResult,
                    _,
                    _,
                >(reader, &self.installation, input_digest, remaining_work)
            }
        };
        let deposit = reader.computation_deposit();
        let (basis, cause) = match begun {
            Begun::Incremental(run) => {
                let prepared = run.prepare(
                    owner,
                    reader,
                    execution,
                    input,
                    remaining_work,
                    declared_bytes,
                )?;
                return Ok(WorthQueryPreparedPartitionedComputation {
                    installed: self.clone(),
                    prepared_work: prepared.remaining_work().spent(),
                    run: PreparedRun::Incremental(prepared),
                    deposit,
                });
            }
            Begun::Full { basis, cause } => (basis, cause),
        };
        let mut recording = FullRecording::new(basis);
        let (plan, membership) = reader.measured(ComputationRead::Membership, |reader| {
            owner.partitions(&mut WorthQueryComputationReader::lend(reader), input)
        });
        let PlanShape::Keyed(mut entries) = plan?.shape;
        if let Some(recording) = &mut recording {
            recording.membership(membership);
        }
        let mut routing = ComputationPartitionRouting::default();
        let mut routing_memory = execution
            .reserve(0)
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        let mut items = BTreeMap::new();
        let mut keys = BTreeMap::new();
        entries.sort_by_key(|(item, _)| *item);
        for (item, value) in entries {
            let (key, charge) = reader.measured(ComputationRead::ItemKey(item), |reader| {
                owner.partition_key(
                    &mut WorthQueryComputationReader::lend(reader),
                    input,
                    &value,
                )
            });
            let key = key?;
            let before = remaining_work;
            let mut meter = EncodingMeter::new(&mut remaining_work, declared_bytes);
            let derived =
                application_computation_partition_identity(&key, &mut |charge| meter.admit(charge))
                    .map_err(|denial| match denial {
                        ApplicationComputationPartitionIdentityDenial::Key(denial) => {
                            WorthQueryPartitionedComputationDenial::KeyNotEncodable { item, denial }
                        }
                        denial => encoding_resource_denial(denial),
                    })?;
            let (partition, routed) = routing.route(item, *derived.digest(), |bound| {
                routing_memory.resize(bound)
            })?;
            remaining_work
                .spend(routed.units())
                .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
            if let Some(recording) = &mut recording {
                recording.item(item, charge.zip(remaining_work.spent_since(before)));
            }
            items.insert(item, value);
            let key_bytes = u64::try_from(derived.work().encoded_bytes()).map_err(|_| {
                WorthQueryPartitionedComputationDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
                )
            })?;
            keys.entry(partition).or_insert((key, key_bytes));
        }
        let items = Arc::new(items);
        let mut memory = GatheredMemory::new(execution, declared_bytes)?;
        let mut partitions = BTreeMap::new();
        for (identity, (key, key_bytes)) in keys {
            let key = Arc::new(key);
            let members = routing.members(identity).collect::<Arc<[_]>>();
            memory.before_gather(execution)?;
            let (gathered, charge) =
                reader.measured(ComputationRead::Partition(identity), |reader| {
                    owner.gather(
                        &mut WorthQueryComputationReader::lend(reader),
                        input,
                        WorthQueryComputationPartitionMembers::new(
                            identity, &key, &members, &items,
                        ),
                    )
                });
            let gathered = gathered.map_err(|denial| {
                WorthQueryPartitionedComputationDenial::gathering(identity, denial)
            })?;
            if let Some(recording) = &mut recording {
                recording.partition(identity, &key, key_bytes, &members, charge);
            }
            let value = GatheredComputationPartition {
                identity,
                key,
                items: members,
                gathered,
            };
            memory.after_gather(&value)?;
            partitions.insert(
                identity,
                KeylessPartition {
                    value,
                    kernel_scratch_bytes: 0,
                    max_result_bytes: declared_bytes,
                },
            );
        }
        let map = ExecutionMap::from_keyless_partitions(partitions)
            .map_err(WorthQueryPartitionedComputationDenial::from_map_overflow)?;
        Ok(WorthQueryPreparedPartitionedComputation {
            installed: self.clone(),
            prepared_work: remaining_work.spent(),
            run: PreparedRun::Full {
                map,
                memory,
                remaining_work: remaining_work.remaining(),
                items,
                recording,
                cause,
            },
            deposit,
        })
    }
}

/// The digest of a producer run's input. One meter admits the whole
/// encoding, so its scratch growths are summed as a partition key's are.
pub(super) fn input_digest<Input, Stopped>(
    input: &Input::Value,
    remaining_work: &mut RemainingWork,
    declared_bytes: u64,
) -> Result<[u8; 32], WorthQueryPartitionedComputationDenial<Stopped>>
where
    Input: ApplicationComputationInput,
{
    let mut meter = EncodingMeter::new(remaining_work, declared_bytes);
    application_computation_input_digest::<Input, _, _>(input, &mut |charge| meter.admit(charge))
        .map_err(|denial| match denial {
            ApplicationComputationPartitionIdentityDenial::Key(denial) => {
                WorthQueryPartitionedComputationDenial::InputNotEncodable(denial)
            }
            denial => encoding_resource_denial(denial),
        })
}

/// An encoding refused for its resources, not its value.
fn encoding_resource_denial<Stopped>(
    denial: ApplicationComputationPartitionIdentityDenial<
        WorthQueryManagedComputationResourceDenial,
    >,
) -> WorthQueryPartitionedComputationDenial<Stopped> {
    match denial {
        ApplicationComputationPartitionIdentityDenial::Admission(denial) => {
            WorthQueryPartitionedComputationDenial::Resource(denial)
        }
        ApplicationComputationPartitionIdentityDenial::Key(_)
        | ApplicationComputationPartitionIdentityDenial::CapacityOverflow
        | ApplicationComputationPartitionIdentityDenial::Allocation => {
            WorthQueryPartitionedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
            )
        }
    }
}
