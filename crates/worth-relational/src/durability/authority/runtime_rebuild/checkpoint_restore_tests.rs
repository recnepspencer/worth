use super::partition_images;
use crate::identity::data::PartitionId;
use crate::identity::data::{EntityId, KindId, VersionId};

fn shares_first_generation(
    left: &partition_images::RestoredPartitions,
    right: &partition_images::RestoredPartitions,
) -> bool {
    shares_partition_generation(left, right, PartitionId::main())
}

fn shares_partition_generation(
    left: &partition_images::RestoredPartitions,
    right: &partition_images::RestoredPartitions,
    partition_id: PartitionId,
) -> bool {
    let left = left.get(&partition_id).unwrap();
    let right = right.get(&partition_id).unwrap();
    std::ptr::eq(
        left.entity_arena.generations.get(0).unwrap(),
        right.entity_arena.generations.get(0).unwrap(),
    )
}

#[test]
fn equal_exact_root_reuses_the_readmitted_partition_substrate() {
    let request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let lane = crate::storage::substrate::maintenance::ReconstructionLane::oracle();
    let fixture = readmitted_mirror_fixture(
        &lane,
        execution,
        &[partition_with_entity(PartitionId::main())],
    );
    assert_eq!(fixture.roots.len(), 1);
    assert_eq!(fixture.roots[0].1, fixture.mirror_images);
    let mirror = fixture.restore_mirror(&lane, execution).unwrap();
    let root = fixture
        .restored
        .partitions
        .get(&fixture.roots[0].0)
        .unwrap();
    assert!(shares_first_generation(&mirror, root));
}

#[test]
fn divergent_sibling_is_not_reused_as_the_storage_mirror() {
    use crate::history::data::CommitId;
    use crate::storage::substrate::SlotInit;

    let request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let lane = crate::storage::substrate::maintenance::ReconstructionLane::oracle();
    let main_id = CommitId(1);
    let sibling_id = CommitId(2);
    let mut main = partition_with_entity(PartitionId::main());
    let mut sibling = main.clone();
    for (partition, version) in [(&mut main, VersionId(3)), (&mut sibling, VersionId(2))] {
        partition.entity_arena.push_slot(SlotInit {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            version_id: version,
            extra: Default::default(),
        });
    }
    let mut fixture = readmitted_mirror_fixture(&lane, execution, &[main]);
    fixture.readmit_root(&lane, execution, sibling_id, &[sibling]);
    assert_eq!(fixture.roots.len(), 2);
    let mirror = fixture.restore_mirror(&lane, execution).unwrap();
    let main_root = fixture.restored.partitions.get(&main_id).unwrap();
    let sibling_root = fixture.restored.partitions.get(&sibling_id).unwrap();
    assert!(shares_first_generation(&mirror, main_root));
    assert!(!shares_first_generation(&mirror, sibling_root));
}

#[test]
fn divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse() {
    use crate::storage::substrate::{RelationEndpoints, RelationExtra, SlotInit};

    let request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let lane = crate::storage::substrate::maintenance::ReconstructionLane::oracle();
    let mut main = partition_with_entity(PartitionId::main());
    let mut secondary = partition_with_entity(PartitionId(41));
    let mut relation = empty_partition(PartitionId(31));
    let kind_id = KindId(2);
    let (slot, generation, _) = relation.relation_arena.push_slot(SlotInit {
        partition_id: relation.partition_id,
        kind_id,
        version_id: VersionId(1),
        extra: RelationExtra::from_unbudgeted_parts(
            Some(RelationEndpoints {
                source: EntityId::new(main.partition_id, 0, 1),
                target: EntityId::new(secondary.partition_id, 0, 1),
            }),
            None,
        ),
    });
    let relation_id =
        crate::identity::data::RelationId::new(relation.partition_id, slot as u64, generation);
    main.adjacency
        .ensure(0, &main.adjacency_policy)
        .insert(kind_id, relation_id);
    secondary
        .reverse_adjacency
        .ensure(0, &secondary.adjacency_policy)
        .insert(kind_id, relation_id);
    let mut fixture = readmitted_mirror_fixture(&lane, execution, &[main, secondary, relation]);
    let main_image = fixture
        .mirror_images
        .iter_mut()
        .find(|image| image.partition_id == PartitionId::main())
        .unwrap();
    main_image.entity_arena.snapshot_pins[0] += 1;

    let mirror = fixture.restore_mirror(&lane, execution).unwrap();
    let root_commit = fixture.roots[0].0;
    let root = fixture.restored.partitions.get(&root_commit).unwrap();
    assert!(!shares_partition_generation(
        &mirror,
        root,
        PartitionId::main()
    ));
    assert!(shares_partition_generation(&mirror, root, PartitionId(41)));
    let mirror_relation = mirror.get(&PartitionId(31)).unwrap();
    let root_relation = root.get(&PartitionId(31)).unwrap();
    assert!(std::ptr::eq(
        mirror_relation.relation_arena.generations.get(0).unwrap(),
        root_relation.relation_arena.generations.get(0).unwrap(),
    ));
}

#[test]
fn equal_image_cannot_bypass_global_contract_readmission() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let lane = crate::storage::substrate::maintenance::ReconstructionLane::oracle();
    let partition = crate::durability::native_region_fixture::populated_partition();
    let fixture = partition_fixture::readmitted_mirror_fixture_with_schema(
        &lane,
        execution,
        &[partition.partition],
        partition.schema,
    );
    assert_eq!(fixture.roots[0].1, fixture.mirror_images);
    let empty = crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog::from_contracts(&[]).unwrap();
    let mirror = fixture.restore_mirror_with_contracts(&lane, execution, &empty);
    let error = mirror.unwrap_err();
    let crate::durability::data::RelationalRecoveryStop::Durability(error) = error else {
        panic!("contract corruption must remain a durability refusal: {error:?}");
    };
    assert_eq!(
        error.class,
        crate::durability::data::RecoveryFailureClass::CorruptCheckpoint
    );
}

#[path = "checkpoint_restore_tests/partition_fixture.rs"]
mod partition_fixture;
use partition_fixture::{empty_partition, partition_with_entity, readmitted_mirror_fixture};

type CheckpointTestRequest<'a, 'b> = worth_execution::ExecutionRequest<'a, 'b>;
