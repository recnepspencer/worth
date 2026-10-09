use crate::facade::identity::EntityId;
use crate::facade::runtime::{
    ProjectionAspectScope, RelationProjectionRecord, RelationRecordProjection,
};
use crate::tests::support::*;
use std::sync::OnceLock;

fn relation_label_aspects() -> Vec<AspectKey> {
    static ASPECTS: OnceLock<Vec<AspectKey>> = OnceLock::new();
    ASPECTS
        .get_or_init(|| vec![AspectKey::new("label").unwrap()])
        .clone()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EdgeProjection {
    relation_id: RelationId,
    source: EntityId,
    target: EntityId,
}

impl RelationRecordProjection for EdgeProjection {
    const KIND: KindId = KindId(2);

    fn projection_scope() -> ProjectionAspectScope {
        ProjectionAspectScope::whole_aspects(relation_label_aspects())
    }

    fn from_record(record: RelationProjectionRecord<'_>) -> Option<Self> {
        Some(Self {
            relation_id: record.relation_id(),
            source: record.source(),
            target: record.target(),
        })
    }
}

// CONTRACT: relation_scans
// LANES: success, determinism, adversarial

#[test]
fn relation_kind_scans_return_only_visible_relations_of_that_kind() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let left = create_entity_outcome(&runtime, "left");
    let right = create_entity_outcome(&runtime, "right");
    let third = create_entity_outcome(&runtime, "third");
    let left = changed_entities(&left)[0];
    let right = changed_entities(&right)[0];
    let third = changed_entities(&third)[0];
    let r1 = create_relation(&runtime, left, right, "r1");
    let r2 = create_relation(&runtime, right, third, "r2");
    let deleted = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("delete-r1").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: r1 }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    };
    let visible = runtime
        .read_truth()
        .project_historical_version(deleted.version_id)
        .relations::<EdgeProjection>();

    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].relation_id, r2);
}

#[test]
fn relation_kind_scans_are_deterministic_across_equivalent_insert_order() {
    let runtime_a =
        runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let a_left = create_entity(&runtime_a, "left");
    let a_right = create_entity(&runtime_a, "right");
    let a_third = create_entity(&runtime_a, "third");
    let _ = create_relation(&runtime_a, a_left, a_right, "r1");
    let _ = create_relation(&runtime_a, a_right, a_third, "r2");
    let scan_a = runtime_a
        .read_truth()
        .project_historical_version(runtime_a.current_version_id())
        .relations::<EdgeProjection>();

    let runtime_b =
        runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let b_left = create_entity(&runtime_b, "left");
    let b_right = create_entity(&runtime_b, "right");
    let b_third = create_entity(&runtime_b, "third");
    let _ = create_relation(&runtime_b, b_right, b_third, "r2");
    let _ = create_relation(&runtime_b, b_left, b_right, "r1");
    let scan_b = runtime_b
        .read_truth()
        .project_historical_version(runtime_b.current_version_id())
        .relations::<EdgeProjection>();

    assert_eq!(scan_a.len(), scan_b.len());
    assert_eq!(
        scan_a
            .iter()
            .map(|record| (record.source.local_slot.0, record.target.local_slot.0))
            .collect::<Vec<_>>(),
        scan_b
            .iter()
            .map(|record| (record.source.local_slot.0, record.target.local_slot.0))
            .collect::<Vec<_>>()
    );
}

#[test]
fn all_authoritative_relation_records_use_canonical_relation_order_not_creation_order() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let left = create_entity(&runtime, "left");
    let middle = create_entity(&runtime, "middle");
    let right = create_entity(&runtime, "right");
    let _earlier_created_but_canonically_second =
        create_relation(&runtime, middle, right, "middle-right");
    let _later_created_but_canonically_first =
        create_relation(&runtime, left, middle, "left-middle");

    let projected = runtime
        .read_truth()
        .project_historical_version(runtime.current_version_id())
        .relations::<EdgeProjection>();
    let all_records = runtime
        .read_truth()
        .project_historical_version(runtime.current_version_id())
        .all_authoritative_relation_records();

    assert_eq!(
        all_records
            .iter()
            .map(|record| (record.source.local_slot.0, record.target.local_slot.0))
            .collect::<Vec<_>>(),
        projected
            .iter()
            .map(|record| (record.source.local_slot.0, record.target.local_slot.0))
            .collect::<Vec<_>>()
    );
}

#[test]
fn relation_aspects_at_version_follow_declared_contract_shape() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let left = create_entity(&runtime, "left");
    let right = create_entity(&runtime, "right");
    let relation = create_relation(&runtime, left, right, "declared");
    let version_id = runtime.history().latest_commit().unwrap().version_id;

    let aspects = runtime
        .read_truth()
        .relation_aspects_at_version(relation, version_id)
        .unwrap();

    assert_eq!(
        aspects,
        vec![
            AspectKey::new("label").unwrap(),
            AspectKey::new("lifecycle").unwrap(),
            AspectKey::new("source").unwrap(),
            AspectKey::new("target").unwrap(),
        ]
    );
}
