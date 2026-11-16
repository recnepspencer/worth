//! Two independently rooted partitions over the installed walk schema.
use worth_query_execution::facade::application_contribution::WorthQueryComputationInputDenial;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionMembers;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionPlan;
use worth_query_execution::facade::application_contribution::WorthQueryComputationPartitionView;
use worth_query_execution::facade::application_contribution::WorthQueryComputationReader;
use worth_query_execution::facade::application_contribution::WorthQueryDeterministicReducer;
use worth_query_execution::facade::application_contribution::WorthQueryInstalledPartitionedComputation;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationCheckpoint;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationDenial;
use worth_query_execution::facade::application_contribution::WorthQueryPartitionedComputationOwner;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::*;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantEntityIdentity;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use worth_query_declaration::facade::application_program::*;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_declaration::facade::application_schema::ApplicationValueValidationDenial;
use worth_query_declaration::facade::application_schema::RelationDirection;
pub struct Feature;
impl ApplicationFeature<WalkSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.structural-chains-feature.v1";
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value(pub u64);
worth_query_declaration::worth_query_portable_type!(Value=>"worth.query.test.structural-chains-value.v1");
pub struct ValueBinding;
impl ApplicationStructuredValueBinding for ValueBinding {
    type Value = Value;
    const IDENTITY_NAME: &'static str = "worth.query.test.structural-chains-value.v1";
    fn validate(_: &Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
pub struct Output;
impl ApplicationOutputPort<WalkSchema, Feature> for Output {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "worth.query.test.structural-chains-output.v1";
}
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "worth.query.test.structural-chains-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}
pub struct Artifact;
impl ApplicationDerivedArtifact<WalkSchema, Feature> for Artifact {
    type Output = Output;
    type Locality = Locality;
    const IDENTITY: &'static str = "worth.query.test.structural-chains-artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "worth.query.test.structural-chains-producer.v1";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "worth.query.test.structural-chains-reuse.v1";
    const STOPPED_OUTCOME: &'static str = "worth.query.test.structural-chains-stopped.v1";
}
pub struct Input;
impl ApplicationComputationInput for Input {
    type Value = Vec<WorthQueryInvariantEntityIdentity<WalkSchema, Node>>;
    const IDENTITY: &'static str = "worth.query.test.structural-chains-input.v1";
}
pub struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "worth.query.test.structural-chains-reuse-evidence.v1";
}
pub struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "worth.query.test.structural-chains-refusal.v1";
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Item(pub u64);
impl ApplicationComputationPartition for Item {
    const IDENTITY: &'static str = "worth.query.test.structural-chains-item.v1";
}
#[derive(serde::Serialize)]
pub struct Key(pub u64);
impl ApplicationComputationPartition for Key {
    const IDENTITY: &'static str = "worth.query.test.structural-chains-partition.v1";
}
pub struct Computation;
impl ApplicationManagedComputation<WalkSchema, Feature> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = Key;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "worth.query.test.structural-chains-computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}
#[derive(Default)]
pub struct Owner {
    pub gathers: AtomicUsize,
    pub gathered: std::sync::Mutex<Vec<u64>>,
}
pub type Reader<'a, 'b, 'c> = WorthQueryComputationReader<'a, 'b, 'c, WalkSchema, InspectWalk>;
pub type Root = Vec<WorthQueryInvariantEntityIdentity<WalkSchema, Node>>;
impl WorthQueryPartitionedComputationOwner<WalkSchema, Feature, Computation> for Owner {
    type Operation = InspectWalk;
    type Item = Item;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;
    fn partitions(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Root,
    ) -> Result<WorthQueryComputationPartitionPlan<Item>, WorthQueryComputationInputDenial<u32>>
    {
        Ok(WorthQueryComputationPartitionPlan::keyed(
            [Item(0), Item(1)],
            |item| worth_execution::PartitionItemId(item.0),
        ))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Root,
        item: &Item,
    ) -> Result<Key, WorthQueryComputationInputDenial<u32>> {
        Ok(Key(item.0))
    }
    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        root: &Root,
        members: WorthQueryComputationPartitionMembers<'_, Key, Item>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        self.gathers.fetch_add(1, Ordering::Relaxed);
        self.gathered.lock().unwrap().push(members.key().0);
        Ok(reader
            .walk(
                Next::reference(),
                RelationDirection::Forward,
                &root[members.key().0 as usize],
            )?
            .len() as u64)
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Key, u64>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        checkpoint.poll()?;
        Ok(*partition.gathered())
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, |a, b| a + b)
    }
    fn complete(&self, result: u64) -> Result<u64, u32> {
        Ok(result)
    }
}
pub type Installed =
    WorthQueryInstalledPartitionedComputation<WalkSchema, Feature, Computation, Owner>;
