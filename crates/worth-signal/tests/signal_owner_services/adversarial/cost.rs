use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::{
    SignalBranchRetentionReleaseOutcome, SignalBranchRetirementReason,
    SignalOwnerCancellationSource,
};

use super::world::AdversarialWorld;

#[test]
fn one_public_advance_reports_one_local_structural_delta() {
    let world = AdversarialWorld::new();
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open");
    world
        .mutation
        .advance_exact(
            &world.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the populated branch advances through its public port");
    let after = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner remains open");
    assert_eq!(
        after.branch_registry_lookups() - before.branch_registry_lookups(),
        1
    );
    assert_eq!(
        after.target_cell_contacts() - before.target_cell_contacts(),
        1
    );
    assert_eq!(
        after.canonical_movements() - before.canonical_movements(),
        1
    );
    assert_eq!(
        after.branch_registry_entries_scanned() - before.branch_registry_entries_scanned(),
        0,
        "an unrelated registry population cannot enter target work"
    );
    assert_eq!(
        after.forked_mutable_graph_nodes_copied(),
        before.forked_mutable_graph_nodes_copied(),
        "ordinary movement does not copy fork state"
    );
}

#[test]
fn lifecycle_operations_report_exact_structural_deltas() {
    let world = AdversarialWorld::new();
    let child_id = world.child_basis.branch_id();
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open");
    let plan = match world
        .lifecycle
        .plan_retirement_exact(world.child_basis, SignalBranchRetirementReason::Superseded)
    {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("the owner issues a linear retirement plan: {other:?}"),
    };
    let after_plan = world
        .basis
        .owner_service_cost_snapshot()
        .expect("planning leaves the owner open");
    assert_eq!(
        after_plan.branch_registry_lookups() - before.branch_registry_lookups(),
        1,
        "planning performs one owner registry lookup"
    );
    assert_eq!(
        after_plan.target_cell_contacts() - before.target_cell_contacts(),
        1,
        "planning contacts the target cell once"
    );
    assert_eq!(
        after_plan.retention_registry_contacts() - before.retention_registry_contacts(),
        1,
        "planning performs one canonical retention read"
    );
    assert_eq!(
        after_plan.canonical_movements(),
        before.canonical_movements(),
        "planning is pre-effect"
    );

    let receipt = match world
        .lifecycle
        .retire_exact(plan, &SignalOwnerCancellationSource::new().token())
    {
        TransitionOutcome::Success(receipt) => receipt,
        other => panic!("the owner-issued plan performs retirement: {other:?}"),
    };
    let after_retirement = world
        .basis
        .owner_service_cost_snapshot()
        .expect("retirement leaves the unrelated owner service open");
    assert_eq!(receipt.retired_branch().id, child_id);
    assert_eq!(
        after_retirement.target_cell_contacts() - after_plan.target_cell_contacts(),
        1,
        "execution rechecks the target cell once"
    );
    assert_eq!(
        after_retirement.canonical_movements() - after_plan.canonical_movements(),
        1,
        "execution performs one canonical retirement movement"
    );
    assert_eq!(
        after_retirement.retention_registry_contacts() - after_plan.retention_registry_contacts(),
        1,
        "execution performs one canonical retention recheck"
    );
}

#[test]
fn closed_basis_port_reports_owner_unavailable() {
    let mut world = AdversarialWorld::new();
    world.close_root();
    assert!(
        world.basis.owner_service_cost_snapshot().is_err(),
        "a weak basis port cannot inspect a closed owner"
    );
}

#[test]
fn closed_lifecycle_port_reports_owner_unavailable() {
    let mut world = AdversarialWorld::new();
    world.close_root();
    assert!(
        world.lifecycle.owner_service_cost_snapshot().is_err(),
        "a weak lifecycle port cannot inspect a closed owner"
    );
}

#[test]
fn fork_and_exact_retention_report_their_owned_work_once() {
    let world = AdversarialWorld::new();
    let before = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner is open");
    let fork = world
        .mutation
        .fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("cost-child")
                .expect("the fork identity is valid"),
            &world.root_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the populated source forks through the canonical owner");
    let after_fork = world
        .basis
        .owner_service_cost_snapshot()
        .expect("the owner remains open after fork");
    assert_eq!(
        after_fork.fork_source_captures() - before.fork_source_captures(),
        1
    );
    assert_eq!(
        after_fork.fork_destination_preparations() - before.fork_destination_preparations(),
        1
    );
    assert_eq!(
        after_fork.fork_destination_installations() - before.fork_destination_installations(),
        1
    );
    assert_eq!(
        after_fork.forked_mutable_graph_nodes_copied() - before.forked_mutable_graph_nodes_copied(),
        0,
        "forking shares populated immutable state instead of copying graph nodes"
    );

    let basis = fork.created_basis().clone();
    let lease = world
        .basis
        .retain_exact(&basis)
        .expect("the exact fork basis can be retained");
    let after_retain = world
        .basis
        .owner_service_cost_snapshot()
        .expect("retention leaves the owner open");
    assert_eq!(
        after_retain.retention_registry_contacts() - after_fork.retention_registry_contacts(),
        1
    );
    assert!(matches!(
        world.basis.release_exact(lease),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
    let after_release = world
        .basis
        .owner_service_cost_snapshot()
        .expect("release leaves the owner open");
    assert_eq!(
        after_release.retention_registry_contacts(),
        after_retain.retention_registry_contacts(),
        "the owner-issued lease terminates directly without a second registry admission"
    );
}

#[test]
fn target_work_cost_is_independent_of_unrelated_live_branch_count() {
    let small = AdversarialWorld::new();
    let small_before = small
        .basis
        .owner_service_cost_snapshot()
        .expect("the small owner is open");
    small
        .mutation
        .advance_exact(
            &small.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the small target advances");
    let small_after = small
        .basis
        .owner_service_cost_snapshot()
        .expect("the small owner remains open");

    let large = AdversarialWorld::new();
    let mut unrelated = Vec::new();
    for ordinal in 0..8 {
        unrelated.push(
            large
                .mutation
                .fork_exact(
                    worth_signal::facade::branch::validate_signal_branch_name(format!(
                        "unrelated-{ordinal}"
                    ))
                    .expect("the unrelated identity is valid"),
                    &large.root_basis,
                    &SignalOwnerCancellationSource::new().token(),
                )
                .expect("the owner admits a small larger court")
                .into_parts()
                .1,
        );
    }
    let large_before = large
        .basis
        .owner_service_cost_snapshot()
        .expect("the larger owner is open");
    large
        .mutation
        .advance_exact(
            &large.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the same target advances in the larger court");
    let large_after = large
        .basis
        .owner_service_cost_snapshot()
        .expect("the larger owner remains open");

    assert_eq!(
        small_after.branch_registry_lookups() - small_before.branch_registry_lookups(),
        large_after.branch_registry_lookups() - large_before.branch_registry_lookups()
    );
    assert_eq!(
        small_after.target_cell_contacts() - small_before.target_cell_contacts(),
        large_after.target_cell_contacts() - large_before.target_cell_contacts()
    );
    assert_eq!(
        small_after.branch_registry_entries_scanned()
            - small_before.branch_registry_entries_scanned(),
        0
    );
    assert_eq!(
        large_after.branch_registry_entries_scanned()
            - large_before.branch_registry_entries_scanned(),
        0
    );
    drop(unrelated);
}

#[cfg(feature = "test-operation-control")]
#[test]
fn unarmed_operation_control_is_cost_neutral() {
    let controlled = AdversarialWorld::new();
    let ordinary = AdversarialWorld::new();
    controlled
        .runtime
        .as_ref()
        .expect("the controlled root remains live")
        .owner_operation_control()
        .expect("obtaining the unarmed control handle succeeds");
    let controlled_before = controlled
        .basis
        .owner_service_cost_snapshot()
        .expect("the controlled owner is open");
    let ordinary_before = ordinary
        .basis
        .owner_service_cost_snapshot()
        .expect("the ordinary owner is open");
    controlled
        .mutation
        .advance_exact(
            &controlled.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the controlled operation succeeds");
    ordinary
        .mutation
        .advance_exact(
            &ordinary.child_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the ordinary operation succeeds");
    let controlled_after = controlled
        .basis
        .owner_service_cost_snapshot()
        .expect("the controlled owner remains open");
    let ordinary_after = ordinary
        .basis
        .owner_service_cost_snapshot()
        .expect("the ordinary owner remains open");
    assert_eq!(
        controlled_after.canonical_movements() - controlled_before.canonical_movements(),
        ordinary_after.canonical_movements() - ordinary_before.canonical_movements()
    );
    assert_eq!(
        controlled_after.target_cell_contacts() - controlled_before.target_cell_contacts(),
        ordinary_after.target_cell_contacts() - ordinary_before.target_cell_contacts()
    );
    assert_eq!(
        controlled_after.branch_registry_entries_scanned()
            - controlled_before.branch_registry_entries_scanned(),
        ordinary_after.branch_registry_entries_scanned()
            - ordinary_before.branch_registry_entries_scanned()
    );
}
