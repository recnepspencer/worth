use super::world::supply_chain::{
    audit_supply_chain_baseline, compile_supply_chain_baseline, CompiledSupplyChainProgram,
    SupplyChainProgramError, SupplyChainScale, SupplyChainWorldDefinition,
};
use worth_relational::facade::history::BranchId;

#[test]
fn supply_chain_world_compiles_causally_through_public_facades() {
    let host_request = crate::test_execution_policy::serial_request();
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

fn empty_scale() -> SupplyChainScale {
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
fn supply_chain_named_handles_are_owner_issued_and_complete() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court())
        .expect("Court Supply Chain definition is valid");
    let program = CompiledSupplyChainProgram::compile(definition)
        .expect("Court Supply Chain program compiles");
    let world =
        compile_supply_chain_baseline(execution, program).expect("Court production world compiles");
    assert_eq!(world.handles.entities.len(), 244);
    assert_eq!(world.handles.relations.len(), 247);
    assert_eq!(world.commit, world.commit_result.commit);
    assert!(world
        .program
        .definition()
        .entities
        .keys()
        .all(|key| world.handles.entities.contains_key(key)));
    assert!(world
        .program
        .definition()
        .relations
        .keys()
        .all(|key| world.handles.relations.contains_key(key)));
    assert_eq!(
        world
            .handles
            .entities
            .values()
            .map(|handle| handle.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        world.handles.entities.len()
    );
    assert_eq!(
        world
            .handles
            .relations
            .values()
            .map(|handle| handle.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        world.handles.relations.len()
    );
    let certified = audit_supply_chain_baseline(execution, world)
        .expect("Court baseline matches independent oracle");
    assert_eq!(
        certified.observed.entities.len(),
        certified.expected.entities.len()
    );
    assert_eq!(
        certified.observed.relations.len(),
        certified.expected.relations.len()
    );
    assert_eq!(certified.world.commit, certified.world.commit_result.commit);
}

#[test]
fn standard_supply_chain_baseline_matches_oracle_and_forks_without_copying() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::standard())
        .expect("Standard Supply Chain definition is valid");
    let program = CompiledSupplyChainProgram::compile(definition)
        .expect("Standard Supply Chain program compiles");
    let world = compile_supply_chain_baseline(execution, program)
        .expect("Standard production world compiles");
    assert_eq!(world.handles.entities.len(), 4_848);
    assert_eq!(world.handles.relations.len(), 3_363);
    let budget = world
        .commit_result
        .patch_budget_summary()
        .expect("Standard baseline records its patch budget");
    assert_eq!(budget.patch_record_count, 8_211);
    assert_eq!(budget.max_patch_records_per_commit, 16_384);
    let certified = audit_supply_chain_baseline(execution, world)
        .expect("Standard baseline matches independent oracle");
    assert_eq!(
        certified.observed.entities.len(),
        certified.expected.entities.len()
    );
    assert_eq!(
        certified.observed.relations.len(),
        certified.expected.relations.len()
    );
    let world = certified.world;
    let catalog_before = world.runtime.history().immutable_commit_count();
    let (_, basis) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("standard main branch has a fork source");
    let outcome = world
        .runtime
        .fork_branch(BranchId("standard-storm".to_owned()), basis)
        .expect("standard fork succeeds");
    assert_eq!(outcome.shared_commit_id(), Some(world.commit.commit_id));
    assert_eq!(
        world.runtime.history().immutable_commit_count(),
        catalog_before
    );
    assert_eq!(
        outcome.source_observation().target(),
        outcome.target_observation().target()
    );
    assert_eq!(
        outcome
            .source_observation()
            .target()
            .as_basis()
            .expect("standard source target")
            .roots(),
        outcome
            .target_observation()
            .target()
            .as_basis()
            .expect("standard fork target")
            .roots()
    );
}

#[test]
fn production_compiler_rejects_a_scaled_empty_definition_before_runtime_admission() {
    let error = CompiledSupplyChainProgram::compile(SupplyChainWorldDefinition::empty(
        SupplyChainScale::court(),
    ))
    .expect_err("a Court-sized empty definition is not an operating baseline");
    assert!(matches!(error, SupplyChainProgramError::Definition(_)));
}
