use super::world::supply_chain::{certified_supply_chain_world, SupplyChainScale};
use crate::mvcc_branch_fork_fixture::fork_branch;
use worth_relational::facade::branch::{
    RelationalBranchBasisDenial, RelationalBranchRetentionTerminalOutcome,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::RelationalMvccCostScope;
use worth_relational::facade::mvcc::{
    RelationalBranchTransactionAdmissionDenial, RelationalPublicationDenial,
    RelationalPublicationOutcome, RelationalTransactionIntent,
};

#[test]
fn deletion_closes_admission_then_waits_for_transaction_and_candidate_operations() {
    let (mut world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let transaction_identity = fork_branch(&world.runtime, "deleting-transaction");
    let (_, transaction_basis) = world.runtime.observe_branch(&transaction_identity).unwrap();
    let transaction = world
        .runtime
        .begin_branch_transaction(&transaction_basis, RelationalTransactionIntent::ordinary())
        .unwrap();

    let waiting = world.runtime.delete_branch(&transaction_identity).unwrap();
    assert_eq!(waiting.waiting().unwrap().active_operation_count(), 1);
    assert!(matches!(
        world
            .runtime
            .begin_branch_transaction(&transaction_basis, RelationalTransactionIntent::ordinary(),),
        Err(RelationalBranchTransactionAdmissionDenial::Deleting)
    ));
    drop(transaction);
    assert!(world
        .runtime
        .delete_branch(&transaction_identity)
        .unwrap()
        .deleted()
        .is_some());

    let candidate_identity = fork_branch(&world.runtime, "deleting-candidate");
    let (_, candidate_basis) = world.runtime.observe_branch(&candidate_identity).unwrap();
    let candidate_transaction = world
        .runtime
        .begin_branch_transaction(&candidate_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(
            candidate_transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert_eq!(
        world
            .runtime
            .delete_branch(&candidate_identity)
            .unwrap()
            .waiting()
            .unwrap()
            .active_operation_count(),
        1
    );
    assert!(matches!(
        world
            .runtime
            .publication_port()
            .compare_and_publish(candidate),
        RelationalPublicationOutcome::Denied(RelationalPublicationDenial::Deleting)
    ));
    assert!(world
        .runtime
        .delete_branch(&candidate_identity)
        .unwrap()
        .deleted()
        .is_some());
}

#[test]
fn performed_publication_retains_the_branch_operation_until_settlement() {
    let (mut world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = fork_branch(&world.runtime, "settlement-retention");
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let transaction = world
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let movement_scope = RelationalMvccCostScope::capture(&world.runtime, vec![identity.clone()]);
    let performed = match world
        .runtime
        .publication_port()
        .compare_and_publish(candidate)
    {
        RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("prepared branch movement must perform: {outcome:?}"),
    };
    let movement_cost = world
        .runtime
        .observe_mvcc_counters(&movement_scope)
        .unwrap();
    assert_eq!(movement_cost.retention_cost_delta().candidate_releases, 1);
    assert_eq!(
        movement_cost
            .retention_cost_delta()
            .performed_settlement_acquires,
        1
    );
    let settlement_scope = RelationalMvccCostScope::capture(&world.runtime, vec![identity.clone()]);

    assert_eq!(
        world
            .runtime
            .delete_branch(&identity)
            .unwrap()
            .waiting()
            .unwrap()
            .active_operation_count(),
        1
    );
    let committed = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    let settlement_cost = world
        .runtime
        .observe_mvcc_counters(&settlement_scope)
        .unwrap();
    assert_eq!(
        settlement_cost
            .retention_cost_delta()
            .performed_settlement_releases,
        1
    );
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    assert!(world
        .runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .is_some());
    assert!(world.runtime.branch_identity(identity.branch_id()).is_err());
}

#[test]
fn deleted_shared_root_survives_while_the_main_head_still_owns_it() {
    let (mut world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = fork_branch(&world.runtime, "maintenance");
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let external = world.runtime.retain_component_basis(&basis).unwrap();
    let deleted = world
        .runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .expect("branch without active operations deletes immediately")
        .clone();

    assert!(world.runtime.branch_identity(identity.branch_id()).is_err());
    assert_eq!(
        basis.observation().selected_root_identity(),
        deleted.retired_root_identity()
    );
    let retained = world.runtime.run_branch_root_reclamation_pass();
    assert!(retained.roots_still_retained() >= 1);

    drop(basis);
    world.runtime.release_component_basis(external).unwrap();
    let still_shared = world.runtime.run_branch_root_reclamation_pass();
    assert!(still_shared.roots_still_retained() >= 1);
    let (_, source) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    assert!(world
        .runtime
        .fork_branch(BranchId("maintenance".to_owned()), source)
        .is_err());
}

#[test]
fn maintenance_reclaims_a_deleted_unique_root_only_after_last_pin_release() {
    let (mut world, _) = certified_supply_chain_world(SupplyChainScale::court());
    for _ in 0..4 {
        world.runtime.run_branch_root_reclamation_pass();
    }
    let identity = fork_branch(&world.runtime, "unique-maintenance");
    let (_, initial_basis) = world.runtime.observe_branch(&identity).unwrap();
    let transaction = world
        .runtime
        .begin_branch_transaction(&initial_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let committed = transaction
        .commit(
            &world.runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(initial_basis);

    let (_, current_basis) = world.runtime.observe_branch(&identity).unwrap();
    let external = world
        .runtime
        .retain_component_basis(&current_basis)
        .unwrap();
    let deleted = world
        .runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .expect("branch without active operations deletes immediately")
        .clone();
    assert_eq!(
        deleted.retired_root_identity(),
        current_basis.observation().selected_root_identity()
    );
    let pinned = world.runtime.run_branch_root_reclamation_pass();
    assert!(pinned.roots_still_retained() >= 1);

    drop(current_basis);
    world.runtime.release_component_basis(external).unwrap();
    assert!(world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .is_ok());
    let mut reclaimed = 0;
    let mut unique_bytes = 0;
    for _ in 0..4 {
        let pass = world.runtime.run_branch_root_reclamation_pass();
        reclaimed += pass.roots_reclaimed();
        unique_bytes += pass.unique_authoritative_bytes_reclaimed();
    }
    assert!(reclaimed >= 1);
    assert!(unique_bytes > 0);
}

#[test]
fn exact_snapshot_carries_its_observation_obligation_until_release() {
    let (mut world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = fork_branch(&world.runtime, "snapshot-obligation");
    let (_, initial_basis) = world.runtime.observe_branch(&identity).unwrap();
    let transaction = world
        .runtime
        .begin_branch_transaction(&initial_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let committed = transaction
        .commit(
            &world.runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(initial_basis);
    assert!(world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .is_ok());
    for _ in 0..4 {
        world.runtime.run_branch_root_reclamation_pass();
    }

    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    drop(basis);
    let deleted = world
        .runtime
        .delete_branch(&identity)
        .unwrap()
        .deleted()
        .expect("a snapshot observation is not a mutable branch operation")
        .clone();

    let retained = world.runtime.run_branch_root_reclamation_pass();
    assert!(retained.roots_still_retained() >= 1);
    assert_eq!(
        world
            .runtime
            .read_truth()
            .inspect_snapshot(&snapshot)
            .unwrap()
            .root_id,
        Some(deleted.retired_root_identity())
    );
    assert!(world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .is_ok());
    let mut reclaimed = 0;
    for _ in 0..4 {
        reclaimed += world
            .runtime
            .run_branch_root_reclamation_pass()
            .roots_reclaimed();
    }
    assert!(reclaimed >= 1);
}

#[test]
fn external_release_distinguishes_foreign_owner_and_owner_loss() {
    let (first, _) = certified_supply_chain_world(SupplyChainScale::court());
    let (second, _) = certified_supply_chain_world(SupplyChainScale::court());
    let (_, basis) = first
        .runtime
        .observe_branch(&first.runtime.main_branch_identity())
        .unwrap();
    let lease = first.runtime.retain_component_basis(&basis).unwrap();

    let foreign = second.runtime.release_component_basis(lease).unwrap_err();
    assert!(matches!(
        foreign.denial(),
        RelationalBranchBasisDenial::ForeignRuntime { .. }
    ));
    let lease = foreign.into_lease();
    drop(basis);
    drop(first);

    assert_eq!(
        lease.release().outcome(),
        RelationalBranchRetentionTerminalOutcome::OwnerUnavailable
    );
}
