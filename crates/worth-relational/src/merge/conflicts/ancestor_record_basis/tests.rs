use super::*;

use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::merge::data::VisibleMergeRecordKind;
use crate::schema::data::{KindResolution, SchemaId, SchemaVersionId};

#[test]
fn ancestor_entity_basis_resolves_by_lineage_when_record_id_changed() {
    let ancestor_id = EntityId::new(PartitionId::main(), 1, 1);
    let branch_id = EntityId::new(PartitionId::main(), 99, 7);
    let lineage_id = LineageId::new(41);
    let view = read_view(
        vec![entity_record(ancestor_id, Some(lineage_id))],
        Vec::new(),
    );
    let context = AncestorRecordBasisContext::new(&view);
    let branch_record = VisibleMergeRecord {
        record_ref: RecordRef::Entity(branch_id),
        record_kind: VisibleMergeRecordKind::Entity,
        kind_id: Some(KindId::new(1)),
        source_kind_id: Some(KindId::new(1)),
        target_kind_id: None,
        lineage_id: Some(lineage_id),
        source_lineage_id: Some(lineage_id),
        target_lineage_id: None,
        source_entity: Some(entity_record(branch_id, Some(lineage_id))),
        target_entity: None,
        source_relation: None,
        target_relation: None,
    };

    let basis = context
        .entity_basis(&branch_record, None)
        .expect("lineage should resolve ancestor entity basis");

    assert_eq!(basis.record_ref(), RecordRef::Entity(ancestor_id));
    assert_eq!(basis.lifecycle(), RecordLifecycleState::Live);
}

#[test]
fn ancestor_relation_basis_resolves_by_storage_slot_when_generation_changed() {
    let left = EntityId::new(PartitionId::main(), 1, 1);
    let right = EntityId::new(PartitionId::main(), 2, 1);
    let ancestor_id = RelationId::new(PartitionId::main(), 5, 1);
    let branch_id = RelationId::new(PartitionId::main(), 5, 9);
    let view = read_view(Vec::new(), vec![relation_record(ancestor_id, left, right)]);
    let context = AncestorRecordBasisContext::new(&view);
    let branch_record = VisibleMergeRecord {
        record_ref: RecordRef::Relation(branch_id),
        record_kind: VisibleMergeRecordKind::Relation,
        kind_id: Some(KindId::new(2)),
        source_kind_id: Some(KindId::new(2)),
        target_kind_id: None,
        lineage_id: None,
        source_lineage_id: None,
        target_lineage_id: None,
        source_entity: None,
        target_entity: None,
        source_relation: Some(relation_record(branch_id, left, right)),
        target_relation: None,
    };

    let basis = context
        .relation_basis(&branch_record, None)
        .expect("slot should resolve ancestor relation basis");

    assert_eq!(basis.record_ref(), RecordRef::Relation(ancestor_id));
    assert_eq!(basis.source_endpoint(), left);
    assert_eq!(basis.target_endpoint(), right);
}

fn read_view(
    entities: Vec<EntityReadRecord>,
    relations: Vec<RelationReadRecord>,
) -> RelationalReadView {
    let runtime =
        crate::runtime::RelationalRuntime::new(crate::runtime::RelationalRuntimeConfig::default())
            .expect("valid native runtime configuration");
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    RelationalReadView {
        snapshot,
        entities,
        relations,
    }
}

fn entity_record(entity_id: EntityId, lineage_id: Option<LineageId>) -> EntityReadRecord {
    EntityReadRecord {
        entity_id,
        lineage_id,
        kind: kind_resolution(1),
        lifecycle: RecordLifecycleState::Live,
        created_at_version: VersionId::new(1),
        retired_at_version: None,
        authoritative_aspect_state: None,
    }
}

fn relation_record(
    relation_id: RelationId,
    source: EntityId,
    target: EntityId,
) -> RelationReadRecord {
    RelationReadRecord {
        relation_id,
        kind: kind_resolution(2),
        lifecycle: RecordLifecycleState::Live,
        created_at_version: VersionId::new(1),
        retired_at_version: None,
        source,
        target,
        authoritative_aspect_state: None,
    }
}

fn kind_resolution(kind: u32) -> KindResolution {
    KindResolution {
        kind_id: KindId::new(kind),
        kind_name: format!("kind-{kind}"),
        schema_id: SchemaId("test".to_string()),
        schema_version_id: SchemaVersionId(1),
    }
}
