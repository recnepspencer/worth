//! Production-boundary locality courts for the Relational owner phases.
//!
//! Each court parks one Supply Chain branch inside a real owner phase and
//! requires an unrelated branch to complete a full ordinary commit through the
//! public facade while that park is held. A whole-runtime exclusive borrow
//! convicts at compile time, never here; what these courts convict is a runtime
//! gate, a global lock or shared cell, that serializes independent branches.
//! Preparation parks at the first `CandidatePreparation` observation near the
//! top of the phase, so it cannot speak for a gate taken later in it;
//! publication parks at `BeforeCriticalSection`, inside the entered phase and
//! before the branch coordination cell, which is the stronger position.
//! Settlement parks at the production `Settlement` observation inside the one
//! settlement executor, holding an installed pending-settlement record, that
//! record's per-commit executor gate, a published-snapshot slot, and a moved
//! but still unsettled canonical route. Its reach stops where that park does:
//! the durable append and derived completion already returned, so it cannot
//! speak for a lock taken and released inside them, and the certified world is
//! memory-resident, so no durable-I/O locality is claimed here.

use super::mvcc_owner_phase_locality_court::{
    assert_court_commit_matches_oracle, begin_court_transaction, branch_reference_state,
    commit_unrelated_branch_while_paused, coordination_counters, join_paused_worker,
    lower_court_delta, BranchCoordination, OwnerPhaseCourtBranches, OwnerPhasePause,
};
use super::world::supply_chain::{
    assert_oracle_matches, certified_supply_chain_world, head_for_supply_chain_branch, BranchLabel,
    DeltaId, SupplyChainScale,
};
use worth_relational::facade::mvcc::{
    RelationalInterruptionBoundary, RelationalPublicationOutcome,
};

#[test]
fn paused_supply_chain_preparation_leaves_an_unrelated_branch_commit_unblocked() {
    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let court = OwnerPhaseCourtBranches::fork(&world);
    let storm_batch = lower_court_delta(&world, &court.storm, DeltaId::StormRerouteAurora);
    let maintenance_batch =
        lower_court_delta(&world, &court.maintenance, DeltaId::MaintainAtlasBerth);

    let pause = OwnerPhasePause::new();
    let storm_transaction = begin_court_transaction(
        &world.runtime,
        &court.storm,
        pause.control(RelationalInterruptionBoundary::CandidatePreparation),
        storm_batch,
    );
    let preparation = world.runtime.preparation_port();
    let storm_worker = std::thread::spawn(move || {
        preparation.prepare_branch_transaction(
            storm_transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    });
    pause.await_arrival("candidate preparation");

    let before = court.capture(&world.runtime);
    let committed = commit_unrelated_branch_while_paused(
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
        &world,
        &committed,
        BranchLabel::Maintenance,
        DeltaId::MaintainAtlasBerth,
    );
    assert_oracle_matches(&world, &expected);
}

#[test]
fn paused_supply_chain_publication_leaves_an_unrelated_branch_commit_unblocked() {
    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let court = OwnerPhaseCourtBranches::fork(&world);
    let storm_batch = lower_court_delta(&world, &court.storm, DeltaId::StormRerouteAurora);
    let maintenance_batch =
        lower_court_delta(&world, &court.maintenance, DeltaId::MaintainAtlasBerth);

    let pause = OwnerPhasePause::new();
    let storm_transaction = begin_court_transaction(
        &world.runtime,
        &court.storm,
        pause.control(RelationalInterruptionBoundary::BeforeCriticalSection),
        storm_batch,
    );
    let storm_candidate = world
        .runtime
        .preparation_port()
        .prepare_branch_transaction(
            storm_transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the storm candidate prepares before the publication court opens");
    let publication = world.runtime.publication_port();
    let storm_worker = std::thread::spawn(move || publication.compare_and_publish(storm_candidate));
    pause.await_arrival("publication critical-section entry");

    let before = court.capture(&world.runtime);
    let committed = commit_unrelated_branch_while_paused(
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
            RelationalPublicationOutcome::Performed(performed) => performed,
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
        &world,
        &committed,
        BranchLabel::Maintenance,
        DeltaId::MaintainAtlasBerth,
    );
    assert_oracle_matches(&world, &expected);
}

#[test]
fn paused_supply_chain_settlement_leaves_an_unrelated_branch_commit_unblocked() {
    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let court = OwnerPhaseCourtBranches::fork(&world);
    let storm_batch = lower_court_delta(&world, &court.storm, DeltaId::StormRerouteAurora);
    let maintenance_batch =
        lower_court_delta(&world, &court.maintenance, DeltaId::MaintainAtlasBerth);

    let pause = OwnerPhasePause::new();
    let storm_transaction = begin_court_transaction(
        &world.runtime,
        &court.storm,
        pause.control(RelationalInterruptionBoundary::Settlement),
        storm_batch,
    );
    let storm_reference_before_movement = branch_reference_state(&world.runtime, &court.storm);
    let storm_candidate = world
        .runtime
        .preparation_port()
        .prepare_branch_transaction(
            storm_transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the storm candidate prepares before the settlement court opens");
    let performed = match world
        .runtime
        .publication_port()
        .compare_and_publish(storm_candidate)
    {
        RelationalPublicationOutcome::Performed(performed) => performed,
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
        &world,
        &committed,
        BranchLabel::Maintenance,
        DeltaId::MaintainAtlasBerth,
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
        &world,
        &storm_committed,
        BranchLabel::Storm,
        DeltaId::StormRerouteAurora,
    );
    assert_oracle_matches(&world, &expected);
}
