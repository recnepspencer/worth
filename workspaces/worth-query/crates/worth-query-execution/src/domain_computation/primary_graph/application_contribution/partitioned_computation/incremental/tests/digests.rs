//! Real serialization entries distinguish unretained execution from reuse-off.
use super::super::super::attribution_tests::{Artifact, Reuse, Stopped};
use super::*;
use serde::{Serialize, Serializer};
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_execution::ChargedBytes;
use worth_query_declaration::facade::application_program::{
    ApplicationComputationExecution, ApplicationComputationInput, ApplicationComputationPartition,
    ApplicationComputationResourceCeiling,
};

static INPUTS: AtomicUsize = AtomicUsize::new(0);
static ITEMS: AtomicUsize = AtomicUsize::new(0);
struct Value(Root);
impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        INPUTS.fetch_add(1, Ordering::Relaxed);
        self.0.serialize(serializer)
    }
}
struct CountedInput;
impl ApplicationComputationInput for CountedInput {
    type Value = Value;
    const IDENTITY: &'static str = "courtroom-counted-input";
}
struct Item(u64);
impl Serialize for Item {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ITEMS.fetch_add(1, Ordering::Relaxed);
        serializer.serialize_u64(self.0)
    }
}
impl ApplicationComputationPartition for Item {
    const IDENTITY: &'static str = "courtroom-counted-item";
}
impl ChargedBytes for Item {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
struct Digests;
impl ApplicationManagedComputation<Schema, Feature> for Digests {
    type Input = CountedInput;
    type Output = Artifact;
    type Partition = Parity;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "courtroom-digest-computation";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(1 << 20, 4096);
}
struct DigestOwner;
impl WorthQueryPartitionedComputationOwner<Schema, Feature, Digests> for DigestOwner {
    type Operation = TouchAccountOperation;
    type Item = Item;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;
    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        input: &Value,
    ) -> Result<WorthQueryComputationPartitionPlan<Item>, WorthQueryComputationInputDenial<u32>>
    {
        reader.field(&input.0, AccountStatus::reference())?;
        Ok(WorthQueryComputationPartitionPlan::keyed(
            (0..4).map(Item),
            |item| PartitionItemId(item.0),
        ))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Value,
        item: &Item,
    ) -> Result<Parity, WorthQueryComputationInputDenial<u32>> {
        Ok(Parity(item.0 % 2))
    }
    fn gather(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Value,
        members: WorthQueryComputationPartitionMembers<'_, Parity, Item>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        Ok(members.items().map(|(_, item)| item.0).sum())
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Parity, u64>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        checkpoint.advance(1)?;
        Ok(*partition.gathered())
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, sum)
    }
    fn complete(&self, value: u64) -> Result<u64, u32> {
        Ok(value)
    }
}

#[test]
fn unretained_runs_serialize_neither_input_nor_items_for_digests() {
    let world = installed_authorization_world(true);
    for retention in [
        ComputationRetention::ProducerOperation,
        ComputationRetention::Unretained,
    ] {
        let installed = WorthQueryInstalledPartitionedComputation::<_, _, Digests, _>::new(
            DigestOwner,
            retention,
        );
        let request = live_scope();
        let principal = authenticated_principal(&world, &request);
        let root = resolved_account(&world, "open", &request);
        let operation = world
            .application
            .installed_schema()
            .installed_operation(TouchAccountOperation::reference())
            .unwrap();
        let admitted = world
            .selected_product()
            .authorize_operation(&principal, &root, &operation, Default::default(), &request)
            .unwrap();
        INPUTS.store(0, Ordering::Relaxed);
        ITEMS.store(0, Ordering::Relaxed);
        ComputationPrior::hand_in_test(Some(ComputationPrior::new(
            edition(),
            Err(Cause::FirstRun),
            None,
        )));
        let result = world.invariant.project_admitted_operation(&admitted, |reader, root| {
            let execution = QueryRequestExecution::open(RuntimeWorldExecutionPlacement::Serial(crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(std::num::NonZeroUsize::MIN, 1 << 30)), &request);
            installed.prepare_through(reader, &execution, &Value(root.clone()))?.compute(WorthQueryManagedComputationExecution::new(&execution))?.complete()
        }).unwrap().into_parts().0;
        ComputationPrior::hand_in_test(None);
        assert_eq!(result.unwrap(), (0..4).sum::<u64>());
        let counts = [
            INPUTS.load(Ordering::Relaxed),
            ITEMS.load(Ordering::Relaxed),
        ];
        match retention {
            ComputationRetention::Unretained => {
                assert_eq!(counts, [0, 0], "unretained omits both digest encodings")
            }
            ComputationRetention::ProducerOperation => assert!(
                counts.into_iter().all(|count| count > 0),
                "the same encoders are live in the retained control"
            ),
        }
    }
}
