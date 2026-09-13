use std::collections::BTreeSet;

use super::world::{commit_through_services, empty_runtime, fork_through_services};
use worth_relational::facade::branch::{
    RelationalBranchBasisDenial, RelationalBranchDeletionOutcome,
    RelationalOwnerLifecycleObservation,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::RelationalTransactionIntent;

#[test]
fn healthy_bundle_executes_every_promised_method_family() {
    let runtime = empty_runtime();
    let services = runtime.owner_component_services();
    assert_eq!(
        services.lifecycle_port().owner_lifecycle_observation(),
        RelationalOwnerLifecycleObservation::Open
    );

    let committed = commit_through_services(&runtime, &services, "six-port-health");
    assert_eq!(committed.commit.branch_id, BranchId("main".to_owned()));
    let archived_identity = fork_through_services(&services, "archived")
        .target_identity()
        .clone();
    let deleted_identity = fork_through_services(&services, "deleted")
        .target_identity()
        .clone();

    let basis = services.basis_port();
    let (descriptor, admitted) = basis
        .observe_branch(&archived_identity)
        .expect("basis service observes the fork");
    assert_eq!(
        basis
            .observe_branch_with_control(
                &archived_identity,
                &worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted(),
            )
            .expect("controlled observation reaches the same canonical method")
            .0,
        descriptor
    );
    assert_eq!(
        basis
            .admit_branch_basis(&archived_identity)
            .expect("identity admission succeeds")
            .descriptor(),
        &descriptor
    );
    assert_eq!(
        basis
            .readmit_branch_basis(&descriptor)
            .expect("descriptor readmission consults live owner truth")
            .descriptor(),
        &descriptor
    );
    let lease = basis
        .retain_component_basis(&admitted)
        .expect("owner retains the admitted exact basis");
    assert_eq!(
        basis
            .readmit_retained_branch_basis(&descriptor, &lease)
            .expect("live exact lease readmits its basis")
            .descriptor(),
        &descriptor
    );
    basis
        .release_component_basis(lease)
        .expect("the same owner explicitly releases the lease");

    services
        .lifecycle_port()
        .archive_branch(&archived_identity)
        .expect("lifecycle service archives the exact branch");
    assert!(matches!(
        runtime.observe_branch(&archived_identity),
        Err(RelationalBranchBasisDenial::ArchivedBranch(branch))
            if branch == BranchId("archived".to_owned())
    ));
    assert!(matches!(
        services
            .lifecycle_port()
            .delete_branch(&deleted_identity)
            .expect("lifecycle service deletes an unretained branch"),
        RelationalBranchDeletionOutcome::Deleted(deleted)
            if deleted.identity() == &deleted_identity
    ));
    assert!(runtime
        .branch_identity(&BranchId("deleted".to_owned()))
        .is_err());
}

#[test]
fn discarded_candidate_leaves_truth_unchanged_and_releases_its_obligation() {
    use super::super::invariant_oracle_expectations::expected_supply_chain_branch;
    use super::super::world::supply_chain::{
        certified_supply_chain_world, compare, lower_supply_chain_production_delta,
        observe_supply_chain_observation, BranchLabel, DeltaId, SupplyChainScale,
    };

    let (world, _baseline) = certified_supply_chain_world(SupplyChainScale::court());
    let services = world.runtime.owner_component_services();
    let (_, source) = services
        .fork_port()
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("the non-empty Supply Chain main branch is a fork source");
    let identity = services
        .fork_port()
        .fork_branch(BranchId("storm".to_owned()), source)
        .expect("the fork port installs the exact storm branch")
        .target_identity()
        .clone();
    let (descriptor, basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the basis port observes the storm predecessor");
    let batch = lower_supply_chain_production_delta(
        &world.runtime,
        &world.program,
        &world.handles,
        identity.branch_id(),
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .expect("the real storm branch admits its semantic reroute delta");
    let mut transaction = world
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("the exact storm predecessor opens a transaction");
    transaction
        .push_batch(batch)
        .expect("the semantic reroute remains inside its declared budget");
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(transaction)
        .expect("the preparation port installs the canonical candidate");
    let discarded = services
        .preparation_port()
        .discard_prepared_candidate(candidate)
        .expect("the preparation port consumes the candidate without movement");
    assert_eq!(discarded.branch(), identity.branch_id());
    assert_eq!(
        discarded.released_record_reservation_count(),
        0,
        "the update-only Supply Chain delta must not invent record reservations"
    );

    let (after_descriptor, after_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("discard leaves the predecessor observable");
    assert_eq!(after_descriptor, descriptor);
    let observation = after_basis.observation();
    let handles = world.handles.for_observation(&observation);
    let observed =
        observe_supply_chain_observation(&world.program, &handles, &world.runtime, &observation)
            .expect("discarded work leaves neutral Supply Chain truth observable");
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Storm, None),
        &observed,
    )
    .expect("discarded work leaves the independent baseline unchanged");
    assert!(matches!(
        services.lifecycle_port().delete_branch(&identity),
        Ok(RelationalBranchDeletionOutcome::Deleted(ref deleted))
            if deleted.identity() == &identity
    ));
}
