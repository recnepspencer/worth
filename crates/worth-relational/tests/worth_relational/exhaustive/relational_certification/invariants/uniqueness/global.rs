#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::invariant_global_uniqueness::*;

#[test]
fn global_uniqueness_uses_selected_branch_proposal_after_sibling_diverges() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let program = CompiledSupplyChainProgram::compile(
        SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let catalog = InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(
                AspectKey::new("call_sign").expect("call-sign aspect"),
                FieldKey::new("call_sign").expect("call-sign field"),
            ),
        )],
    };
    let world = compile_supply_chain_baseline_with_invariant_catalog(execution, program, catalog)
        .expect("Court world with native uniqueness compiles");
    let main = BranchId("main".to_owned());
    let storm = BranchId("storm".to_owned());
    let (_, source) = world
        .runtime
        .observe_fork_source(&main)
        .expect("main remains forkable");
    world
        .runtime
        .fork_branch(execution, storm.clone(), source)
        .expect("storm fork shares the baseline root");

    let oracle_genesis = OracleBranch::genesis(OracleState::from_definition(
        &SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    ));
    let oracle_storm = oracle_genesis
        .fork(
            super::world::supply_chain::BranchLabel::Storm,
            super::world::supply_chain::BranchLabel::Operating,
        )
        .expect("oracle storm branch forks from operating");
    let oracle_main = insert_vessel(
        &oracle_genesis,
        next_vessel_key(&oracle_genesis),
        DIVERGENT_CALL_SIGN,
    )
    .expect("oracle accepts the main branch vessel");
    let oracle_storm = insert_vessel(
        &oracle_storm,
        next_vessel_key(&oracle_storm),
        DIVERGENT_CALL_SIGN,
    )
    .expect("oracle accepts the independent storm vessel");

    let main_commit = commit_vessel(execution, &world.runtime, main.clone(), "main-only-vessel");
    assert!(main_commit.is_ok(), "main-only call sign is admissible");
    assert_unique_execution(main_commit.as_ref().expect("main commit"));
    let main_values = vessel_call_signs(execution, &world.runtime, &main);
    let storm_before = vessel_call_signs(execution, &world.runtime, &storm);
    assert_eq!(main_values, oracle_vessel_call_signs(&oracle_main));
    assert_eq!(storm_before, oracle_vessel_call_signs(&oracle_genesis));

    // A global-current-state mutant would see main's value here and reject this
    // legal child commit. The semantic oracle is independent of production
    // snapshots, indexes, and record identifiers.
    let storm_commit = commit_vessel(
        execution,
        &world.runtime,
        storm.clone(),
        "storm-only-vessel",
    );
    assert!(
        storm_commit.is_ok(),
        "child uniqueness must use the child root plus its proposal: {storm_commit:?}"
    );
    assert_unique_execution(storm_commit.as_ref().expect("storm commit"));
    let storm_values = vessel_call_signs(execution, &world.runtime, &storm);
    assert_eq!(storm_values, oracle_vessel_call_signs(&oracle_storm));
    assert_eq!(main_values, oracle_vessel_call_signs(&oracle_main));
}
#[path = "global/semantic_oracle.rs"]
pub(crate) mod semantic_oracle;
