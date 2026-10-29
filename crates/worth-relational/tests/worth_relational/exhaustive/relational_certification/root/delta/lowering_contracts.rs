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

#[test]
fn phase5_lowering_rejects_wrong_branch_and_duplicate_delta_preconditions() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(execution, &world.runtime, "storm");
    assert!(matches!(
        lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &BranchId("storm".to_owned()),
            &BTreeSet::new(),
            DeltaId::MaintainAtlasBerth,
        ),
        Err(SupplyChainProductionDeltaLoweringError::WrongBranch {
            expected: BranchLabel::Maintenance,
            observed: BranchLabel::Storm,
        })
    ));
    let previously_applied = BTreeSet::from([DeltaId::StormRerouteAurora]);
    assert!(matches!(
        lower_supply_chain_production_delta(
            execution,
            &world.runtime,
            &world.program,
            &world.handles,
            &BranchId("storm".to_owned()),
            &previously_applied,
            DeltaId::StormRerouteAurora,
        ),
        Err(SupplyChainProductionDeltaLoweringError::DuplicateDelta(
            DeltaId::StormRerouteAurora
        ))
    ));
}

#[test]
fn maintenance_rewire_and_medical_lower_from_each_actual_branch_pre_state() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    prove_number_prestate(
        execution,
        NumberPrestateCase {
            branch: "maintenance",
            entity: EntityKey::new(EntityKind::Voyage, 0),
            field: "arrival",
            prestate: 12_000,
            delta: DeltaId::MaintainAtlasBerth,
            observe: |record| match record {
                EntityRecord::Voyage(voyage) => voyage.arrival.0 as u64,
                _ => panic!("Aurora voyage keeps its kind"),
            },
            expected: 12_060,
        },
    );
    prove_number_prestate(
        execution,
        NumberPrestateCase {
            branch: "rewire",
            entity: EntityKey::new(EntityKind::PortCall, 1),
            field: "revision",
            prestate: 40,
            delta: DeltaId::RewireAuroraPortCall,
            observe: |record| match record {
                EntityRecord::PortCall(call) => call.revision as u64,
                _ => panic!("Aurora port call keeps its kind"),
            },
            expected: 41,
        },
    );

    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(execution, &world.runtime, "medical-hold");
    let cargo = EntityKey::new(EntityKind::CargoLot, 0);
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
        &BTreeSet::new(),
        DeltaId::HoldMedicalCargo,
    )
    .expect("Medical Hold is an absolute transition over the actual cargo row");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("medical-hold".to_owned()),
        batch,
    );
    let EntityRecord::CargoLot(cargo) =
        &observe_branch(execution, &mut world, "medical-hold").entities[&cargo]
    else {
        panic!("medical cargo keeps its kind");
    };
    assert_eq!(cargo.booking, BookingStatus::Held);
}
