use super::super::world::AdversarialWorld;
use super::ordered_race::run;
use super::races::{
    advance, capture, prove_advance_is_effectful, prove_capture_and_restore_are_effectful, restore,
};

fn assert_one_success(
    left: Result<(), &'static str>,
    right: Result<(), &'static str>,
    label: &str,
) {
    assert_eq!(
        [left.is_ok(), right.is_ok()]
            .into_iter()
            .filter(|succeeded| *succeeded)
            .count(),
        1,
        "{label} must have exactly one canonical winner: left={left:?} right={right:?}"
    );
}

fn control(world: &AdversarialWorld) -> worth_signal::facade::branch::SignalOwnerOperationControl {
    world
        .runtime
        .as_ref()
        .expect("the race retains its owner root")
        .owner_operation_control()
        .expect("operation control is issued after sealing")
}

fn run_advance_advance(left_parked: bool) {
    let world = AdversarialWorld::new();
    let control = control(&world);
    let left_mutation = world.mutation.clone();
    let right_mutation = world.mutation.clone();
    let left_basis = world.child_basis.clone();
    let right_basis = world.child_basis.clone();
    let result = run(
        &world.runtime,
        &control,
        move || advance(&left_mutation, &left_basis),
        move || advance(&right_mutation, &right_basis),
        left_parked,
    );
    assert_one_success(result.0, result.1, "ordered advance/advance");
}

fn run_advance_restore(left_parked: bool) {
    let world = AdversarialWorld::new();
    let (snapshot, current) = capture(&world.mutation, &world.child_basis)
        .expect("the race has a real admitted snapshot");
    let control = control(&world);
    let left_mutation = world.mutation.clone();
    let right_mutation = world.mutation.clone();
    let left_basis = current.clone();
    let right_basis = current;
    let right_snapshot = snapshot.clone();
    let result = run(
        &world.runtime,
        &control,
        move || advance(&left_mutation, &left_basis),
        move || restore(&right_mutation, &right_basis, &right_snapshot),
        left_parked,
    );
    assert_one_success(result.0, result.1, "ordered advance/restore");
}

fn run_restore_restore(left_parked: bool) {
    let world = AdversarialWorld::new();
    let (snapshot, current) = capture(&world.mutation, &world.child_basis)
        .expect("the race has a real admitted snapshot");
    let control = control(&world);
    let left_mutation = world.mutation.clone();
    let right_mutation = world.mutation.clone();
    let left_basis = current.clone();
    let right_basis = current;
    let left_snapshot = snapshot.clone();
    let right_snapshot = snapshot;
    let result = run(
        &world.runtime,
        &control,
        move || restore(&left_mutation, &left_basis, &left_snapshot),
        move || restore(&right_mutation, &right_basis, &right_snapshot),
        left_parked,
    );
    assert_one_success(result.0, result.1, "ordered restore/restore");
}

fn run_snapshot_advance(left_parked: bool) {
    let world = AdversarialWorld::new();
    let control = control(&world);
    let left_mutation = world.mutation.clone();
    let right_mutation = world.mutation.clone();
    let left_basis = world.child_basis.clone();
    let right_basis = world.child_basis.clone();
    let result = run(
        &world.runtime,
        &control,
        move || capture(&left_mutation, &left_basis).map(|_| ()),
        move || advance(&right_mutation, &right_basis),
        left_parked,
    );
    assert_one_success(result.0, result.1, "ordered snapshot/advance");
}

#[test]
fn every_independently_admissible_same_branch_pair_forces_both_legal_winner_orderings() {
    prove_advance_is_effectful();
    prove_capture_and_restore_are_effectful();
    for left_parked in [true, false] {
        run_advance_advance(left_parked);
        run_advance_restore(left_parked);
        run_restore_restore(left_parked);
        run_snapshot_advance(left_parked);
    }
}
