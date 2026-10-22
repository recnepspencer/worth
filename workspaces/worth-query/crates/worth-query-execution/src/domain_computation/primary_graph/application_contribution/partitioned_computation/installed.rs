//! The installed partitioned owner and its prepare phase.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_execution::ExecutionMap;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationManagedComputation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::request_execution::QueryRequestExecution;
use super::super::WorthQueryManagedComputationResourceDenial;
use super::compute::{Denial, PreparedRun, WorthQueryPreparedPartitionedComputation};
use super::incremental::{
    self, Begun, ComputationInstallation, FullRecording, Suppression,
    WorthQueryPartitionedComputationFullCause,
};
use super::items::{input_digest, item_digests, planned_items, route_item};
use super::plan::{GatheredComputationPartition, PlanShape};
use super::preparation_work::PreparationWork;
use super::routing::ComputationPartitionRouting;
use super::{
    InputValue, WorthQueryComputationPartitionMembers, WorthQueryComputationReader,
    WorthQueryPartitionedComputationDenial, WorthQueryPartitionedComputationOwner,
};
use crate::domain_computation::primary_graph::application_attempt::ComputationRead;
use crate::domain_computation::primary_graph::{
    DecisionReader, WorthQueryApplicationOperationInvariantProjectionReader,
};

mod certification;
mod collection;

pub struct WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    pub(super) owner: Arc<Owner>,
    pub(super) completion: Option<fn(&Owner::Output) -> Result<(), Owner::Stopped>>,
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
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    fn clone(&self) -> Self {
        Self {
            owner: Arc::clone(&self.owner),
            completion: self.completion,
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
            completion: None,
            retention,
            installation: ComputationInstallation::new(),
            marker: PhantomData,
        }
    }

    /// Plans and gathers the input's partitions on the calling thread, reading
    /// through the handler's `reader`: names the items, digests each one when
    /// a producer runs the computation, derives every item's partition
    /// identity from its key, routes it and gathers each partition. Digests,
    /// key derivation and routing advance the original request's execution.
    /// The owner's reads are the operation's decision reads.
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
        let request = execution
            .execution_request()
            .map_err(WorthQueryPartitionedComputationDenial::Resource)?;
        let owner = &*self.owner;
        reader.begin_computation_reads();
        let mut preparation_work = PreparationWork::new(execution.has_accounting());
        let begun = match self.retention {
            ComputationRetention::Unretained => Begun::Full {
                basis: Err(Suppression::Policy),
                cause: WorthQueryPartitionedComputationFullCause::Unretained,
            },
            ComputationRetention::ProducerOperation => {
                let input_digest =
                    input_digest::<Computation::Input, _>(input, execution, &mut preparation_work)?;
                incremental::begin::<
                    Computation::Partition,
                    Owner::Item,
                    Owner::PartitionResult,
                    _,
                    _,
                >(&request, reader, &self.installation, input_digest)
                .map_err(|denial| {
                    WorthQueryPartitionedComputationDenial::Read(
                        super::reader::WorthQueryComputationReadDenial::RetainedFact(denial.kind()),
                    )
                })?
            }
        };
        let deposit = reader.computation_deposit();
        let (basis, cause) = match begun {
            Begun::Incremental(run) => {
                let prepared = run.prepare(owner, reader, execution, input, preparation_work)?;
                if let Some(prepared) = prepared {
                    return Ok(WorthQueryPreparedPartitionedComputation {
                        installed: self.clone(),
                        prepared_work: prepared.preparation_work().spent(),
                        run: PreparedRun::Incremental(prepared),
                        deposit,
                        kernel_observation_for_test: None,
                    });
                }
                // Routing again met a partition identity collision, and the
                // reader is back at the run's start. The run is made in full
                // with no record to reuse: the incremental run took it.
                (
                    Err(Suppression::Collision),
                    WorthQueryPartitionedComputationFullCause::IdentityCollision,
                )
            }
            Begun::Full { basis, cause } => (basis, cause),
        };
        incremental::observe_full_preparation(cause);
        let mut recording = basis.map(FullRecording::new);
        let (plan, membership) = reader.measured(ComputationRead::Membership, |reader| {
            owner.partitions(&mut WorthQueryComputationReader::lend(reader), input)
        });
        let PlanShape::Keyed(entries) = plan?.shape;
        let items = planned_items(entries)?;
        if self.retention == ComputationRetention::ProducerOperation {
            let before = preparation_work;
            let digests = item_digests(&items, execution, &mut preparation_work)?;
            if let Ok(recording) = &mut recording {
                recording.membership(
                    membership.map(|charge| (charge, preparation_work.spent_since(before))),
                    &items,
                    digests,
                );
            }
        }
        let mut routing = ComputationPartitionRouting::default();
        let mut keys = BTreeMap::new();
        for (item, value) in items.iter() {
            let (key, charge) = reader.measured(ComputationRead::ItemKey(*item), |reader| {
                owner.partition_key(&mut WorthQueryComputationReader::lend(reader), input, value)
            });
            let key = key?;
            let before = preparation_work;
            let partition =
                route_item(*item, &key, &mut routing, execution, &mut preparation_work)?;
            if let Ok(recording) = &mut recording {
                recording.item(
                    *item,
                    charge.map(|charge| (charge, preparation_work.spent_since(before))),
                );
            }
            keys.entry(partition).or_insert(key);
        }
        let mut partitions = BTreeMap::new();
        for (identity, key) in keys {
            let key = Arc::new(key);
            let members = routing.members(identity).collect::<Arc<[_]>>();
            execution
                .checkpoint()
                .map_err(WorthQueryPartitionedComputationDenial::from_kernel_stop)?;
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
            if let Ok(recording) = &mut recording {
                recording.partition(identity, &key, charge);
            }
            let value = GatheredComputationPartition {
                identity,
                key,
                items: members,
                gathered,
            };
            partitions.insert(identity, value);
        }
        if let Ok(recording) = &mut recording {
            recording.routing(routing);
        }
        let map = ExecutionMap::<_, ()>::from_native_partitions(partitions).map_err(|kind| {
            WorthQueryPartitionedComputationDenial::Resource(
                WorthQueryManagedComputationResourceDenial::NativeAllocationRefused {
                    kind,
                    requested_payload_bytes: None,
                },
            )
        })?;
        Ok(WorthQueryPreparedPartitionedComputation {
            installed: self.clone(),
            prepared_work: preparation_work.spent(),
            run: PreparedRun::Full {
                map,
                compute_ceiling: None,
                recording,
                cause,
            },
            deposit,
            kernel_observation_for_test: None,
        })
    }
}
