//! The input declares scalar frames; their checkpoints do real managed work.
use super::{
    application::*,
    control::{Control, Phase},
    operation::Apply,
};
use std::sync::Arc;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_contribution::*;
#[derive(serde::Serialize)]
pub struct ReadInput {
    pub node: u64,
    pub upstream_stamp: u64,
    pub parts: Vec<super::super::expected_history::Partition>,
}
pub struct InputType;
impl ApplicationComputationInput for InputType {
    type Value = ReadInput;
    const IDENTITY: &'static str = "matrix-input";
}
#[derive(Clone, serde::Serialize)]
pub struct Key(u64);
impl ApplicationComputationPartition for Key {
    const IDENTITY: &'static str = "matrix-partition";
}
#[derive(Clone, serde::Serialize)]
pub struct Item {
    node: u64,
    part: super::super::expected_history::Partition,
}
impl ChargedBytes for Item {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl ApplicationComputationPartition for Item {
    const IDENTITY: &'static str = "matrix-item";
}
pub struct Reuse;
impl ApplicationComputationReuse for Reuse {
    const IDENTITY: &'static str = "matrix-reuse";
}
pub struct Stop;
impl ApplicationComputationStopped for Stop {
    const IDENTITY: &'static str = "matrix-stop";
}
pub struct Computation;
impl ApplicationManagedComputation<Schema, Feature> for Computation {
    type Input = InputType;
    type Output = Artifact;
    type Partition = Key;
    type Reuse = Reuse;
    type Stopped = Stop;
    const IDENTITY: &'static str = "matrix-computation";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(8192, 16384);
}
pub struct Owner(pub Arc<Control>);
type Reader<'a, 'b, 'c> = WorthQueryComputationReader<'a, 'b, 'c, Schema, Apply>;
impl WorthQueryPartitionedComputationOwner<Schema, Feature, Computation> for Owner {
    type Operation = Apply;
    type Item = Item;
    type Gathered = Item;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u64;
    fn partitions(
        &self,
        _: &mut Reader<'_, '_, '_>,
        input: &ReadInput,
    ) -> Result<WorthQueryComputationPartitionPlan<Item>, WorthQueryComputationInputDenial<u64>>
    {
        let mut items = input
            .parts
            .iter()
            .map(|part| Item {
                node: input.node,
                part: part.clone(),
            })
            .collect::<Vec<_>>();
        if self.0.declaration == 1 {
            items.reverse();
        }
        self.0
            .declared_partitions(input.node, items.iter().map(|item| item.part.key).collect());
        Ok(WorthQueryComputationPartitionPlan::keyed(items, |item| {
            PartitionItemId(item.part.key)
        }))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &ReadInput,
        item: &Item,
    ) -> Result<Key, WorthQueryComputationInputDenial<u64>> {
        Ok(Key(item.part.key))
    }
    fn gather(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &ReadInput,
        members: WorthQueryComputationPartitionMembers<'_, Key, Item>,
    ) -> Result<Item, WorthQueryComputationInputDenial<u64>> {
        Ok(members.items().next().unwrap().1.clone())
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Key, Item>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u64>> {
        let item = partition.gathered();
        let work = self
            .0
            .enter_compute(item.node, item.part.key, item.part.work);
        let _flight = super::control::KernelFlight(&self.0);
        let result = checkpoint
            .advance(usize::try_from(work).unwrap())
            .map_err(WorthQueryManagedComputationDenial::from);
        result?;
        if item.part.key == 0 && self.0.fails(item.node, Phase::Compute) {
            Err(WorthQueryManagedComputationDenial::Owner(item.node))
        } else {
            Ok(item.part.value)
        }
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, |a, b| a + b)
    }
    fn complete(&self, value: u64) -> Result<u64, u64> {
        Ok(value)
    }
}
