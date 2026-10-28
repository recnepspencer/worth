pub(crate) use super::world::supply_chain::{
    audit_supply_chain_baseline, compile_supply_chain_baseline, CompiledSupplyChainProgram,
    SupplyChainProgramError, SupplyChainScale, SupplyChainWorldDefinition,
};
pub(crate) use worth_relational::facade::history::BranchId;

#[test]
fn supply_chain_world_compiles_causally_through_public_facades() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let program =
        CompiledSupplyChainProgram::compile(SupplyChainWorldDefinition::empty(empty_scale()))
            .expect("empty Supply Chain program compiles");
    let world =
        compile_supply_chain_baseline(execution, program).expect("empty production world compiles");
    assert_eq!(world.handles.entities.len(), 0);
    assert_eq!(world.handles.relations.len(), 0);
    assert_eq!(world.schema_receipt.retained_entity_kind_count(), 8);
    assert_eq!(world.schema_receipt.retained_relation_kind_count(), 10);
    assert!(world
        .runtime
        .read_truth()
        .read_snapshot(&world.handles.snapshot, execution)
        .expect("fixture read request remains live")
        .is_some());
    let certified = audit_supply_chain_baseline(execution, world)
        .expect("empty production baseline is observable");
    assert!(certified.observed.entities.is_empty());
    assert!(certified.expected.entities.is_empty());
    assert_eq!(certified.world.commit, certified.world.commit_result.commit);
    assert_eq!(
        certified
            .world
            .commit_result
            .patch_budget_summary()
            .expect("empty baseline records its patch budget")
            .patch_record_count,
        0
    );

    let court_program = CompiledSupplyChainProgram::compile(
        SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Court Supply Chain program compiles");
    let court_world = compile_supply_chain_baseline(execution, court_program)
        .expect("Court production world compiles through public facades");
    assert_eq!(court_world.schema_receipt.retained_entity_kind_count(), 8);
    assert_eq!(
        court_world.schema_receipt.retained_relation_kind_count(),
        10
    );
    assert_eq!(court_world.commit, court_world.commit_result.commit);
}

pub(crate) fn empty_scale() -> SupplyChainScale {
    let mut scale = SupplyChainScale::court();
    scale.ports = 0;
    scale.terminals = 0;
    scale.berths = 0;
    scale.vessels = 0;
    scale.voyages = 0;
    scale.port_calls = 0;
    scale.cargo_lots = 0;
    scale.regions = 1;
    scale
}

#[test]
fn production_compiler_rejects_a_scaled_empty_definition_before_runtime_admission() {
    let error = CompiledSupplyChainProgram::compile(SupplyChainWorldDefinition::empty(
        SupplyChainScale::court(),
    ))
    .expect_err("a Court-sized empty definition is not an operating baseline");
    assert!(matches!(error, SupplyChainProgramError::Definition(_)));
}
