pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot};

#[test]
fn production_snapshot_schema_ignores_program_descriptor_mutation() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court()).unwrap();
    let program = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(definition).unwrap();
    let world = compile_supply_chain_baseline(execution, program).unwrap();
    let mut altered_program = world.program.clone();
    altered_program.definition_mut_for_test().schema =
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SupplyChainSchema::canonical(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2);

    let observed = observe_supply_chain_snapshot(
        execution,
        &altered_program,
        &world.handles,
        &world.runtime,
        &world.handles.snapshot,
    )
    .expect("the owner-selected production snapshot remains observable");

    assert_eq!(observed.schema, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V1);
    assert_ne!(
        observed.schema,
        altered_program.definition().schema.version,
        "schema observation must not copy the caller's program descriptor"
    );
}
