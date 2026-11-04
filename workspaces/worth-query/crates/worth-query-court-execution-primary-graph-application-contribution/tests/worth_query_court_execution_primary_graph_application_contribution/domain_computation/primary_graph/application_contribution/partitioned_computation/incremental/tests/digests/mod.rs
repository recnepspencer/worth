//! Real serialization entries distinguish unretained execution from reuse-off.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Artifact;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Reuse;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::attribution_tests::Stopped;
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;
use serde::Serialize;
use serde::Serializer;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use worth_query_declaration::facade::application_program::ApplicationComputationExecution;
use worth_query_declaration::facade::application_program::ApplicationComputationInput;
use worth_query_declaration::facade::application_program::ApplicationComputationPartition;

pub static INPUTS: AtomicUsize = AtomicUsize::new(0);
pub static ITEMS: AtomicUsize = AtomicUsize::new(0);
pub struct Value(Root);
impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        INPUTS.fetch_add(1, Ordering::Relaxed);
        self.0.serialize(serializer)
    }
}
pub struct CountedInput;
impl ApplicationComputationInput for CountedInput {
    type Value = Value;
    const IDENTITY: &'static str = "courtroom-counted-input";
}
pub struct Item(u64);
impl Serialize for Item {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ITEMS.fetch_add(1, Ordering::Relaxed);
        serializer.serialize_u64(self.0)
    }
}
impl ApplicationComputationPartition for Item {
    const IDENTITY: &'static str = "courtroom-counted-item";
}
pub struct Digests;
impl ApplicationManagedComputation<Schema, Feature> for Digests {
    type Input = CountedInput;
    type Output = Artifact;
    type Partition = Parity;
    type Reuse = Reuse;
    type Stopped = Stopped;
    const IDENTITY: &'static str = "courtroom-digest-computation";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
}
pub struct DigestOwner;
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
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
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
        world
            .application
            .with_application_advancement(&request, |phase| {
                let custody = phase.test_host_request_for_test();
                let principal = authenticated_principal(&custody, &world, &request);
                let root = resolved_account(&custody, &world, "open", &request);
                let operation = world
                    .application
                    .installed_schema()
                    .installed_operation(TouchAccountOperation::reference())
                    .unwrap();
                let admitted = world
                    .selected_product(&custody)
                    .authorize_operation(
                        &phase,
                        &principal,
                        &root,
                        &operation,
                        Default::default(),
                        &request,
                    )
                    .unwrap();
                INPUTS.store(0, Ordering::Relaxed);
                ITEMS.store(0, Ordering::Relaxed);
                ComputationPrior::hand_in_test_for_test(Some(ComputationPrior::new_for_test(
                    edition(),
                    Err(Cause::FirstRun),
                    None,
                )));
                let result = world
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admitted,
                        |reader, root| {
                            let execution = phase
                                .execution_for_for_test(&(*world.application.product_runtime_for_test()))
                                .expect("the installed fixture lends its computation execution");
                            installed
                                .prepare_through_for_test(reader, execution, &Value(root.clone()))?
                                .compute(WorthQueryManagedComputationExecution::new(execution))?
                                .complete()
                        },
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .into_parts()
                    .0;
                ComputationPrior::hand_in_test_for_test(None);
                assert_eq!(result.unwrap(), (0..4).sum::<u64>());
            })
            .expect("the digest computation owns its host call");
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
