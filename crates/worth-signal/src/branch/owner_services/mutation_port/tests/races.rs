use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdvanceDenial, SignalBranchRestoreDenial,
};
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;

use super::super::super::SignalOwnerCancellationSource;
use super::world::{set_dependency, MutationWorld};

const RACE_BOUND: Duration = Duration::from_secs(3);

#[derive(Debug)]
enum RaceResult {
    Performed(AdmittedSignalBranchBasis),
    BasisMismatch,
    Unexpected(String),
}

#[derive(Clone, Copy)]
enum AdvanceChoice {
    InputBOnly,
    BothInputs,
}

#[test]
fn same_basis_advance_race_forces_each_effectful_request_as_legal_winner() {
    assert_uncontended_advance(AdvanceChoice::InputBOnly);
    assert_uncontended_advance(AdvanceChoice::BothInputs);
    force_advance_winner(AdvanceChoice::InputBOnly, AdvanceChoice::BothInputs);
    force_advance_winner(AdvanceChoice::BothInputs, AdvanceChoice::InputBOnly);
}

#[test]
fn advance_restore_race_has_one_movement_and_a_healthy_follow_up() {
    let world = MutationWorld::<()>::new();
    let captured = world
        .port
        .capture_exact(
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("race snapshot setup performs");
    let expected = captured.captured_basis().clone();
    let snapshot = captured.admitted_snapshot().clone();
    let barrier = Arc::new(Barrier::new(3));
    let (send, receive) = mpsc::sync_channel(2);
    let before = world.owner.cost_snapshot();

    let advance_port = world.port.clone();
    let advance_expected = expected.clone();
    let advance_barrier = Arc::clone(&barrier);
    let advance_send = send.clone();
    let derived = world.derived;
    let input_b = world.input_b;
    let advance = thread::spawn(move || {
        advance_barrier.wait();
        let result = advance_port.advance_exact(
            &advance_expected,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| set_dependency(transaction, derived, input_b),
        );
        let _ = advance_send.send(map_advance_result(result));
    });
    let restore_port = world.port.clone();
    let restore_expected = expected;
    let restore_barrier = Arc::clone(&barrier);
    let restore_send = send;
    let restore = thread::spawn(move || {
        restore_barrier.wait();
        let result = restore_port.restore_exact(
            &restore_expected,
            &snapshot,
            &SignalOwnerCancellationSource::new().token(),
        );
        let _ = restore_send.send(map_restore_result(result));
    });
    barrier.wait();
    let results = receive_two(&receive);
    advance.join().expect("advance contender exits");
    restore.join().expect("restore contender exits");
    let winner = assert_one_performed_one_stale(results);
    let after = world.owner.cost_snapshot();
    assert_eq!(
        after.canonical_movements(),
        before.canonical_movements() + 1
    );
    let restore_won = winner
        .observation()
        .target()
        .as_basis()
        .and_then(|target| target.restore_snapshot_id())
        .is_some();
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        if restore_won {
            vec![world.input_a]
        } else {
            vec![world.input_b]
        },
        "the independently observed state identifies the actual mixed-race winner"
    );
    world
        .port
        .advance_exact(
            &winner,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the mixed-race winner supports a healthy follow-up");
}

#[test]
fn restore_restore_race_has_one_movement_and_preserves_the_winning_snapshot() {
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
            &winner,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the restore winner supports a healthy follow-up");
}

fn assert_uncontended_advance(choice: AdvanceChoice) {
    let world = MutationWorld::<()>::new();
    let outcome = world
        .port
        .advance_exact(
            &world.source_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| apply_choice(transaction, &world, choice),
        )
        .expect("each race request performs in an uncontended twin");
    assert_eq!(outcome.advanced_basis().observation().generation().get(), 1);
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        expected_sources(&world, choice)
    );
}

