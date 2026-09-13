use super::super::super::SignalOwnerCancellationSource;
use super::world::{set_dependency, MutationWorld};
use crate::branch::validate_signal_branch_name;

#[test]
fn four_method_port_matrix_moves_populated_owner_state_with_exact_cost() {
    let world = MutationWorld::<()>::new();
    let before = world.owner.cost_snapshot();
    let cancellation = SignalOwnerCancellationSource::new();

    let fork = world
        .port
        .fork_exact(
            validate_signal_branch_name("mutation-port-child")
                .expect("the child name is validated"),
            &world.source_basis,
            &cancellation.token(),
        )
        .expect("fork_exact installs the real owner child");
    assert_eq!(
        fork.created_branch().parent_branch_id,
        Some(world.source_branch.id)
    );
    assert_eq!(fork.created_branch().id, fork.created_basis().branch_id());

    let mut runtime_ctx = ();
    let advanced = world
        .port
        .advance_exact(
            &world.source_basis,
            &mut runtime_ctx,
            &cancellation.token(),
            |transaction| set_dependency(transaction, world.derived, world.input_b),
        )
        .expect("advance_exact changes populated semantic state");
    assert_eq!(
        advanced.advanced_basis().observation().generation().get(),
        1
    );

    let captured = world
        .port
        .capture_exact(advanced.advanced_basis(), &cancellation.token())
        .expect("capture_exact stores a real owner snapshot");
    assert_eq!(
        captured.admitted_snapshot().snapshot().meta.branch_id,
        world.source_branch.id
    );

    let intervening = world
        .port
        .advance_exact(
            captured.captured_basis(),
            &mut runtime_ctx,
            &cancellation.token(),
            |transaction| set_dependency(transaction, world.derived, world.input_a),
        )
        .expect("an intervening mutation changes canonical truth");
    let restored = world
        .port
        .restore_exact(
            intervening.advanced_basis(),
            captured.admitted_snapshot(),
            &cancellation.token(),
        )
        .expect("restore_exact reinstalls the captured semantic state");

    let after = world.owner.cost_snapshot();
    assert_eq!(
        after.owner_upgrade_attempts(),
        before.owner_upgrade_attempts() + 5
    );
    assert_eq!(
        after.branch_registry_lookups(),
        before.branch_registry_lookups() + 5
    );
    assert_eq!(
        after.branch_registry_reservations(),
        before.branch_registry_reservations() + 1
    );
    assert_eq!(
        after.target_cell_contacts(),
        before.target_cell_contacts() + 5
    );
    assert_eq!(after.target_cell_waits(), before.target_cell_waits());
    assert_eq!(
        after.canonical_movements(),
        before.canonical_movements() + 4
    );
    assert_eq!(
        after.retention_registry_contacts(),
        before.retention_registry_contacts() + 5
    );
    assert_eq!(
        after.fork_source_captures(),
        before.fork_source_captures() + 1
    );
    assert_eq!(
        after.fork_destination_installations(),
        before.fork_destination_installations() + 1
    );
    assert_eq!(
        after.forked_mutable_graph_nodes_copied(),
        before.forked_mutable_graph_nodes_copied()
    );
    assert_eq!(after.branch_registry_entries_scanned(), 0);
    assert_eq!(restored.observation().generation().get(), 4);
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        vec![world.input_b],
        "independent state observation sees the captured value after restore"
    );
}

#[test]
fn fork_exact_returns_the_installed_owner_handle_without_reconstruction() {
    let world = MutationWorld::<()>::new();
    let fork = world
        .port
        .fork_exact(
            validate_signal_branch_name("exact-custody-child").expect("the name validates"),
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("fork returns owner-issued destination parts");
    let admission = world.owner.admit().expect("fork proof admits");
    let installed_handle = world
        .owner
        .lookup_cell(&admission, fork.created_branch().id)
        .expect("fork destination is installed")
        .with_state(&admission, |state, _| state.handle().clone())
        .expect("fork destination exposes its canonical handle");

    assert_eq!(fork.created_branch(), &installed_handle);
    assert_eq!(fork.created_basis().owner_branch_id(), installed_handle.id);
}

#[test]
fn fork_exact_preserves_structural_sharing_and_isolates_first_write() {
    let world = MutationWorld::<()>::new();
    let fork = world
        .port
        .fork_exact(
            validate_signal_branch_name("sharing-child").expect("the child name validates"),
            &world.source_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("the real port forks populated state");
    let child = fork.created_branch().clone();
    let admission = world.owner.admit().expect("sharing observation admits");
    let source_cell = world
        .owner
        .lookup_cell(&admission, world.source_branch.id)
        .expect("source cell remains installed");
    let child_cell = world
        .owner
        .lookup_cell(&admission, child.id)
        .expect("child cell is installed");
    let source_identity = source_cell
        .with_state(&admission, |state, _| state.state().persistent_identity())
        .expect("source identity observes");
    let child_identity = child_cell
        .with_state(&admission, |state, _| state.state().persistent_identity())
        .expect("child identity observes");
    let sharing = source_identity.sharing_with(&child_identity);
    assert!(sharing.graph.arena_root_shared);
    assert!(sharing.graph.topology_root_shared);
    assert!(sharing.graph.cause_root_shared);
    assert!(sharing.config_roots_shared);
    assert!(sharing.derived_roots_shared);

    let mut runtime_ctx = ();
    world
        .port
        .advance_exact(
            fork.created_basis(),
            &mut runtime_ctx,
            &SignalOwnerCancellationSource::new().token(),
            |transaction| set_dependency(transaction, world.derived, world.input_b),
        )
        .expect("the fork child performs a semantic first write");
    assert_eq!(world.dependency_sources(&child), vec![world.input_b]);
    assert_eq!(
        world.dependency_sources(&world.source_branch),
        vec![world.input_a],
        "child mutation cannot alter source truth"
    );
}
