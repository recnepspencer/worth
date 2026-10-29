pub(crate) use {crate::relational_certification::ordinary_context::production_failures::court_program};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString};
pub(crate) use {worth_relational::facade::transactions::planned_single_field_locator, worth_relational::facade::transactions::EntityMutationIntent, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::UpdateEntityFieldsIntent, worth_relational::facade::transactions::WorkerIntentBatch};

#[test]
fn pinned_snapshot_observation_does_not_select_a_later_head() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world =
        compile_supply_chain_baseline(execution, court_program()).expect("Court world compiles");
    let port_key = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 0);
    let port_id = world.handles.entities[&port_key].id;
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new("name").expect("canonical field"),
            worth_foundational::facade::FieldKey::new("name").expect("canonical field"),
        ),
        worth_foundational::facade::AspectValue::String(worth_foundational::facade::InternedString::Raw("later-head".to_owned())),
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
            worth_relational::facade::transactions::WorkerIntentBatch::new("latest-head-twin").push(worth_relational::facade::transactions::MutationIntent::Entity(
                worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: port_id,
                    fields: worth_relational::facade::transactions::AspectFieldPatch::new(fields),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .commit(execution, &world.runtime)
        .expect("later head update commits through the public facade");

    let observed = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe(execution, &world)
        .expect("the original pinned snapshot remains readable");
    match observed.entities.get(&port_key) {
        Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Port(record)) => assert_eq!(record.name, "Meridian"),
        other => panic!("expected the pinned Port observation, got {other:?}"),
    }
}
