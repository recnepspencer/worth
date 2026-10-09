use super::production_failures::court_program;
use super::world::supply_chain::compile_supply_chain_baseline;
use super::world::supply_chain::{EntityKey, EntityKind, EntityRecord};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
use worth_relational::facade::transactions::{
    planned_single_field_locator, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

#[test]
fn pinned_snapshot_observation_does_not_select_a_later_head() {
    let world = compile_supply_chain_baseline(court_program()).expect("Court world compiles");
    let port_key = EntityKey::new(EntityKind::Port, 0);
    let port_id = world.handles.entities[&port_key].id;
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            AspectKey::new("name").expect("canonical field"),
            FieldKey::new("name").expect("canonical field"),
        ),
        AspectValue::String(InternedString::Raw("later-head".to_owned())),
    );
    let mut transaction = {
        let transaction_validation_input = {
            let identity = world.runtime.main_branch_identity();
            world
                .runtime
                .admit_branch_basis(&identity)
                .expect("configured main branch must remain owner-admissible")
        };
        world
            .runtime
            .begin_branch_transaction(
                &transaction_validation_input,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            WorkerIntentBatch::new("latest-head-twin").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: port_id,
                    fields: worth_relational::facade::transactions::AspectFieldPatch::new(fields),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .commit(
            &world.runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("later head update commits through the public facade");

    let observed = super::world::supply_chain::observe_supply_chain(&world)
        .expect("the original pinned snapshot remains readable");
    match observed.entities.get(&port_key) {
        Some(EntityRecord::Port(record)) => assert_eq!(record.name, "Meridian"),
        other => panic!("expected the pinned Port observation, got {other:?}"),
    }
}
