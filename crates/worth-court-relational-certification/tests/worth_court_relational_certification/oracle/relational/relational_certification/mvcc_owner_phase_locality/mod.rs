use crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::commit_unrelated_branch_while_paused::commit_unrelated_branch_while_paused;
#[allow(unused_imports)]
pub(crate) use {crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::BranchCoordination, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::OwnerPhaseCourtBranches, crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::OwnerPhasePause, worth_relational::facade::mvcc::RelationalInterruptionBoundary, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::assert_court_commit_matches_oracle, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::begin_court_transaction, crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::branch_reference_state, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::coordination_counters, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::head_for_supply_chain_branch, crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::join_paused_worker, crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::lower_court_delta};

#[test]
fn paused_supply_chain_preparation_leaves_an_unrelated_branch_commit_unblocked() {
    let host_request = std::sync::Arc::new(
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(),
    );
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, expected) =
        certified_supply_chain_world(request_execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(request_execution, &world, &expected);
    let court = crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::OwnerPhaseCourtBranches::fork(request_execution, &world);
    let storm_batch = lower_court_delta(
        request_execution,
        &world,
        &court.storm,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    );
    let maintenance_batch = lower_court_delta(
        request_execution,
        &world,
        &court.maintenance,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
    );

    let pause = crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::OwnerPhasePause::new();
    let storm_transaction = begin_court_transaction(
        &world.runtime,
        &court.storm,
        pause.control(
            worth_relational::facade::mvcc::RelationalInterruptionBoundary::CandidatePreparation,
        ),
        storm_batch,
    );
    let preparation = world.runtime.preparation_port();
    let storm_worker = std::thread::spawn(move || {
        let host_request = std::sync::Arc::new(
            worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(),
        );
        let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);
        preparation.prepare_branch_transaction(request_execution, storm_transaction)
    });
    pause.await_arrival("candidate preparation");

    let before = court.capture(&world.runtime);
    let committed = commit_unrelated_branch_while_paused(
        request_execution,
        &world,
        &court,
        &before,
        maintenance_batch,
        &pause,
        "candidate preparation",
    );
    assert!(
        !storm_worker.is_finished(),
        "the storm branch must stay parked in candidate preparation for the whole unrelated commit"
    );
    before.assert_branch_locality(&court, &world.runtime, &committed);

    let storm_candidate = join_paused_worker(storm_worker, &pause, "candidate preparation")
        .expect("the storm branch prepares once its owner-phase pause opens");
    let preparation = world.runtime.preparation_port();
    preparation
        .discard_prepared_candidate(storm_candidate)
        .expect("the released storm candidate discards through its own owner port");
    assert_court_commit_matches_oracle(
        request_execution,
        &world,
        &committed,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
    );
    assert_oracle_matches(request_execution, &world, &expected);
}

#[test]
fn paused_supply_chain_publication_leaves_an_unrelated_branch_commit_unblocked() {
    let host_request = std::sync::Arc::new(
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(),
    );
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, expected) =
        certified_supply_chain_world(request_execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(request_execution, &world, &expected);
    let court = crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::OwnerPhaseCourtBranches::fork(request_execution, &world);
    let storm_batch = lower_court_delta(
        request_execution,
        &world,
        &court.storm,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    );
    let maintenance_batch = lower_court_delta(
        request_execution,
        &world,
        &court.maintenance,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
    );

    let pause = crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::OwnerPhasePause::new();
    let storm_transaction = begin_court_transaction(
        &world.runtime,
        &court.storm,
        pause.control(
            worth_relational::facade::mvcc::RelationalInterruptionBoundary::BeforeCriticalSection,
        ),
        storm_batch,
    );
    let storm_candidate = world
        .runtime
        .preparation_port()
        .prepare_branch_transaction(request_execution, storm_transaction)
        .expect("the storm candidate prepares before the publication court opens");
    let publication = world.runtime.publication_port();
    let storm_worker = std::thread::spawn({
        let publication_request = std::sync::Arc::clone(&host_request);
        move || {
            let request_execution =
                &worth_execution::ExecutionRequest::serial(&publication_request);
            publication.compare_and_publish(request_execution, storm_candidate)
        }
    });
    pause.await_arrival("publication critical-section entry");

    let before = court.capture(&world.runtime);
    let committed = commit_unrelated_branch_while_paused(
        request_execution,
        &world,
        &court,
        &before,
        maintenance_batch,
        &pause,
        "publication critical-section entry",
    );
    assert!(
        !storm_worker.is_finished(),
        "the storm branch must stay parked at its critical section for the whole unrelated commit"
    );
    before.assert_branch_locality(&court, &world.runtime, &committed);

    let performed =
        match join_paused_worker(storm_worker, &pause, "publication critical-section entry") {
            worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) => {
                performed
            }
            outcome => panic!("the released storm publication must perform: {outcome:?}"),
        };
    let storm_committed = world
        .runtime
        .settle_performed_publication(performed)
        .expect("the released storm publication settles through its owner runtime");
    world
        .runtime
        .snapshots()
        .release_snapshot(&storm_committed.snapshot)
        .expect("the exact storm settlement snapshot closes");
    assert_ne!(
        branch_reference_state(&world.runtime, &court.storm),
        before.storm_reference,
        "the released storm publication moves its own branch reference"
    );
    assert_eq!(
        coordination_counters(&world.runtime, &court.storm),
        BranchCoordination {
            contacts: before.storm_coordination.contacts + 1,
            waits: before.storm_coordination.waits,
        },
        "storm's own publication charges exactly one uncontended storm contact, so the zero-delta claim measures a live counter"
    );
    assert_court_commit_matches_oracle(
        request_execution,
        &world,
        &committed,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
    );
    assert_oracle_matches(request_execution, &world, &expected);
}

