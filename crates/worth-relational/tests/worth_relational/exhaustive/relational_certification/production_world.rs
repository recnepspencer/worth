#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::production_world::*;

#[test]
fn supply_chain_named_handles_are_owner_issued_and_complete() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
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
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
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
    assert_eq!(budget.max_patch_records_per_commit, Some(16_384));
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
        .fork_branch(execution, BranchId("standard-storm".to_owned()), basis)
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
