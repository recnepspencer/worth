//! Real partitioned owner whose gather reads a persistent orbit.
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
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantEntityIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_program;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::root;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use worth_query_declaration::facade::application_program::*;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_declaration::facade::application_schema::ApplicationValueValidationDenial;
pub struct Feature;
impl ApplicationFeature<OrbitSchema<true>> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.aggregate-feature.v1";
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value(pub u64);
worth_query_declaration::worth_query_portable_type!(Value=>"worth.query.test.aggregate-value.v1");
pub struct ValueBinding;
impl ApplicationStructuredValueBinding for ValueBinding {
    type Value = Value;
    const IDENTITY_NAME: &'static str = "worth.query.test.aggregate-value.v1";
    fn validate(_: &Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
pub struct Output;
impl ApplicationOutputPort<OrbitSchema<true>, Feature> for Output {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "worth.query.test.aggregate-output.v1";
}
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "worth.query.test.aggregate-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}
pub struct Artifact;
impl ApplicationDerivedArtifact<OrbitSchema<true>, Feature> for Artifact {
    type Output = Output;
    type Locality = Locality;
    const IDENTITY: &'static str = "worth.query.test.aggregate-artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "worth.query.test.aggregate-producer.v1";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "worth.query.test.aggregate-reuse.v1";
    const STOPPED_OUTCOME: &'static str = "worth.query.test.aggregate-stopped.v1";
}
pub struct Input;
impl ApplicationComputationInput for Input {
    type Value = WorthQueryInvariantEntityIdentity<OrbitSchema<true>, Node>;
    const IDENTITY: &'static str = "worth.query.test.aggregate-input.v1";
}
pub struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "worth.query.test.aggregate-reuse-evidence.v1";
}
pub struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "worth.query.test.aggregate-refusal.v1";
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Item(pub u64);
impl ApplicationComputationPartition for Item {
    const IDENTITY: &'static str = "worth.query.test.aggregate-item.v1";
}
#[derive(serde::Serialize)]
pub struct Key;
impl ApplicationComputationPartition for Key {
    const IDENTITY: &'static str = "worth.query.test.aggregate-partition.v1";
}
pub struct Computation;
impl ApplicationManagedComputation<OrbitSchema<true>, Feature> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = Key;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "worth.query.test.aggregate-computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}
#[derive(Default)]
pub struct Owner {
    pub gathers: AtomicUsize,
}
pub type Reader<'a, 'b, 'c> =
    WorthQueryComputationReader<'a, 'b, 'c, OrbitSchema<true>, ReplaceNext>;
pub type Root = WorthQueryInvariantEntityIdentity<OrbitSchema<true>, Node>;
impl WorthQueryPartitionedComputationOwner<OrbitSchema<true>, Feature, Computation> for Owner {
    type Operation = ReplaceNext;
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
            [Item(0)],
            |item| worth_execution::PartitionItemId(item.0),
        ))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Root,
        _: &Item,
    ) -> Result<Key, WorthQueryComputationInputDenial<u32>> {
        Ok(Key)
    }
    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        root: &Root,
        _: WorthQueryComputationPartitionMembers<'_, Key, Item>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        self.gathers.fetch_add(1, Ordering::Relaxed);
        Ok(reader.orbit(Next::reference(), root)?.len() as u64)
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
    WorthQueryInstalledPartitionedComputation<OrbitSchema<true>, Feature, Computation, Owner>;