#[test]
fn paused_supply_chain_settlement_leaves_an_unrelated_branch_commit_unblocked() {
    let host_request = std::sync::Arc::new(
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(),
    );
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, expected) =
        certified_supply_chain_world(request_execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(request_execution, &world, &expected);
    let court = crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::OwnerPhaseCourtBranches::fork(request_execution, &world);
    let storm_batch = lower_court_delta(
        request_execution,
        &world,
        &court.storm,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    );
    let maintenance_batch = lower_court_delta(
        request_execution,
        &world,
        &court.maintenance,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
    );

    let pause = crate::relational_certification::ordinary_context::mvcc_owner_phase_locality_court::OwnerPhasePause::new();
    let storm_transaction = begin_court_transaction(
        &world.runtime,
        &court.storm,
        pause.control(worth_relational::facade::mvcc::RelationalInterruptionBoundary::Settlement),
        storm_batch,
    );
    let storm_reference_before_movement = branch_reference_state(&world.runtime, &court.storm);
    let storm_candidate = world
        .runtime
        .preparation_port()
        .prepare_branch_transaction(request_execution, storm_transaction)
        .expect("the storm candidate prepares before the settlement court opens");
    let performed = match world
        .runtime
        .publication_port()
        .compare_and_publish(request_execution, storm_candidate)
    {
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) => {
            performed
        }
        outcome => panic!("the uncontended storm candidate must perform: {outcome:?}"),
    };
    let storm_commit_id = performed.canonical_commit().commit.commit_id;

    // Prove the park is where this court says it is before relying on it. Only
    // a branch that already moved and still retains its runtime-owned pending
    // record is inside settlement, so an earlier boundary firing here cannot be
    // mistaken for the settlement executor.
    let settlement = world.runtime.settlement_port();
    assert_ne!(
        branch_reference_state(&world.runtime, &court.storm),
        storm_reference_before_movement,
        "the storm reference moves before settlement, so the park is past publication"
    );
    assert!(
        settlement.retains_pending_settlement(storm_commit_id),
        "the moved storm route retains its runtime-owned pending settlement record"
    );

    let settling = settlement.clone();
    let storm_worker = std::thread::spawn(move || settling.settle_performed_publication(performed));
    pause.await_arrival("settlement executor");

    let before = court.capture(&world.runtime);
    let maintenance_before = coordination_counters(&world.runtime, &court.maintenance);
    let committed = commit_unrelated_branch_while_paused(
        request_execution,
        &world,
        &court,
        &before,
        maintenance_batch,
        &pause,
        "settlement executor",
    );
    assert!(
        !storm_worker.is_finished(),
        "the storm branch must stay parked in its settlement executor for the whole unrelated commit"
    );
    before.assert_branch_locality(&court, &world.runtime, &committed);
    // The unrelated branch enters its own coordination cell exactly twice, once
    // admitting its transaction and once for its publication critical section,
    // and never waits. The same counter that reads zero on storm moves here in
    // the same window, so the zero-delta storm claim measures a live counter.
    assert_eq!(
        coordination_counters(&world.runtime, &court.maintenance),
        BranchCoordination {
            contacts: maintenance_before.contacts + 2,
            waits: maintenance_before.waits,
        },
        "the unrelated ordinary commit charges its own admission and cutover contacts uncontended"
    );
    assert!(
        settlement.retains_pending_settlement(storm_commit_id),
        "the parked storm record stays installed for the whole unrelated commit"
    );
    assert!(
        !settlement.retains_pending_settlement(committed.commit.commit_id),
        "the unrelated branch reserved, used, and released its own pending record while storm held its own"
    );
    assert_court_commit_matches_oracle(
        request_execution,
        &world,
        &committed,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Maintenance,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::MaintainAtlasBerth,
    );

    let storm_committed = join_paused_worker(storm_worker, &pause, "settlement executor")
        .expect("the storm branch settles once its owner-phase pause opens");
    assert_eq!(
        storm_committed.commit.commit_id, storm_commit_id,
        "the released settlement finishes the exact commit identity it parked on"
    );
    assert_eq!(
        head_for_supply_chain_branch(&world.runtime, &court.storm),
        storm_committed.commit,
        "the observed storm head is exactly the settled canonical commit"
    );
    assert!(
        !settlement.retains_pending_settlement(storm_commit_id),
        "finished settlement leaves no pending record behind"
    );
    assert_court_commit_matches_oracle(
        request_execution,
        &world,
        &storm_committed,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Storm,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::StormRerouteAurora,
    );
    assert_oracle_matches(request_execution, &world, &expected);
}
