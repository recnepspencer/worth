use crate::contracts::diagnostics::data::artifacts::{DiagnosticCode, RelationalDiagnosticsEntry};
use crate::contracts::diagnostics::data::fields::{
    RelationalDiagnosticFields, RelationalDiagnosticValue,
};
use crate::contracts::identity::data::primitives::{KindId, PartitionId};
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};

use super::outcomes::MutationEvent;

pub(crate) fn diagnostic_entry_for_mutation_event(
    event: MutationEvent,
) -> RelationalDiagnosticsEntry {
    match event {
        MutationEvent::EntityCreated { entity_id, kind_id } => entity_created(entity_id, kind_id),
        MutationEvent::BulkEntitiesCreated {
            partition_id,
            kind_id,
            count,
        } => bulk_entities_created(partition_id, kind_id, count),
        MutationEvent::EntityUpdated { entity_id } => entity_updated(entity_id),
        MutationEvent::EntityReplaced {
            replaced_entity_id,
            replacement_entity_id,
            kind_id,
        } => entity_replaced(replaced_entity_id, replacement_entity_id, kind_id),
        MutationEvent::EntityDeleted { entity_id } => entity_deleted(entity_id),
        MutationEvent::RelationCreated {
            relation_id,
            source,
            target,
            kind_id,
        } => relation_created(relation_id, source, target, kind_id),
        MutationEvent::RelationUpdated { relation_id } => relation_updated(relation_id),
        MutationEvent::BulkRelationsCreated {
            partition_id,
            kind_id,
            count,
        } => bulk_relations_created(partition_id, kind_id, count),
        MutationEvent::RelationDeleted { relation_id } => relation_deleted(relation_id),
    }
}

fn entity_created(entity_id: EntityId, kind_id: KindId) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::EntityCreated,
        "entity created",
        RelationalDiagnosticValue::object([
            ("entity", RelationalDiagnosticValue::EntityId(entity_id)),
            ("kind_id", RelationalDiagnosticValue::KindId(kind_id)),
        ])
        .into(),
    )
}

fn bulk_entities_created(
    partition_id: PartitionId,
    kind_id: KindId,
    count: usize,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::EntityCreated,
        "bulk entities created",
        bulk_record_fields(partition_id, kind_id, count),
    )
}

fn entity_updated(entity_id: EntityId) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::EntityUpdated,
        "entity updated",
        RelationalDiagnosticValue::object([(
            "entity",
            RelationalDiagnosticValue::EntityId(entity_id),
        )])
        .into(),
    )
}

fn entity_replaced(
    replaced_entity_id: EntityId,
    replacement_entity_id: EntityId,
    kind_id: KindId,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::EntityUpdated,
        "entity replaced",
        RelationalDiagnosticValue::object([
            (
                "replaced_entity",
                RelationalDiagnosticValue::EntityId(replaced_entity_id),
            ),
            (
                "replacement_entity",
                RelationalDiagnosticValue::EntityId(replacement_entity_id),
            ),
            ("kind_id", RelationalDiagnosticValue::KindId(kind_id)),
        ])
        .into(),
    )
}

fn entity_deleted(entity_id: EntityId) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::EntityDeleted,
        "entity deleted",
        RelationalDiagnosticValue::object([(
            "entity",
            RelationalDiagnosticValue::EntityId(entity_id),
        )])
        .into(),
    )
}

fn relation_created(
    relation_id: RelationId,
    source: EntityId,
    target: EntityId,
    kind_id: KindId,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::RelationCreated,
        "relation created",
        RelationalDiagnosticValue::object([
            (
                "relation",
                RelationalDiagnosticValue::RelationId(relation_id),
            ),
            ("source", RelationalDiagnosticValue::EntityId(source)),
            ("target", RelationalDiagnosticValue::EntityId(target)),
            ("kind_id", RelationalDiagnosticValue::KindId(kind_id)),
        ])
        .into(),
    )
}

fn relation_updated(relation_id: RelationId) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::RelationUpdated,
        "relation updated",
        RelationalDiagnosticValue::object([(
            "relation",
            RelationalDiagnosticValue::RelationId(relation_id),
        )])
        .into(),
    )
}

fn bulk_relations_created(
    partition_id: PartitionId,
    kind_id: KindId,
    count: usize,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::RelationCreated,
        "bulk relations created",
        bulk_record_fields(partition_id, kind_id, count),
    )
}

fn relation_deleted(relation_id: RelationId) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::RelationDeleted,
        "relation deleted",
        RelationalDiagnosticValue::object([(
            "relation",
            RelationalDiagnosticValue::RelationId(relation_id),
        )])
        .into(),
    )
}

fn bulk_record_fields(
    partition_id: PartitionId,
    kind_id: KindId,
    count: usize,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "partition_id",
            RelationalDiagnosticValue::PartitionId(partition_id),
        ),
        ("kind_id", RelationalDiagnosticValue::KindId(kind_id)),
        ("count", RelationalDiagnosticValue::unsigned(count)),
    ])
    .into()
}
