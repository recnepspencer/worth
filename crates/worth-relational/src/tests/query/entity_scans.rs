use crate::facade::identity::EntityId;
use crate::facade::runtime::{
    EntityProjectionRecord, EntityRecordProjection, ProjectionAspectScope,
};
use crate::tests::support::*;
use std::sync::OnceLock;
use worth_foundational::facade::{AspectValue, InternedString};

fn entity_name_aspects() -> Vec<AspectKey> {
    static ASPECTS: OnceLock<Vec<AspectKey>> = OnceLock::new();
    ASPECTS
        .get_or_init(|| vec![AspectKey::new("name").unwrap()])
        .clone()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NamedEntityProjection {
    entity_id: EntityId,
    name: String,
}

impl EntityRecordProjection for NamedEntityProjection {
    const KIND: KindId = KindId(1);

    fn projection_scope() -> ProjectionAspectScope {
        ProjectionAspectScope::whole_aspects(entity_name_aspects())
    }

    fn from_record(record: EntityProjectionRecord<'_>) -> Option<Self> {
        let AspectValue::String(name) = record.aspect_value(&AspectKey::new("name").unwrap())?
        else {
            return None;
        };
        Some(Self {
            entity_id: record.entity_id(),
            name: raw_interned_string(name)?.to_string(),
        })
    }
}

fn raw_interned_string(value: &InternedString) -> Option<&str> {
    match value {
        InternedString::Raw(value) => Some(value.as_str()),
        InternedString::Symbol(_) => None,
    }
}

// CONTRACT: entity_kind_scans
// LANES: success, adversarial, determinism, historical

#[test]
fn entity_kind_scans_can_be_partition_scoped_without_cross_partition_leakage() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let left = create_entity_in_partition(execution, &runtime, "left-a", PartitionId(7));
    let _right = create_entity_in_partition(execution, &runtime, "right-a", PartitionId(11));
    let version_id = runtime.history().latest_commit().unwrap().version_id;

    let scoped = runtime
        .read_truth()
        .project_historical_version(execution, version_id)
        .entities_in::<NamedEntityProjection>(execution, PartitionId(7));

    assert_eq!(scoped.len(), 1);
    assert_eq!(scoped[0].entity_id, left);
    assert!(scoped
        .iter()
        .all(|record| record.entity_id.partition_id == PartitionId(7)));
}

#[test]
fn entity_kind_scans_are_deterministic_across_equivalent_insert_order() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let ordered = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let ordered_a = create_entity_in_partition(execution, &ordered, "a", PartitionId(3));
    let ordered_b = create_entity_in_partition(execution, &ordered, "b", PartitionId(3));

    let reversed = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let reversed_b = create_entity_in_partition(execution, &reversed, "b", PartitionId(3));
    let reversed_a = create_entity_in_partition(execution, &reversed, "a", PartitionId(3));

    let ordered_records = ordered
        .read_truth()
        .project_historical_version(execution, ordered.current_version_id())
        .entities_in::<NamedEntityProjection>(execution, PartitionId(3));
    let reversed_records = reversed
        .read_truth()
        .project_historical_version(execution, reversed.current_version_id())
        .entities_in::<NamedEntityProjection>(execution, PartitionId(3));

    assert_eq!(
        ordered_records
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        vec![ordered_a, ordered_b]
    );
    assert_eq!(
        reversed_records
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        vec![reversed_b, reversed_a]
    );
    assert_eq!(
        ordered
            .read_truth()
            .project_historical_version(execution, ordered.current_version_id())
            .entities_in::<NamedEntityProjection>(execution, PartitionId(3))
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        ordered_records
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>()
    );
    assert!(reversed_b.local_slot.0 < reversed_a.local_slot.0);
}

#[test]
fn entity_kind_scans_preserve_historical_partition_visibility() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let baseline =
        create_entity_outcome_on_branch(execution, &runtime, "base", BranchId("main".to_string()));
    let main_entity = changed_entities(&baseline)[0];
    let left = create_entity_in_partition(execution, &runtime, "left", PartitionId(17));
    let historical_version = runtime.history().latest_commit().unwrap().version_id;
    let _other_partition =
        create_entity_in_partition(execution, &runtime, "other", PartitionId(23));
    let _update = update_entity(execution, &runtime, main_entity, "base-updated");
    let _later_left =
        create_entity_in_partition(execution, &runtime, "left-later", PartitionId(17));

    let historical = runtime
        .read_truth()
        .project_historical_version(execution, historical_version)
        .entities_in::<NamedEntityProjection>(execution, PartitionId(17));

    assert_eq!(historical.len(), 1);
    assert_eq!(historical[0].entity_id, left);
    assert_eq!(historical[0].name, "left");
}
