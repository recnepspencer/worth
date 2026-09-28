use std::sync::Arc;

use worth_runtime_bridge::facade::{RelationalBridgeRecordIdentityParts, TruthBranchIdentity};

use crate::facade::identity::PartitionId;
use crate::presentation::bridge::relational_test_support::{changed_entities, create_entity_in_partition, create_entity_outcome};

use super::super::RuntimeBridgeRelationalSource;
use super::support::runtime_with_test_schema;

#[test]
fn retained_entity_projection_denies_mixed_branch_and_snapshot_axes() {
    let runtime = runtime_with_test_schema();
    let committed = create_entity_outcome(&runtime, "retained-projection");
    let entity = changed_entities(&committed)[0];
    let main_identity = runtime.main_branch_identity();
    let source = RuntimeBridgeRelationalSource::for_graph_role(Arc::new(runtime), "model")
        .expect("test graph role");
    let (_, basis) = source
        .observe_branch_basis(&main_identity)
        .expect("owner-admitted main basis");
    let lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("retained projection observation");
    let record = RelationalBridgeRecordIdentityParts::entity(
        entity.partition_id.0,
        entity.local_slot.0,
        entity.generation.0,
    );

    assert!(source
        .read_retained_entity_aspect_state(
            lease.snapshot_identity(),
            &TruthBranchIdentity::from_relational_branch_id("main"),
            record,
        )
        .expect("matching retained axes must project")
        .is_some());
    let denial = source
        .read_retained_entity_aspect_state(
            lease.snapshot_identity(),
            &TruthBranchIdentity::from_relational_branch_id("feature"),
            record,
        )
        .expect_err("the same snapshot presented as another branch must deny");
    assert!(denial.to_string().contains("different truth branch"));
}

#[test]
fn retained_entity_projection_enforces_partition_authority_before_projection() {
    let runtime = runtime_with_test_schema();
    let admitted = create_entity_in_partition(&runtime, "partition-main", PartitionId::main());
    let foreign = create_entity_in_partition(&runtime, "partition-foreign", PartitionId::new(7));
    let main_identity = runtime.main_branch_identity();
    let source = RuntimeBridgeRelationalSource::for_graph_partition(
        Arc::new(runtime),
        "model",
        PartitionId::main(),
        worth_foundational::facade::TruthPartitionRole::new("model-main")
            .expect("truth partition role"),
    )
    .expect("partition-bound source");
    let (_, basis) = source
        .observe_branch_basis(&main_identity)
        .expect("owner-admitted partition basis");
    let lease = source
        .retain_branch_basis_for_bridge(&basis)
        .expect("retained partition observation");
    let branch = TruthBranchIdentity::from_relational_branch_id("main");

    assert!(source
        .read_retained_entity_aspect_state(
            lease.snapshot_identity(),
            &branch,
            bridge_entity_identity(admitted),
        )
        .expect("admitted partition must project")
        .is_some());
    let denial = source
        .read_retained_entity_aspect_state(
            lease.snapshot_identity(),
            &branch,
            bridge_entity_identity(foreign),
        )
        .expect_err("foreign partition must deny before projection");
    assert!(denial.to_string().contains("partition authority"));
}

fn bridge_entity_identity(
    entity: crate::facade::identity::EntityId,
) -> RelationalBridgeRecordIdentityParts {
    RelationalBridgeRecordIdentityParts::entity(
        entity.partition_id.0,
        entity.local_slot.0,
        entity.generation.0,
    )
}
