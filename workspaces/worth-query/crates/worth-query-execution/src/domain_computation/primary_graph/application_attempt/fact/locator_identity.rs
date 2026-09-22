use super::WorthQueryApplicationObservedFact;

pub(super) fn encode(fact: &WorthQueryApplicationObservedFact) -> String {
    match fact {
        WorthQueryApplicationObservedFact::SourceEntity { entity_id } => format!(
            "application-source-entity:{}:{}:{}",
            entity_id.partition_value(),
            entity_id.local_slot_value(),
            entity_id.generation_value()
        ),
        WorthQueryApplicationObservedFact::SourceAspectRevision {
            entity_id, aspect, ..
        } => format!(
            "application-source-aspect:{}:{}:{}:{}",
            entity_id.partition_value(),
            entity_id.local_slot_value(),
            entity_id.generation_value(),
            aspect.as_str()
        ),
        WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
            relation_kind,
            anchor,
            direction,
            ..
        } => format!(
            "application-source-adjacency:{direction:?}:{}:{}:{}:kind:{}",
            anchor.partition_value(),
            anchor.local_slot_value(),
            anchor.generation_value(),
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::Entity {
            entity_id, kind, ..
        } => format!(
            "application-entity:{}:{}:{}:kind:{}",
            entity_id.partition_value(),
            entity_id.local_slot_value(),
            entity_id.generation_value(),
            kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::Field {
            entity_id, locator, ..
        }
        | WorthQueryApplicationObservedFact::AbsentField {
            entity_id, locator, ..
        } => format!(
            "application-field:{}:{}:{}:{}/{}",
            entity_id.partition_value(),
            entity_id.local_slot_value(),
            entity_id.generation_value(),
            locator.aspect().aspect_key().as_str(),
            locator
                .field_path()
                .fields()
                .first()
                .expect("installed application fields have one field path")
                .as_str()
        ),
        WorthQueryApplicationObservedFact::Relation {
            relation_kind,
            from,
            to,
            ..
        } => format!(
            "application-relation:{}:{}:{}->{}:{}:{}:kind:{}",
            from.partition_value(),
            from.local_slot_value(),
            from.generation_value(),
            to.partition_value(),
            to.local_slot_value(),
            to.generation_value(),
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::Adjacency {
            relation_kind,
            anchor,
            direction,
            ..
        } => format!(
            "application-adjacency:{direction:?}:{}:{}:{}:kind:{}",
            anchor.partition_value(),
            anchor.local_slot_value(),
            anchor.generation_value(),
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::IndexedEntitySelection { index_id, .. } => {
            format!("application-indexed-entity-selection:{}", index_id.0)
        }
        WorthQueryApplicationObservedFact::WorkflowDefinitionPredecessor {
            relation_kind,
            lineage,
            expected_definition,
            ..
        } => format!(
            "workflow-definition-predecessor:kind:{}:lineage:{lineage:?}:definition:{expected_definition:?}",
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::WorkflowDefinitionCurrent {
            relation_kind,
            lineage,
            expected_definition,
            ..
        } => format!(
            "workflow-definition-current:kind:{}:lineage:{lineage:?}:definition:{expected_definition:?}",
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::WorkflowInstanceCapacity {
            relation_kind,
            lineage,
            ..
        } => format!(
            "workflow-instance-capacity:kind:{}:lineage:{lineage:?}",
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::WorkflowHistoryBasis { instance, .. } =>
            format!("workflow-history-basis:instance:{instance:?}"),
    }
}
