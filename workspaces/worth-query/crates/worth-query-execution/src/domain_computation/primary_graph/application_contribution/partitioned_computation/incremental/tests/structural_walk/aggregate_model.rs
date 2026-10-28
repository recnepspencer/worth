//! Real partitioned owner whose gather reads a persistent orbit.
use crate::domain_computation::primary_graph::application_contribution::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use crate::domain_computation::primary_graph::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_query_declaration::facade::{
    application_program::*,
    application_schema::{
        ApplicationStructuredValueBinding, ApplicationValueValidationDenial,
    },
};
pub(super) struct Feature;
impl ApplicationFeature<OrbitSchema<true>> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.aggregate-feature.v1";
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Value(pub u64);
worth_query_declaration::worth_query_portable_type!(Value=>"worth.query.test.aggregate-value.v1");
pub(super) struct ValueBinding;
impl ApplicationStructuredValueBinding for ValueBinding {
    type Value = Value;
    const IDENTITY_NAME: &'static str = "worth.query.test.aggregate-value.v1";
    fn validate(_: &Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
pub(super) struct Output;
impl ApplicationOutputPort<OrbitSchema<true>, Feature> for Output {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "worth.query.test.aggregate-output.v1";
}
pub(super) struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "worth.query.test.aggregate-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}
pub(super) struct Artifact;
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
pub(super) struct Input;
impl ApplicationComputationInput for Input {
    type Value = WorthQueryInvariantEntityIdentity<OrbitSchema<true>, Node>;
    const IDENTITY: &'static str = "worth.query.test.aggregate-input.v1";
}
pub(super) struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "worth.query.test.aggregate-reuse-evidence.v1";
}
pub(super) struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "worth.query.test.aggregate-refusal.v1";
}
#[derive(Clone, Debug, serde::Serialize)]
pub(super) struct Item(pub u64);
impl ApplicationComputationPartition for Item {
    const IDENTITY: &'static str = "worth.query.test.aggregate-item.v1";
}
#[derive(serde::Serialize)]
pub(super) struct Key;
impl ApplicationComputationPartition for Key {
    const IDENTITY: &'static str = "worth.query.test.aggregate-partition.v1";
}
pub(super) struct Computation;
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
pub(super) struct Owner {
    pub(super) gathers: AtomicUsize,
}
type Reader<'a, 'b, 'c> = WorthQueryComputationReader<'a, 'b, 'c, OrbitSchema<true>, ReplaceNext>;
type Root = WorthQueryInvariantEntityIdentity<OrbitSchema<true>, Node>;
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
pub(super) type Installed =
    WorthQueryInstalledPartitionedComputation<OrbitSchema<true>, Feature, Computation, Owner>;
