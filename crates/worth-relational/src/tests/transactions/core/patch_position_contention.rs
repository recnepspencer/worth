use std::sync::Arc;
use std::time::Duration;

use crate::facade::inspection::RelationalMvccCostScope;
use crate::facade::mvcc::{
    BranchBoundRelationalTransaction, PreparedRelationalCommitCandidate,
    RelationalOperationControl, RelationalTransactionIntent,
};
use crate::runtime::RelationalPatchPositionReservationGate;
use crate::tests::support::*;

/// Failure budget for the pause handshake. Ordering comes from the seam signal,
/// never from elapsed time; this only bounds a regression that moves or deletes
/// the seam so it reports by name instead of stalling the run.
const PAUSE_SIGNAL_BUDGET: Duration = Duration::from_secs(30);

/// Two unrelated branches publish at once. The winner is held inside its
/// cutover, the one region that provably runs while the single global
/// patch-position reservation is in hand, so the second publisher meets a taken
/// reservation as a scheduled fact rather than a timing coincidence. Exactly one
/// attempt performs, the other is a typed no-movement deferral, the allocator
/// counters are exact, and the deferred branch gives back everything it took.
#[test]
fn a_held_patch_position_reservation_defers_an_unrelated_branch_without_residue() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 8,
            max_active_snapshot_handles: 16,
            max_transaction_overlay_bytes: 1_048_576,
            max_transaction_footprint_loci: 1_024,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 2,
            candidate_max_lifetime_millis: 30_000,
            max_prepared_root_bytes: 268_435_456,
        })
        .build();
    create_entity(&runtime, "patch-position-contention-anchor");
    fork_contention_branch(&runtime, "winner");
    fork_contention_branch(&runtime, "loser");

    let loser_identity = runtime
        .branch_identity(&BranchId("loser".to_owned()))
        .expect("the deferred branch is registered");
    let loser_scope = RelationalMvccCostScope::capture(&runtime, vec![loser_identity]);
    let commits_before = runtime.history().immutable_commit_count();
    let pending_routes_before = runtime.history.pending_canonical_publication_route_count();
    let loser_reference_before = runtime
        .branch_reference_state(&BranchId("loser".to_owned()))
        .expect("the deferred branch has an observable reference");

    let (reached, reached_signal) = std::sync::mpsc::sync_channel::<()>(1);
    let release = Arc::new(RelationalPatchPositionReservationGate::default());
    let winner = prepared_write_on(
        &runtime,
        "winner",
        "contention-winner",
        RelationalOperationControl::uninterrupted()
            .with_patch_position_reservation_pause(reached, Arc::clone(&release)),
    );
    let winner_cell = winner.publication_cell_for_test();
    let loser = prepared_write_on(
        &runtime,
        "loser",
        "contention-loser",
        RelationalOperationControl::uninterrupted(),
    );
    let loser_cell = loser.publication_cell_for_test();
    let position_before = runtime.patch_position_reservation_counters();

    let winner_port = runtime.publication_port();
    let winner_thread = std::thread::spawn(move || winner_port.compare_and_publish(winner));
    // The seam signal, not elapsed time, is what orders this test: the winner is
    // inside its cutover from here until the gate is opened. A seam that is
    // never reached times out by name rather than stalling the run.
    reached_signal
        .recv_timeout(PAUSE_SIGNAL_BUDGET)
        .expect("the winner must reach the reservation-held pause inside its cutover");
    let position_at_pause = runtime.patch_position_reservation_counters();
    assert!(
        !winner_thread.is_finished(),
        "the winner is held inside its cutover, reservation in hand"
    );
    let winner_contacts_at_pause = winner_cell.coordination().contact_count();
    let winner_waits_at_pause = winner_cell.coordination().wait_count();
    let loser_contacts_at_pause = loser_cell.coordination().contact_count();
    let loser_waits_at_pause = loser_cell.coordination().wait_count();
    let settlements_at_pause = runtime.publication_binding().pending_settlement_count();
    let settlement_contacts_at_pause = runtime
        .publication_binding()
        .pending_settlement_contact_count();

    let loser_outcome = runtime.publication_port().compare_and_publish(loser);

    // Every observation is taken before the winner is released, so the order of
    // the assertions below cannot change what any of them saw.
    let position_after_deferral = runtime.patch_position_reservation_counters();
    let winner_contacts_after_deferral = winner_cell.coordination().contact_count();
    let winner_waits_after_deferral = winner_cell.coordination().wait_count();
    let loser_contacts_after_deferral = loser_cell.coordination().contact_count();
    let loser_waits_after_deferral = loser_cell.coordination().wait_count();
    let settlements_after_deferral = runtime.publication_binding().pending_settlement_count();
    let settlement_contacts_after_deferral = runtime
        .publication_binding()
        .pending_settlement_contact_count();
    let loser_reference_at_deferral = runtime
        .branch_reference_state(&BranchId("loser".to_owned()))
        .expect("the deferred branch stays registered");
    // Opening the gate never blocks, so every assertion below runs with the
    // winner already free and no path left that can strand it.
    release.open();

    assert_eq!(
        position_at_pause.assignments,
        position_before.assignments + 1,
        "the pause must hold the single patch-position reservation, not a point elsewhere on the publication path"
    );
    assert_eq!(
        position_at_pause.deferrals, position_before.deferrals,
        "nothing is turned away until the loser meets the held reservation"
    );
    assert!(
        matches!(
            &loser_outcome,
            crate::mvcc::RelationalPublicationOutcome::Deferred(
                crate::mvcc::RelationalPublicationDeferred::PatchPositionReservationContended
            )
        ),
        "a held patch position defers by type, not by failure: {loser_outcome:?}"
    );
    assert_eq!(
        position_after_deferral.contacts,
        position_before.contacts + 2,
        "both publishers reach the reservation exactly once"
    );
    assert_eq!(
        position_after_deferral.deferrals,
        position_before.deferrals + 1,
        "exactly one publisher is turned away"
    );
    assert_eq!(
        position_after_deferral.assignments,
        position_before.assignments + 1,
        "exactly one publisher is assigned a patch position"
    );
    assert_eq!(
        position_after_deferral.overflows, 0,
        "contention is not capacity exhaustion"
    );
    assert_eq!(
        loser_waits_after_deferral, loser_waits_at_pause,
        "the loser is turned away by the global reservation, never by a branch wait"
    );
    assert_eq!(
        loser_contacts_after_deferral,
        loser_contacts_at_pause + 1,
        "the loser enters its own branch coordination exactly once"
    );
    assert_eq!(
        winner_contacts_after_deferral, winner_contacts_at_pause,
        "the deferred publisher never contacts the held branch"
    );
    assert_eq!(winner_waits_after_deferral, winner_waits_at_pause);
    assert_eq!(
        settlements_at_pause, 1,
        "the paused winner already installed its pre-effect settlement record"
    );
    assert_eq!(
        settlements_after_deferral, 1,
        "the deferred attempt leaves exactly the winner's settlement record"
    );
    assert!(
        settlement_contacts_after_deferral > settlement_contacts_at_pause,
        "the deferred attempt did reach settlement admission before releasing"
    );
    assert_eq!(
        loser_reference_at_deferral, loser_reference_before,
        "a contention deferral performs no movement"
    );

    let performed = match winner_thread.join().expect("the winner publisher joins") {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("the reservation holder performs: {outcome:?}"),
    };
    let settled = runtime
        .settle_performed_publication(performed)
        .expect("the winner settles through its installed record");

    let loser_cost = runtime
        .observe_mvcc_counters(&loser_scope)
        .expect("the deferred branch keeps its counter owner");
    let loser_retention = loser_cost.retention_cost_delta();
    assert_eq!(
        loser_retention.candidate_acquires, loser_retention.candidate_releases,
        "the deferred branch released every candidate it acquired"
    );
    assert_eq!(
        loser_retention.candidate_acquires, 1,
        "the deferred branch acquired exactly the one candidate it prepared"
    );
    assert_eq!(
        loser_retention.observation_acquires, loser_retention.observation_releases,
        "the deferred branch released every basis observation it acquired"
    );
    assert_eq!(
        loser_retention.performed_settlement_acquires, 0,
        "a deferred attempt never becomes a performed settlement"
    );
    assert_eq!(loser_retention.head_installs, 0);
    assert_eq!(loser_retention.head_transfers, 0);
    assert_eq!(loser_cost.branch_population_scans(), 0);
    assert_eq!(
        runtime.publication_binding().pending_settlement_count(),
        0,
        "the settled winner leaves no record and the loser never installed one"
    );
    assert_eq!(
        runtime.history.pending_canonical_publication_route_count(),
        pending_routes_before,
        "no route survives the contended attempt"
    );
    assert_eq!(
        runtime.history().immutable_commit_count(),
        commits_before + 1,
        "contention admits exactly one commit"
    );
    assert_eq!(
        runtime
            .branch_reference_state(&BranchId("loser".to_owned()))
            .expect("the deferred branch stays registered"),
        loser_reference_before,
        "the deferred branch is where it started"
    );

    // The candidate slot is the one residue a counter cannot fake: if the
    // deferral kept it, the second refill below would be refused instead.
    let refill = prepared_write_on(
        &runtime,
        "loser",
        "post-contention-refill",
        RelationalOperationControl::uninterrupted(),
    );
    let second_refill = prepared_write_on(
        &runtime,
        "winner",
        "post-contention-second-refill",
        RelationalOperationControl::uninterrupted(),
    );
    let mut overflow = begin_contention_transaction(
        &runtime,
        "loser",
        RelationalOperationControl::uninterrupted(),
    );
    overflow
        .push_batch(
            batch_create("post-contention-overflow"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    assert!(
        matches!(
            runtime.prepare_branch_transaction(
                overflow,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            ),
            Err(TransactionCommitError::PublicationDeferred {
                deferred: crate::mvcc::RelationalPublicationDeferred::CandidateCapacityExhausted {
                    maximum_candidates: 2,
                },
                ..
            })
        ),
        "the contended attempt returned its candidate slot and no more"
    );
    runtime.discard_prepared_candidate(refill).unwrap();
    runtime.discard_prepared_candidate(second_refill).unwrap();
    release_test_commit_snapshot(&runtime, &settled);
}

fn fork_contention_branch(runtime: &RelationalRuntime, branch: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main has an exact fork source");
    runtime
        .fork_branch(BranchId(branch.to_owned()), source)
        .expect("contention branch fork succeeds");
}

fn begin_contention_transaction(
    runtime: &RelationalRuntime,
    branch: &str,
    control: RelationalOperationControl,
) -> BranchBoundRelationalTransaction {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("contention branch identity exists");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("contention branch basis is admitted");
    runtime
        .begin_branch_transaction_with_control(
            &basis,
            RelationalTransactionIntent::ordinary(),
            control,
        )
        .expect("contention transaction binds")
}

fn prepared_write_on(
    runtime: &RelationalRuntime,
    branch: &str,
    entity: &str,
    control: RelationalOperationControl,
) -> PreparedRelationalCommitCandidate {
    let mut transaction = begin_contention_transaction(runtime, branch, control);
    transaction
        .push_batch(
            batch_create(entity),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("contention candidate prepares")
}
