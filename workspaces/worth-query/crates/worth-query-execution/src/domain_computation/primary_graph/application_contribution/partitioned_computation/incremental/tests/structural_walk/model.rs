//! Minimal real partitioned owner over the centrally installed walk schema.
use crate::domain_computation::primary_graph::application_contribution::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::*;
use crate::domain_computation::primary_graph::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_query_declaration::facade::{
    application_program::*,
    application_schema::{
        ApplicationStructuredValueBinding, ApplicationValueValidationDenial, RelationDirection,
    },
};
pub(super) struct Feature;
impl ApplicationFeature<WalkSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.structural-feature.v1";
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Value(pub u64);
worth_query_declaration::worth_query_portable_type!(Value=>"worth.query.test.structural-value.v1");
pub(super) struct ValueBinding;
impl ApplicationStructuredValueBinding for ValueBinding {
    type Value = Value;
    const IDENTITY_NAME: &'static str = "worth.query.test.structural-value.v1";
    fn validate(_: &Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
pub(super) struct Output;
impl ApplicationOutputPort<WalkSchema, Feature> for Output {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "worth.query.test.structural-output.v1";
}
pub(super) struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "worth.query.test.structural-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Partition;
}
pub(super) struct Artifact;
impl ApplicationDerivedArtifact<WalkSchema, Feature> for Artifact {
    type Output = Output;
    type Locality = Locality;
    const IDENTITY: &'static str = "worth.query.test.structural-artifact.v1";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "worth.query.test.structural-producer.v1";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "worth.query.test.structural-reuse.v1";
    const STOPPED_OUTCOME: &'static str = "worth.query.test.structural-stopped.v1";
}
pub(super) struct Input;
impl ApplicationComputationInput for Input {
    type Value = WorthQueryInvariantEntityIdentity<WalkSchema, Node>;
    const IDENTITY: &'static str = "worth.query.test.structural-input.v1";
}
pub(super) struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "worth.query.test.structural-reuse-evidence.v1";
}
pub(super) struct Stopped;
impl ApplicationComputationStopped for Stopped {
    const IDENTITY: &'static str = "worth.query.test.structural-refusal.v1";
}
#[derive(Clone, Debug, serde::Serialize)]
pub(super) struct Item(pub u64);
impl ApplicationComputationPartition for Item {
    const IDENTITY: &'static str = "worth.query.test.structural-item.v1";
}
#[derive(serde::Serialize)]
pub(super) struct Key;
impl ApplicationComputationPartition for Key {
    const IDENTITY: &'static str = "worth.query.test.structural-partition.v1";
}
pub(super) struct Computation;
impl ApplicationManagedComputation<WalkSchema, Feature> for Computation {
    type Input = Input;
    type Output = Artifact;
    type Partition = Key;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "worth.query.test.structural-computation.v1";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}
#[derive(Default)]
pub(super) struct Owner {
    pub(super) gathers: AtomicUsize,
}
type Reader<'a, 'b, 'c> = WorthQueryComputationReader<'a, 'b, 'c, WalkSchema, InspectWalk>;
type Root = WorthQueryInvariantEntityIdentity<WalkSchema, Node>;
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
        Ok(reader
            .walk(Next::reference(), RelationDirection::Forward, root)?
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
pub(super) type Installed =
    WorthQueryInstalledPartitionedComputation<WalkSchema, Feature, Computation, Owner>;
