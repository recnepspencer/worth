//! Restore restore race has one movement and preserves the winning snapshot.

use super::*;

#[test]
fn restore_restore_race_has_one_movement_and_preserves_the_winning_snapshot() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let world = MutationWorld::<()>::new();
    let snapshot_a = world
        .port
        .capture_exact(
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("snapshot A captures input A");
    let advanced_b = world
        .port
        .advance_exact(
            request_execution,
            snapshot_a.captured_basis(),
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| set_dependency(transaction, world.derived, world.input_b),
        )
        .expect("intervening state changes to input B");
    let snapshot_b = world
        .port
        .capture_exact(
            advanced_b.advanced_basis(),
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("snapshot B captures input B");
    let expected = snapshot_b.captured_basis().clone();
    let snapshot_a_id = snapshot_a.admitted_snapshot().snapshot().meta.snapshot_id;
    let snapshot_b_id = snapshot_b.admitted_snapshot().snapshot().meta.snapshot_id;
    let barrier = Arc::new(Barrier::new(3));
    let (send, receive) = mpsc::sync_channel(2);
    let before = world.owner.cost_snapshot();

    let mut contenders = Vec::new();
    for snapshot in [
        snapshot_a.admitted_snapshot().clone(),
        snapshot_b.admitted_snapshot().clone(),
    ] {
        let port = world.port.clone();
        let expected = expected.clone();
        let barrier = Arc::clone(&barrier);
        let send = send.clone();
        contenders.push(thread::spawn(move || {
            barrier.wait();
            let result = port.restore_exact(
                &expected,
                &snapshot,
                &SignalOwnerCancellationSource::new().token(),
            );
            let _ = send.send(map_restore_result(result));
        }));
    }
    drop(send);
    barrier.wait();
    let results = receive_two(&receive);
    for contender in contenders {
        contender.join().expect("restore contender exits");
    }
    let winner = assert_one_performed_one_stale(results);
    let after = world.owner.cost_snapshot();
    assert_eq!(
        after.canonical_movements(),
        before.canonical_movements() + 1
    );
    let winning_snapshot_id = winner
        .observation()
        .target()
        .as_basis()
        .and_then(|target| target.restore_snapshot_id())
        .expect("a restore winner records its exact snapshot identity");
    let expected_sources = if winning_snapshot_id == snapshot_a_id.0 {
        vec![world.input_a]
    } else {
        assert_eq!(winning_snapshot_id, snapshot_b_id.0);
        vec![world.input_b]
    };
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        expected_sources,
        "canonical state matches the owner-issued winning snapshot"
    );
    world
        .port
        .advance_exact(
            request_execution,
            &winner,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the restore winner supports a healthy follow-up");
}
