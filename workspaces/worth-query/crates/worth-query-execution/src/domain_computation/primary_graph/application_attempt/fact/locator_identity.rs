use super::WorthQueryApplicationObservedFact;

pub(super) fn encode(fact: &WorthQueryApplicationObservedFact) -> String {
    let mut output = String::new();
    write(fact, &mut output).expect("a String destination accepts locator formatting");
    output
}

pub(super) fn write(
    fact: &WorthQueryApplicationObservedFact,
    output: &mut dyn std::fmt::Write,
) -> std::fmt::Result {
    match fact {
        WorthQueryApplicationObservedFact::RetiredOutputEntity { read_locator, .. } => output.write_str(read_locator),
        WorthQueryApplicationObservedFact::SourceEntity { entity_id } => write!(output,
            "application-source-entity:{}:{}:{}",
            entity_id.partition_value(),
            entity_id.local_slot_value(),
            entity_id.generation_value()
        ),
        WorthQueryApplicationObservedFact::SourceAspectRevision {
            entity_id, aspect, ..
        } => write!(output,
            "application-source-aspect:{}:{}:{}:{}",
            entity_id.partition_value(),
            entity_id.local_slot_value(),
            entity_id.generation_value(),
            aspect.as_str()
        ),
        WorthQueryApplicationObservedFact::SourceFieldRevision { entity_id, locator, .. } => write!(output,
            "application-source-field:{}:{}:{}:{}/{}",
            entity_id.partition_value(),
            entity_id.local_slot_value(),
            entity_id.generation_value(),
            locator.aspect().aspect_key().as_str(),
            locator.field_path().fields()[0].as_str(),
        ),
        WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
            relation_kind,
            anchor,
            direction,
            ..
        } => write!(output,
            "application-source-adjacency:{direction:?}:{}:{}:{}:kind:{}",
            anchor.partition_value(),
            anchor.local_slot_value(),
            anchor.generation_value(),
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::Entity {
            entity_id, kind, ..
        } => write!(output,
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
        } => write!(output,
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
        } => write!(output,
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
        } => write!(output,
            "application-adjacency:{direction:?}:{}:{}:{}:kind:{}",
            anchor.partition_value(),
            anchor.local_slot_value(),
            anchor.generation_value(),
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::IndexedEntitySelection { index_id, .. } => {
            write!(output, "application-indexed-entity-selection:{}", index_id.0)
        }
        WorthQueryApplicationObservedFact::WorkflowDefinitionPredecessor {
            relation_kind,
            lineage,
            expected_definition,
            ..
        } => write!(output,
            "workflow-definition-predecessor:kind:{}:lineage:{lineage:?}:definition:{expected_definition:?}",
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::WorkflowDefinitionCurrent {
            relation_kind,
            lineage,
            expected_definition,
            ..
        } => write!(output,
            "workflow-definition-current:kind:{}:lineage:{lineage:?}:definition:{expected_definition:?}",
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::WorkflowInstanceCapacity {
            relation_kind,
            lineage,
            ..
        } => write!(output,
            "workflow-instance-capacity:kind:{}:lineage:{lineage:?}",
            relation_kind.as_u32()
        ),
        WorthQueryApplicationObservedFact::WorkflowHistoryBasis { instance, .. } =>
            write!(output, "workflow-history-basis:instance:{instance:?}"),
    }
}
