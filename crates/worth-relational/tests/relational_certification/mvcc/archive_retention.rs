use super::world::supply_chain::{certified_supply_chain_world, SupplyChainScale};
use crate::mvcc_branch_fork_fixture::fork_branch;
use worth_relational::facade::branch::{
    RelationalBranchBasisDenial, RelationalBranchLifecyclePosture, RelationalForkDenial,
};
use worth_relational::facade::inspection::RelationalMvccCostScope;
use worth_relational::facade::mvcc::{
    RelationalBranchTransactionAdmissionDenial, RelationalPublicationDenial,
    RelationalPublicationOutcome, RelationalTransactionIntent,
};
use worth_relational::facade::runtime::ProjectionAspectScope;

#[test]
fn archive_preserves_existing_reads_and_closes_every_new_mutation_door() {
    let (mut world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let identity = fork_branch(&world.runtime, "storm");
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let immutable_observation = basis.observation();
    let retained_commit = immutable_observation.canonical_commit().cloned();
    let retained_index = immutable_observation.correctness_index();
    let retained_root = immutable_observation.selected_root_identity();
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&immutable_observation)
        .unwrap();
    let voyage_id = world.handles.aurora_voyage().id;
    assert!(snapshot_contains_entity(
        &world.runtime,
        &snapshot,
        voyage_id
    ));
    let external = world.runtime.retain_component_basis(&basis).unwrap();
    let active = world
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let candidate_transaction = world
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(
            candidate_transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let cost_scope = RelationalMvccCostScope::capture(&world.runtime, vec![identity.clone()]);

    world.runtime.archive_branch(&identity).unwrap();
    assert_eq!(basis.observation().selected_root_identity(), retained_root);
    assert_eq!(
        immutable_observation.canonical_commit(),
        retained_commit.as_ref()
    );
    assert_eq!(immutable_observation.correctness_index(), retained_index);
    assert!(snapshot_contains_entity(
        &world.runtime,
        &snapshot,
        voyage_id
    ));
    assert_eq!(
        world
            .runtime
            .branch_reference_state(identity.branch_id())
            .unwrap()
            .lifecycle_posture(),
        RelationalBranchLifecyclePosture::Archived,
    );
    assert!(matches!(
        world.runtime.observe_branch(&identity),
        Err(RelationalBranchBasisDenial::ArchivedBranch(_))
    ));
    assert!(matches!(
        world.runtime.observe_fork_source(identity.branch_id()),
        Err(RelationalForkDenial::SourceArchived)
    ));
    assert!(matches!(
        world
            .runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary()),
        Err(RelationalBranchTransactionAdmissionDenial::Archived)
    ));
    assert!(matches!(
        world
            .runtime
            .publication_port()
            .compare_and_publish(candidate),
        RelationalPublicationOutcome::Denied(RelationalPublicationDenial::Archived)
    ));
    drop(active);
    world.runtime.release_component_basis(external).unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();

    let cost = world.runtime.observe_mvcc_counters(&cost_scope).unwrap();
    assert_eq!(cost.retention_cost_delta().candidate_releases, 1);
    assert_eq!(cost.retention_cost_delta().transaction_releases, 1);
    assert_eq!(cost.retention_cost_delta().external_pin_releases, 1);
}

fn snapshot_contains_entity(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: worth_relational::facade::identity::EntityId,
) -> bool {
    runtime
        .read_truth()
        .project_snapshot(snapshot)
        .unwrap()
        .entity_record_with_projection_scope(entity, ProjectionAspectScope::empty(), |_| Some(()))
        .is_some()
}