fn force_advance_winner(winner: AdvanceChoice, loser: AdvanceChoice) {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let (entered_send, entered_receive) = mpsc::sync_channel(1);
    let (release_send, release_receive) = mpsc::sync_channel(1);
    let (result_send, result_receive) = mpsc::sync_channel(2);
    let winner_port = world.port.clone();
    let winner_basis = world.source_basis.clone();
    let derived = world.derived;
    let input_a = world.input_a;
    let input_b = world.input_b;
    let winner_results = result_send.clone();
    let winner_thread = thread::spawn(move || {
        let result = winner_port.advance_exact(
            &winner_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| {
                let _ = entered_send.send(());
                release_receive
                    .recv_timeout(RACE_BOUND)
                    .expect("winner park has a bounded release");
                apply_choice_nodes(transaction, derived, input_a, input_b, winner)
            },
        );
        let _ = winner_results.send(map_advance_result(result));
    });
    entered_receive
        .recv_timeout(RACE_BOUND)
        .expect("the chosen winner reaches its real transaction callback");

    let loser_port = world.port.clone();
    let loser_basis = world.source_basis.clone();
    let loser_results = result_send;
    let loser_thread = thread::spawn(move || {
        let result = loser_port.advance_exact(
            &loser_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |transaction| apply_choice_nodes(transaction, derived, input_a, input_b, loser),
        );
        let _ = loser_results.send(map_advance_result(result));
    });
    wait_for_cell_wait(&world, before.target_cell_waits());
    release_send.send(()).expect("the winner park releases");
    let results = receive_two(&result_receive);
    winner_thread.join().expect("winner thread exits");
    loser_thread.join().expect("loser thread exits");
    assert_one_performed_one_stale(results);
    let after = world.owner.cost_snapshot();
    assert_eq!(
        after.canonical_movements(),
        before.canonical_movements() + 1
    );
    assert_eq!(
        after.owner_upgrade_attempts(),
        before.owner_upgrade_attempts() + 2
    );
    assert_eq!(
        after.branch_registry_lookups(),
        before.branch_registry_lookups() + 2
    );
    assert_eq!(
        after.target_cell_contacts(),
        before.target_cell_contacts() + 2
    );
    assert_eq!(after.target_cell_waits(), before.target_cell_waits() + 1);
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        expected_sources(&world, winner)
    );
}

fn wait_for_cell_wait(world: &MutationWorld<()>, prior_waits: u64) {
    let deadline = Instant::now() + RACE_BOUND;
    while world.owner.cost_snapshot().target_cell_waits() == prior_waits {
        assert!(
            Instant::now() < deadline,
            "loser did not reach real cell contention"
        );
        thread::yield_now();
    }
}

fn receive_two(receiver: &Receiver<RaceResult>) -> [RaceResult; 2] {
    [
        receiver
            .recv_timeout(RACE_BOUND)
            .expect("first contender settles"),
        receiver
            .recv_timeout(RACE_BOUND)
            .expect("second contender settles"),
    ]
}

fn assert_one_performed_one_stale(results: [RaceResult; 2]) -> AdmittedSignalBranchBasis {
    let mut performed = None;
    let mut stale = 0;
    for result in results {
        match result {
            RaceResult::Performed(basis) => performed = Some(basis),
            RaceResult::BasisMismatch => stale += 1,
            RaceResult::Unexpected(error) => panic!("unexpected race result: {error}"),
        }
    }
    assert_eq!(stale, 1);
    performed.expect("exactly one request performs")
}

fn map_advance_result(
    result: Result<crate::branch::SignalBranchAdvanceOutcome, SignalBranchAdvanceDenial>,
) -> RaceResult {
    match result {
        Ok(outcome) => RaceResult::Performed(outcome.into_basis()),
        Err(SignalBranchAdvanceDenial::BasisMismatch { .. }) => RaceResult::BasisMismatch,
        Err(denial) => RaceResult::Unexpected(format!("{denial:?}")),
    }
}

fn map_restore_result(
    result: Result<AdmittedSignalBranchBasis, SignalBranchRestoreDenial>,
) -> RaceResult {
    match result {
        Ok(basis) => RaceResult::Performed(basis),
        Err(SignalBranchRestoreDenial::BasisMismatch { .. }) => RaceResult::BasisMismatch,
        Err(denial) => RaceResult::Unexpected(format!("{denial:?}")),
    }
}

fn apply_choice(
    transaction: &mut crate::logic::transaction::SignalTransaction<'_, (), (), (), (), ()>,
    world: &MutationWorld<()>,
    choice: AdvanceChoice,
) -> Result<(), crate::data::error::SignalError> {
    apply_choice_nodes(
        transaction,
        world.derived,
        world.input_a,
        world.input_b,
        choice,
    )
}

fn apply_choice_nodes(
    transaction: &mut crate::logic::transaction::SignalTransaction<'_, (), (), (), (), ()>,
    derived: crate::data::handle::NodeId,
    input_a: crate::data::handle::NodeId,
    input_b: crate::data::handle::NodeId,
    choice: AdvanceChoice,
) -> Result<(), crate::data::error::SignalError> {
    match choice {
        AdvanceChoice::InputBOnly => set_dependency(transaction, derived, input_b),
        AdvanceChoice::BothInputs => transaction.set_dependencies(
            derived,
            [
                DependencyEdge::new(input_a, Aspect::new(0)),
                DependencyEdge::new(input_b, Aspect::new(0)),
            ],
        ),
    }
}

fn expected_sources(
    world: &MutationWorld<()>,
    choice: AdvanceChoice,
) -> Vec<crate::data::handle::NodeId> {
    match choice {
        AdvanceChoice::InputBOnly => vec![world.input_b],
        AdvanceChoice::BothInputs => vec![world.input_a, world.input_b],
    }
}
