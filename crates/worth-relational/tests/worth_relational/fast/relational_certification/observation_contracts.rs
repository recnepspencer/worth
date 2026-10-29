pub(crate) use crate::fast::relational_certification::world::supply_chain::*;

#[test]
fn production_snapshot_schema_ignores_program_descriptor_mutation() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court()).unwrap();
    let program = CompiledSupplyChainProgram::compile(definition).unwrap();
    let world = compile_supply_chain_baseline(execution, program).unwrap();
    let mut altered_program = world.program.clone();
    altered_program.definition_mut_for_test().schema =
        SupplyChainSchema::canonical(SchemaVersion::V2);

    let observed = observe_supply_chain_snapshot(
        execution,
        &altered_program,
        &world.handles,
        &world.runtime,
        &world.handles.snapshot,
    )
    .expect("the owner-selected production snapshot remains observable");

    assert_eq!(observed.schema, SchemaVersion::V1);
    assert_ne!(
        observed.schema,
        altered_program.definition().schema.version,
        "schema observation must not copy the caller's program descriptor"
    );
}
