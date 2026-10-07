//! Actual pure partition kernels; only gathering may read committed upstream facts.
use super::{application::*, expected_history::Partition, operation::Apply};
use worth_query_decl::facade::{
    application_operation::application_computation_partition_identity, application_program::*,
};
use worth_query_host::facade::application_contribution::*;
use worth_query_host::facade::primary_graph::WorthQueryInvariantEntityIdentity;
#[derive(serde::Serialize)]
pub struct ReadInput {
    pub upstream: Vec<WorthQueryInvariantEntityIdentity<Schema, Node>>,
    pub partitions: Vec<Partition>,
}
impl ChargedBytes for Partition {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl ApplicationComputationPartition for Partition {
    const IDENTITY: &'static str = "neutral-item";
}
pub struct InputType;
impl ApplicationComputationInput for InputType {
    type Value = ReadInput;
    const IDENTITY: &'static str = "neutral-input";
}
#[derive(Clone, serde::Serialize)]
pub struct Key(u64);
impl ApplicationComputationPartition for Key {
    const IDENTITY: &'static str = "neutral-partition";
}
/// Semantic keys are ranks in the declared canonical identity order, not kernel completion order.
pub(super) fn identities(partitions: &[Partition]) -> Vec<u64> {
    let mut identities = partitions
        .iter()
        .map(|part| {
            application_computation_partition_identity(&key(part.key), &mut |_| Ok::<_, ()>(()))
                .unwrap()
                .partition()
                .value()
        })
        .collect::<Vec<_>>();
    identities.sort();
    identities
}
fn key(rank: u64) -> Key {
    let mut keys = (0..4).map(Key).collect::<Vec<_>>();
    keys.sort_by_key(|key| {
        application_computation_partition_identity(key, &mut |_| Ok::<_, ()>(()))
            .unwrap()
            .partition()
    });
    keys.remove(usize::try_from(rank).unwrap())
}
pub struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "neutral-reuse";
}
pub struct Stop;
impl ApplicationComputationStopped for Stop {
    const IDENTITY: &'static str = "neutral-stop";
}
pub struct Computation;
impl ApplicationManagedComputation<Schema, Feature> for Computation {
    type Input = InputType;
    type Output = Artifact;
    type Partition = Key;
    type Reuse = Reuse;
    type Stopped = Stop;
    const IDENTITY: &'static str = "neutral-computation";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(8192, 16384);
}
pub struct Owner;
type Reader<'a, 'b, 'c> = WorthQueryComputationReader<'a, 'b, 'c, Schema, Apply>;
impl WorthQueryPartitionedComputationOwner<Schema, Feature, Computation> for Owner {
    type Operation = Apply;
    type Item = Partition;
    type Gathered = Vec<u64>;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u64;
    fn partitions(
        &self,
        _: &mut Reader<'_, '_, '_>,
        input: &ReadInput,
    ) -> Result<WorthQueryComputationPartitionPlan<Partition>, WorthQueryComputationInputDenial<u64>>
    {
        Ok(WorthQueryComputationPartitionPlan::keyed(
            input.partitions.clone(),
            |part| PartitionItemId(part.key),
        ))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &ReadInput,
        item: &Partition,
    ) -> Result<Key, WorthQueryComputationInputDenial<u64>> {
        Ok(key(item.key))
    }
    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        input: &ReadInput,
        partition: WorthQueryComputationPartitionMembers<'_, Key, Partition>,
    ) -> Result<Vec<u64>, WorthQueryComputationInputDenial<u64>> {
        let part = partition.items().next().unwrap().1;
        let mut value = part.value;
        // Add the declared upstream values once, in the first semantic partition.
        if part.key == 0 {
            for upstream in &input.upstream {
                value += reader.field(upstream, Value::reference())?.unwrap();
            }
        }
        Ok(vec![part.key, value, part.work, u64::from(part.fails)])
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Key, Vec<u64>>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u64>> {
        let facts = partition.gathered();
        checkpoint.advance(usize::try_from(facts[2]).unwrap())?;
        if facts[3] != 0 {
            return Err(WorthQueryManagedComputationDenial::Owner(facts[0]));
        }
        Ok(facts[1])
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, |left, right| left + right)
    }
    fn complete(&self, value: u64) -> Result<u64, u64> {
        Ok(value)
    }
}

/// Key derivation is a declared public encoding operation, outside the driver.
pub(super) fn declared_key_work(partitions: &[Partition]) -> u64 {
    use worth_query_decl::facade::application_operation::CanonicalEncodingCharge;
    let mut units = 0;
    for part in partitions {
        application_computation_partition_identity(&key(part.key), &mut |charge| {
            if let CanonicalEncodingCharge::Work(work) = charge {
                units += work;
            }
            Ok::<_, ()>(())
        })
        .unwrap();
    }
    units
}
