//! Native graph values drive item edits, routing and membership without an invoice.
use super::*;
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

#[derive(Default)]
struct SourceOwner {
    gathered: Mutex<Vec<u64>>,
}
impl WorthQueryPartitionedComputationOwner<Schema, Feature, Computation> for SourceOwner {
    type Operation = TouchAccountOperation;
    type Item = Number;
    type Gathered = u64;
    type PartitionResult = u64;
    type Output = u64;
    type Stopped = u32;
    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_>,
        input: &Root,
    ) -> Result<WorthQueryComputationPartitionPlan<Number>, WorthQueryComputationInputDenial<u32>>
    {
        let source = reader
            .field(input, AccountLabel::reference())?
            .expect("seeded authored label");
        Ok(WorthQueryComputationPartitionPlan::keyed(
            source.chars().enumerate().map(|(index, character)| {
                Number((index as u64 + 1) * 1_000_000 + u64::from(character))
            }),
            |item| PartitionItemId(item.0 / 1_000_000),
        ))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Root,
        item: &Number,
    ) -> Result<Parity, WorthQueryComputationInputDenial<u32>> {
        Ok(Parity(item.0 % 1_000_000 % 2))
    }
    fn gather(
        &self,
        _: &mut Reader<'_, '_, '_>,
        _: &Root,
        partition: WorthQueryComputationPartitionMembers<'_, Parity, Number>,
    ) -> Result<u64, WorthQueryComputationInputDenial<u32>> {
        self.gathered.lock().unwrap().push(partition.key().0);
        Ok(partition
            .items()
            .map(|(id, item)| id.0 * (item.0 % 1_000_000))
            .sum())
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, Parity, u64>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<u64, WorthQueryManagedComputationDenial<u32>> {
        checkpoint.poll()?;
        Ok(*partition.gathered())
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<u64> {
        WorthQueryDeterministicReducer::canonical(|| 0, sum)
    }
    fn complete(&self, reduced: u64) -> Result<u64, u32> {
        Ok(reduced)
    }
}
impl TestOwner for SourceOwner {
    fn gathered(&self) -> &Mutex<Vec<u64>> {
        &self.gathered
    }
}

#[test]
fn native_value_key_and_membership_edits_carry_unaffected_partitions() {
    let _observations = QueryObservation::install_on_this_thread();
    let world = crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world_with_product_resources(
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources().with_execution_policy(policy()));
    let installed =
        WorthQueryInstalledPartitionedComputation::<Schema, Feature, Computation, _>::new(
            SourceOwner::default(),
            ComputationRetention::ProducerOperation,
        );
    let mut previous = native_attempt(
        &world,
        &installed,
        ComputationPrior::new(edition(), Err(Cause::FirstRun), None),
    );
    let subject = previous
        .sealed
        .state
        .facts
        .facts()
        .find_map(|(_, fact, _)| {
            if let WorthQueryApplicationObservedFact::Field { entity_id, .. } = fact {
                Some(*entity_id)
            } else {
                None
            }
        })
        .unwrap();
    // Initial label is "primary". Stable position IDs survive value/key edits.
    for (source, expected_gathers) in [
        ("trimary", vec![0]),    // value change, same key
        ("urimary", vec![0, 1]), // same item moves between keys
        ("urimaryz", vec![0]),   // new even member
        ("urimary", vec![0]),    // remove that member
        ("urimar", vec![1]),     // remove an odd member
    ] {
        let field = AccountLabel::reference();
        let locator = world
            .application
            .primary_provider
            .graph
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .unwrap()
            .clone();
        let request = live_scope();
        world.application.with_application_advancement(&request, |phase| {
            let active = phase.execution_for(&world.application.product_runtime).unwrap();
            let execution = active.execution_request().unwrap();
            crate::domain_computation::primary_graph::tests::fixture::publish_relational_mutation(
                &execution, &world, WorkerIntentBatch::new("native-membership-edit").push(
                    MutationIntent::Entity(EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: subject,
                        fields: AspectFieldPatch::from(BTreeMap::from([(locator, StringApplicationValueBinding::encode(&source.to_owned()).unwrap())])),
                    }))));
        }).unwrap();
        let edited = native_attempt(&world, &installed, retained_prior(previous.sealed));
        assert_eq!(
            edited.gathered, expected_gathers,
            "actual affected partitions for {source}"
        );
        let expected = source
            .chars()
            .enumerate()
            .map(|(index, character)| (index as u64 + 1) * u64::from(character))
            .sum::<u64>();
        assert_eq!(edited.value, expected, "independent authored-value result");
        let fresh = native_attempt(
            &world,
            &installed,
            ComputationPrior::new(edition(), Err(Cause::FirstRun), None),
        );
        assert_eq!(edited.value, fresh.value);
        let retained = edited
            .sealed
            .state
            .typed
            .downcast_ref::<super::super::super::retained::RetainedPartitions<Parity, Number, u64>>(
            )
            .unwrap();
        assert_eq!(
            retained.items.keys().map(|id| id.0).collect::<Vec<_>>(),
            (1..=source.len() as u64).collect::<Vec<_>>()
        );
        let warm = native_attempt(&world, &installed, retained_prior(edited.sealed));
        assert!(
            warm.gathered.is_empty(),
            "unchanged native state carries every partition"
        );
        assert_eq!(warm.value, expected);
        previous = warm;
    }
}
