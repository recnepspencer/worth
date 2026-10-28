#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::root_delta_lowering_contracts::*;

#[test]
fn phase5_lowering_reads_actual_branch_pre_state_instead_of_expected_oracle() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(execution, &world.runtime, "storm");
    let voyage = EntityKey::new(EntityKind::Voyage, 0);
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("storm".to_owned()),
        update_number(&world.handles, voyage, "arrival", 9_000),
    );

    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId("storm".to_owned()),
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .expect("actual pre-state satisfies the Storm contract");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("storm".to_owned()),
        batch,
    );

    let observed = observe_branch(execution, &mut world, "storm");
    let EntityRecord::Voyage(voyage) = &observed.entities[&voyage] else {
        panic!("Aurora voyage keeps its owner-observed kind");
    };
    assert_eq!(voyage.arrival.0, 9_030);
    let EntityRecord::Voyage(oracle_voyage) =
        &baseline.entities[&EntityKey::new(EntityKind::Voyage, 0)]
    else {
        panic!("oracle baseline declares Aurora voyage");
    };
    assert_ne!(
        voyage.arrival.0,
        oracle_voyage.arrival.0 + 30,
        "the oracle baseline cannot feed production intent lowering"
    );
}
