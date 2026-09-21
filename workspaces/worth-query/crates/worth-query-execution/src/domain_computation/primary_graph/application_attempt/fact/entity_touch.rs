use worth_relational::facade::identity::EntityId;

use super::WorthQueryApplicationObservedFact;

pub(super) fn evaluate(fact: &WorthQueryApplicationObservedFact, candidate: EntityId) -> bool {
    match fact {
        WorthQueryApplicationObservedFact::SourceEntity { entity_id } => *entity_id == candidate,
        WorthQueryApplicationObservedFact::SourceAspectRevision { entity_id, .. } => {
            *entity_id == candidate
        }
        WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
            anchor, endpoints, ..
        } => *anchor == candidate || endpoints.contains(&candidate),
        WorthQueryApplicationObservedFact::Entity { entity_id, .. }
        | WorthQueryApplicationObservedFact::Field { entity_id, .. }
        | WorthQueryApplicationObservedFact::AbsentField { entity_id, .. } => {
            *entity_id == candidate
        }
        WorthQueryApplicationObservedFact::Relation { from, to, .. } => {
            *from == candidate || *to == candidate
        }
        WorthQueryApplicationObservedFact::Adjacency {
            anchor, relations, ..
        } => {
            *anchor == candidate
                || relations
                    .iter()
                    .any(|relation| relation.from == candidate || relation.to == candidate)
        }
        WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. } => {
            candidates.contains(&candidate)
        }
        WorthQueryApplicationObservedFact::WorkflowDefinitionPredecessor {
            lineage,
            expected_definition,
            ..
        } => {
            lineage.is_some_and(|entity| entity == candidate)
                || expected_definition.is_some_and(|entity| entity == candidate)
        }
        WorthQueryApplicationObservedFact::WorkflowDefinitionCurrent {
            lineage,
            expected_definition,
            ..
        } => *lineage == candidate || *expected_definition == candidate,
        WorthQueryApplicationObservedFact::WorkflowInstanceCapacity {
            lineage, instances, ..
        } => {
            *lineage == candidate
                || instances
                    .iter()
                    .any(|relation| relation.from == candidate || relation.to == candidate)
        }
        WorthQueryApplicationObservedFact::WorkflowTransitionCapacity {
            instance,
            transitions,
            ..
        } => {
            *instance == candidate
                || transitions
                    .iter()
                    .any(|relation| relation.from == candidate || relation.to == candidate)
        }
    }
}
