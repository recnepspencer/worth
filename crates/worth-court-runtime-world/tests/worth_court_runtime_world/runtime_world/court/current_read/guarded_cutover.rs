use super::*;
use crate::court::ordinary_context::operation_control::combined_race::ReleaseRelational;
use crate::court::ordinary_context::operation_control::{clean, serial_request, WAIT};
use std::sync::{mpsc, Arc};
use worth_relational::facade::mvcc::{
    RelationalOperationControl, RelationalPatchPositionReservationGateForTest,
};

#[test]
fn exact_control_guard_spans_native_cutover_and_product_loss_retains_actual_movement() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let current = court.bootstrap();
    let target = fork(&court, &execution, &current);
    let policy = match court.world.execution_placement() {
        RuntimeWorldExecutionPlacement::Serial(policy)
        | RuntimeWorldExecutionPlacement::Leased { policy, .. } => policy,
    };
    let token = RuntimeWorldCancellationSource::new().token();
    let (reached, arrival) = mpsc::sync_channel(1);
    let gate = Arc::new(RelationalPatchPositionReservationGateForTest::default());
    let candidate = court.records.candidate_with_control(
        &execution,
        target.basis().relational_basis(),
        "grain",
        "5",
        RelationalOperationControl::uninterrupted()
            .with_patch_position_reservation_pause_for_test(reached, Arc::clone(&gate)),
    );
    let prepared = court
        .world
        .publication_port()
        .prepare_without_signal(
            target.clone(),
            intent(
                CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                    .with_prepared_relational_candidate(candidate),
                &current,
            ),
            &token,
            None,
        )
        .unwrap();
    let observation = court.world.observation_port();
    assert!(!observation
        .product_branch_writers_are_locked_out_for_test(&current)
        .unwrap());
    std::thread::scope(|threads| {
        let release = ReleaseRelational(gate);
        let port = court.world.publication_port();
        let worker = threads.spawn(move || {
            port.execute_without_signal(
                worth_execution::ExecutionRequest::serial(&serial_request(policy)),
                prepared,
                &token,
            )
        });
        let arrived = arrival.recv_timeout(WAIT);
        if arrived.is_err() {
            drop(release);
            let result = worker.join().unwrap();
            panic!("Native cutover did not reach its real latch: {result:?}");
        }
        // These probes inspect the actual C and D World cells, not a proxy
        // latch or an inference from a contender's elapsed time.
        let current_locked = observation
            .product_branch_writers_are_locked_out_for_test(&current)
            .unwrap();
        let target_locked = observation
            .product_branch_writers_are_locked_out_for_test(&target)
            .unwrap();
        // A disjoint D World writer can win before the original Native effect.
        let winner = move_signal(&court, &execution, &target);
        drop(release);
        let result = worker.join().unwrap();
        assert!(current_locked);
        assert!(!target_locked);
        assert!(!observation
            .product_branch_writers_are_locked_out_for_test(&current)
            .unwrap());
        let RuntimeWorldPublicationOutcome::ProductUnpublished(effects) = result else {
            panic!("D loss after real Native movement retains recovery custody");
        };
        assert_eq!(effects.owner_effect_count(), 1);
        assert_eq!(effects.cause(), ProductUnpublishedCause::StaleProductHead);
        assert_eq!(court.observe(&target), winner);
        assert_eq!(grain(&court, &execution, &winner), "4");
        let actual = direct_basis(&court, &target);
        assert_eq!(
            court.records.read(&execution, &actual).records["grain"],
            "5"
        );
        let control_move = court.publish_cargo(&current, "9");
        clean(
            &court,
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects),
        );
        assert_eq!(direct_basis(&court, &target), actual);
        assert_eq!(court.observe(&target), winner);
        drop(control_move);
    });
}
