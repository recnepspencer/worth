#[allow(unused_imports)]
pub(crate) use {std::collections::BTreeSet, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus, worth_relational::facade::history::BranchId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, crate::relational_certification::ordinary_context::root_delta_lowering_contracts::NumberPrestateCase, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, crate::relational_certification::ordinary_context::root_delta_lowering_contracts::fork, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, crate::relational_certification::ordinary_context::root_delta_lowering_contracts::observe_branch, crate::relational_certification::ordinary_context::root_delta_lowering_contracts::prove_number_prestate, crate::relational_certification::ordinary_context::root_delta_lowering_contracts::update_number, crate::relational_certification::ordinary_context::root_delta_lowering_contracts::update_text};

#[test]
fn phase5_lowering_reads_actual_branch_pre_state_instead_of_expected_oracle() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(execution, &world.runtime, "storm");
    let voyage = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0);
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
        &std::collections::BTreeSet::new(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    )
    .expect("actual pre-state satisfies the Storm contract");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("storm".to_owned()),
        batch,
    );

    let observed = observe_branch(execution, &mut world, "storm");
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(voyage) = &observed.entities[&voyage] else {
        panic!("Aurora voyage keeps its owner-observed kind");
    };
    assert_eq!(voyage.arrival.0, 9_030);
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(oracle_voyage) =
        &baseline.entities[&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0)]
    else {
        panic!("oracle baseline declares Aurora voyage");
    };
    assert_ne!(
        voyage.arrival.0,
        oracle_voyage.arrival.0 + 30,
        "the oracle baseline cannot feed production intent lowering"
    );
}

#[test]
fn phase5_lowering_rejects_wrong_branch_and_duplicate_delta_preconditions() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(execution, &world.runtime, "storm");
    assert!(matches!(
        lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &BranchId("storm".to_owned()),
            &std::collections::BTreeSet::new(),
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
        ),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::WrongBranch {
            expected: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
            observed: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
        })
    ));
    let previously_applied = std::collections::BTreeSet::from([worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora]);
    assert!(matches!(
        lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &BranchId("storm".to_owned()),
            &previously_applied,
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
        ),
        Err(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError::DuplicateDelta(
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora
        ))
    ));
}

#[test]
fn maintenance_rewire_and_medical_lower_from_each_actual_branch_pre_state() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    prove_number_prestate(
        execution,
        NumberPrestateCase {
            branch: "maintenance",
            entity: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage, 0),
            field: "arrival",
            prestate: 12_000,
            delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
            observe: |record| match record {
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::Voyage(voyage) => voyage.arrival.0 as u64,
                _ => panic!("Aurora voyage keeps its kind"),
            },
            expected: 12_060,
        },
    );
    prove_number_prestate(
        execution,
        NumberPrestateCase {
            branch: "rewire",
            entity: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall, 1),
            field: "revision",
            prestate: 40,
            delta: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::RewireAuroraPortCall,
            observe: |record| match record {
                worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::PortCall(call) => call.revision as u64,
                _ => panic!("Aurora port call keeps its kind"),
            },
            expected: 41,
        },
    );

    let (mut world, baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(execution, &world.runtime, "medical-hold");
    let cargo = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot, 0);
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("medical-hold".to_owned()),
        update_text(&world.handles, cargo, "booking", "Available"),
    );
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId("medical-hold".to_owned()),
        &std::collections::BTreeSet::new(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::HoldMedicalCargo,
    )
    .expect("Medical Hold is an absolute transition over the actual cargo row");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("medical-hold".to_owned()),
        batch,
    );
    let worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord::CargoLot(cargo) =
        &observe_branch(execution, &mut world, "medical-hold").entities[&cargo]
    else {
        panic!("medical cargo keeps its kind");
    };
    assert_eq!(cargo.booking, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus::Held);
}
