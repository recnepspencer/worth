//! Which owner calls the sealed read set keeps for each fact, for an owner
//! reading through the reader of a real admitted operation.

use std::sync::Mutex;
use serde::Serialize;
use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_program::ApplicationArtifactDependency;
use worth_query_declaration::facade::application_program::ApplicationArtifactRetention;
use worth_query_declaration::facade::application_program::ApplicationArtifactSuccession;
use worth_query_declaration::facade::application_program::ApplicationComputationExecution;
use worth_query_declaration::facade::application_program::ApplicationComputationInput;
use worth_query_declaration::facade::application_program::ApplicationComputationPartition;
use worth_query_declaration::facade::application_program::ApplicationComputationReuse;
use worth_query_declaration::facade::application_program::ApplicationComputationStopped;
use worth_query_declaration::facade::application_program::ApplicationDerivedArtifact;
use worth_query_declaration::facade::application_program::ApplicationFeature;
use worth_query_declaration::facade::application_program::ApplicationFeatureInputLeaf;
use worth_query_declaration::facade::application_program::ApplicationLocalityGranule;
use worth_query_declaration::facade::application_program::ApplicationLocalityScope;
use worth_query_declaration::facade::application_program::ApplicationManagedComputation;
use worth_query_declaration::facade::application_program::ApplicationOutputPort;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_declaration::facade::application_schema::ApplicationValueValidationDenial;
use worth_query_execution::facade::application_contribution::WorthQueryComputationInputDenial;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionMembers;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionPlan;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionView;
use worth_query_execution::facade::application_contribution::WorthQueryComputationReader;
use worth_query_execution::facade::application_contribution::WorthQueryDeterministicReducer;
use worth_query_execution::facade::application_contribution::WorthQueryInstalledPartitionedComputation;
use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationOwner;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationFactReaders;
use crate::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use crate::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use crate::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema as Schema;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantEntityIdentity;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationCheckpoint;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationDenial;
pub struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.tests.attribution-feature.v1";
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value(u64);
worth_query_declaration::worth_query_portable_type!(
    Value => "worth.query.tests.attribution-value.v1"
);
pub struct ValueBinding;
impl ApplicationStructuredValueBinding for ValueBinding {
    type Value = Value;
    const IDENTITY_NAME: &'static str = "worth.query.tests.attribution-value.v1";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
pub struct Output;
impl ApplicationOutputPort<Schema, Feature> for Output {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "worth.query.tests.attribution-output.v1";
}
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "worth.query.tests.attribution-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}
pub struct Artifact;
impl ApplicationDerivedArtifact<Schema, Feature> for Artifact {
    type Output = Output;
    type Locality = Locality;
    const IDENTITY: &'static str = "worth.query.tests.attribution-artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "worth.query.tests.attribution-producer.v1";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "worth.query.tests.attribution-reuse.v1";

    const STOPPED_OUTCOME: &'static str = "worth.query.tests.attribution-stopped.v1";
}
pub struct Input;
impl ApplicationComputationInput for Input {
    type Value = WorthQueryInvariantEntityIdentity<Schema, Account>;
    const IDENTITY: &'static str = "worth.query.tests.attribution-input.v1";
}
pub struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "worth.query.tests.attribution-evidence.v1";
}
pub struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "worth.query.tests.attribution-owner-stopped.v1";
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Number(pub u64);
impl ApplicationComputationPartition for Number {
    const IDENTITY: &'static str = "worth.query.tests.attribution-number.v1";
}
#[derive(Serialize)]
pub struct Parity(pub u64);
impl ApplicationComputationPartition for Parity {
    const IDENTITY: &'static str = "worth.query.tests.attribution-parity.v1";
}
pub struct Computation;
impl ApplicationManagedComputation<Schema, Feature> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = Parity;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "worth.query.tests.attribution-computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}
#[derive(Clone, Copy)]
pub enum StatusReadBy {
    Membership,
    ItemKeys,
}
pub struct Owner {
    pub status: StatusReadBy,
    pub gathered: Mutex<Vec<PartitionIdentity>>,
}
pub type Reader<'call, 'reader, 'runtime> =
    WorthQueryComputationReader<'call, 'reader, 'runtime, Schema, TouchAccountOperation>;
pub type Root = WorthQueryInvariantEntityIdentity<Schema, Account>;
impl WorthQueryPartitionedComputationOwner<Schema, Feature, Computation> for Owner {
    type Operation = TouchAccountOperation;
    type Item = Number;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;

    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
    ) -> Result<WorthQueryComputationPartitionPlan<Number>, WorthQueryComputationInputDenial<u32>>
    {
        if matches!(self.status, StatusReadBy::Membership) {
            reader.field(account, AccountStatus::reference())?;
        }
        Ok(WorthQueryComputationPartitionPlan::keyed(
            [3, 1, 2].map(Number),
            |item| PartitionItemId(item.0),
        ))
    }

    fn partition_key(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        item: &Number,
    ) -> Result<Parity, WorthQueryComputationInputDenial<u32>> {
        if matches!(self.status, StatusReadBy::ItemKeys) {
            reader.field(account, AccountStatus::reference())?;
        }
        Ok(Parity(item.0 % 2))
    }

    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        account: &Root,
        partition: WorthQueryComputationPartitionMembers<'_, Parity, Number>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        reader.field(account, AccountLabel::reference())?;
        self.gathered.lock().unwrap().push(partition.identity());
        Ok(partition.items().map(|(_, item)| item.0).sum())
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Parity, u64>,
        _: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        Ok(*partition.gathered())
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, |left, right| left + right)
    }

    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}
pub fn routed(
    status: StatusReadBy,
    runs: usize,
    handler_reads_status: bool,
) -> (Option<Vec<ComputationFactReaders>>, Vec<PartitionIdentity>) {
    let world = installed_authorization_world(true);
    let request = live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let custody = phase.test_host_request();
            let principal = authenticated_principal(&custody, &world, &request);
            let account = resolved_account(&custody, &world, "open", &request);
            let operation = world
                .application
                .installed_schema()
                .installed_operation(TouchAccountOperation::reference())
                .unwrap();
            let admission = world
                .selected_product(&custody)
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let installed = WorthQueryInstalledPartitionedComputation::<
                Schema,
                Feature,
                Computation,
                Owner,
            >::new(
                Owner {
                    status,
                    gathered: Mutex::default(),
                },
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::installed::ComputationRetention::Unretained,
            );
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| {
                        let execution = phase
                            .execution_for(&world.application.product_runtime)
                            .expect("the installed fixture lends its computation execution");
                        for _ in 0..runs {
                            if installed.prepare_through(reader, execution, root).is_err() {
                                panic!("the owner's reads are declared reads of the operation");
                            }
                        }
                        if handler_reads_status {
                            reader
                                .require_decision_field(root, AccountStatus::reference())
                                .unwrap();
                        }
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let read_set = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let mut gathered = installed.owner.gathered.lock().unwrap().clone();
            gathered.sort_unstable();
            gathered.dedup();
            (
                read_set.computation_facts().map(|facts| {
                    facts
                        .facts()
                        .map(|(_, _, readers)| readers.clone())
                        .collect()
                }),
                gathered,
            )
        })
        .expect("the attribution read owns its host call")
}
