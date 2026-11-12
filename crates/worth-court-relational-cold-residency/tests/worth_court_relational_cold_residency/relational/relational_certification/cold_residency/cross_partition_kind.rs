//! An unrelated owning-partition write preserves cold endpoint adjacency meaning.
use super::{
    current_publications::{changed_relations, create_relation_outcome},
    current_source::capture_and_reopen,
    region_publications::create_entity,
};
use worth_relational::facade::{durability::*, identity::*};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};
#[test]
fn a_cold_point_read_keeps_cross_partition_kind_authority_after_an_unrelated_write() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    let source_entity = create_entity(request, &producer, "cross-source", PartitionId(2));
    let target_entity = create_entity(request, &producer, "cross-target", PartitionId(3));
    let relation = create_relation_outcome(
        request,
        &producer,
        source_entity,
        target_entity,
        "cross-edge",
    );
    let expected_relations = changed_relations(&relation);
    release_test_commit_snapshot(&producer, &relation);
    let (runtime, source, _, _) = capture_and_reopen(request, producer, medium, source_entity);
    let current_members = source.members.keys().copied().collect::<Vec<_>>();
    create_entity(request, &runtime, "unrelated-record", PartitionId::main());
    runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &current_members,
                source: None,
            },
        )
        .unwrap();
    let mut reader = begin_main_transaction(&runtime);
    assert!(reader
        .read_entity(request, source_entity)
        .unwrap()
        .effective()
        .is_some());
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let view = runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    let actual = view
        .outgoing_relations_of_kind(request, source_entity, KindId(2))
        .unwrap()
        .iter()
        .map(|r| r.relation_id)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected_relations);
}
