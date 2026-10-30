use serde::{Deserialize, Serialize};

use crate::storage::data::{EntityReadRecord, RelationReadRecord};

use super::{
    DeterministicQueryFragmentKey, DeterministicQueryPlanKey, QueryExecutionShape,
    QueryOrderingContract, SnapshotPinnedQueryPlan,
};
use crate::identity::data::EntityId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryFragmentCounters {
    pub target_count: usize,
    pub authoritative_entity_records_emitted: usize,
    pub authoritative_relation_records_emitted: usize,
    pub touched_partitions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TraversalEntityVisitKey {
    pub depth: u32,
    pub root_seed: EntityId,
    pub via_relation: Option<crate::identity::data::RelationId>,
    pub entity_id: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TraversalRelationVisitKey {
    pub depth: u32,
    pub root_seed: EntityId,
    pub relation_id: crate::identity::data::RelationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TraversalReductionBasis {
    pub entity_visit_keys: Vec<TraversalEntityVisitKey>,
    pub relation_visit_keys: Vec<TraversalRelationVisitKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QueryWorkerFragment {
    pub plan_key: DeterministicQueryPlanKey,
    pub fragment_key: DeterministicQueryFragmentKey,
    pub ordering: QueryOrderingContract,
    pub entities: Vec<EntityReadRecord>,
    pub relations: Vec<RelationReadRecord>,
    pub counters: QueryFragmentCounters,
    pub traversal_basis: Option<TraversalReductionBasis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalQueryResult {
    pub execution_shape: QueryExecutionShape,
    pub ordering: QueryOrderingContract,
    pub entities: Vec<EntityReadRecord>,
    pub relations: Vec<RelationReadRecord>,
    pub reduction_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryComplexitySummary {
    pub packet_count: usize,
    pub fragment_count: usize,
    pub touched_partitions: usize,
    pub target_count: usize,
    pub authoritative_entity_records_emitted: usize,
    pub authoritative_relation_records_emitted: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryExecutionOutcome {
    pub plan: SnapshotPinnedQueryPlan,
    pub result: CanonicalQueryResult,
    pub complexity: QueryComplexitySummary,
}

#[cfg(test)]
pub fn reduce_query_fragments(
    execution_shape: QueryExecutionShape,
    ordering: QueryOrderingContract,
    mut fragments: Vec<QueryWorkerFragment>,
) -> CanonicalQueryResult {
    fragments.sort_by_key(|fragment| fragment.fragment_key);
    let (entities, relations) = match ordering {
        QueryOrderingContract::CanonicalEntityIdOrder => {
            let mut entities = fragments
                .into_iter()
                .flat_map(|fragment| fragment.entities.into_iter())
                .collect::<Vec<_>>();
            entities.sort_by_key(|record| record.entity_id);
            (entities, Vec::new())
        }
        QueryOrderingContract::CanonicalRelationIdOrder => {
            let mut relations = fragments
                .into_iter()
                .flat_map(|fragment| fragment.relations.into_iter())
                .collect::<Vec<_>>();
            relations.sort_by_key(|record| record.relation_id);
            (Vec::new(), relations)
        }
        QueryOrderingContract::CanonicalRecordRefOrder => {
            let mut entities = Vec::new();
            let mut relations = Vec::new();
            for fragment in fragments {
                entities.extend(fragment.entities);
                relations.extend(fragment.relations);
            }
            entities.sort_by_key(|record| record.entity_id);
            relations.sort_by_key(|record| record.relation_id);
            (entities, relations)
        }
        QueryOrderingContract::CanonicalTraversalOrder => reduce_traversal_fragments(fragments),
    };

    let reduction_digest = super::query_result_reduction_digest(ordering, &entities, &relations);
    CanonicalQueryResult {
        execution_shape,
        ordering,
        entities,
        relations,
        reduction_digest,
    }
}

#[cfg(test)]
fn reduce_traversal_fragments(
    fragments: Vec<QueryWorkerFragment>,
) -> (Vec<EntityReadRecord>, Vec<RelationReadRecord>) {
    let entity_capacity = fragments
        .iter()
        .map(|fragment| fragment.entities.len())
        .sum();
    let relation_capacity = fragments
        .iter()
        .map(|fragment| fragment.relations.len())
        .sum();
    let mut keyed_entities = Vec::with_capacity(entity_capacity);
    let mut keyed_relations = Vec::with_capacity(relation_capacity);
    let mut unkeyed_entities = Vec::with_capacity(entity_capacity);
    let mut unkeyed_relations = Vec::with_capacity(relation_capacity);

    for fragment in fragments {
        let QueryWorkerFragment {
            entities,
            relations,
            traversal_basis,
            ..
        } = fragment;
        match traversal_basis {
            Some(traversal_basis) => {
                keyed_entities.extend(traversal_basis.entity_visit_keys.into_iter().zip(entities));
                keyed_relations.extend(
                    traversal_basis
                        .relation_visit_keys
                        .into_iter()
                        .zip(relations),
                );
            }
            None => {
                unkeyed_entities.extend(entities);
                unkeyed_relations.extend(relations);
            }
        }
    }

    (
        reduce_traversal_entities(keyed_entities, unkeyed_entities),
        reduce_traversal_relations(keyed_relations, unkeyed_relations),
    )
}

#[cfg(test)]
fn reduce_traversal_entities(
    mut keyed_entities: Vec<(TraversalEntityVisitKey, EntityReadRecord)>,
    unkeyed_entities: Vec<EntityReadRecord>,
) -> Vec<EntityReadRecord> {
    if keyed_entities.is_empty() {
        let mut seen = std::collections::BTreeSet::new();
        return unkeyed_entities
            .into_iter()
            .filter(|record| seen.insert(record.entity_id))
            .collect();
    }

    keyed_entities.sort_by(|left, right| {
        left.0.cmp(&right.0).then_with(|| {
            super::query_authoritative_entity_record_digest(&left.1)
                .cmp(&super::query_authoritative_entity_record_digest(&right.1))
        })
    });
    let mut seen = std::collections::BTreeSet::new();
    keyed_entities
        .into_iter()
        .filter_map(|(_, record)| seen.insert(record.entity_id).then_some(record))
        .collect()
}

#[cfg(test)]
fn reduce_traversal_relations(
    mut keyed_relations: Vec<(TraversalRelationVisitKey, RelationReadRecord)>,
    unkeyed_relations: Vec<RelationReadRecord>,
) -> Vec<RelationReadRecord> {
    if keyed_relations.is_empty() {
        let mut seen = std::collections::BTreeSet::new();
        return unkeyed_relations
            .into_iter()
            .filter(|record| seen.insert(record.relation_id))
            .collect();
    }

    keyed_relations.sort_by(|left, right| {
        left.0.cmp(&right.0).then_with(|| {
            super::query_authoritative_relation_record_digest(&left.1)
                .cmp(&super::query_authoritative_relation_record_digest(&right.1))
        })
    });
    let mut seen = std::collections::BTreeSet::new();
    keyed_relations
        .into_iter()
        .filter_map(|(_, record)| seen.insert(record.relation_id).then_some(record))
        .collect()
}
