//! Independent reduction used by the query courts.
use super::fragment_reduction::{TraversalEntityVisitKey, TraversalRelationVisitKey};
use super::{
    CanonicalQueryResult, QueryExecutionShape, QueryOrderingContract, QueryWorkerFragment,
};
use crate::storage::data::read_records::{EntityReadRecord, RelationReadRecord};

#[doc(hidden)]
pub fn reduce_query_fragments_for_test(
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

    let reduction_digest = crate::query::data::canonical_digest::query_result_reduction_digest(
        ordering, &entities, &relations,
    );
    CanonicalQueryResult {
        execution_shape,
        ordering,
        entities,
        relations,
        reduction_digest,
    }
}

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
            crate::query::data::canonical_digest::query_authoritative_entity_record_digest(&left.1)
                .cmp(
                    &crate::query::data::canonical_digest::query_authoritative_entity_record_digest(
                        &right.1,
                    ),
                )
        })
    });
    let mut seen = std::collections::BTreeSet::new();
    keyed_entities
        .into_iter()
        .filter_map(|(_, record)| seen.insert(record.entity_id).then_some(record))
        .collect()
}

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
            crate::query::data::canonical_digest::query_authoritative_relation_record_digest(
                &left.1,
            )
            .cmp(
                &crate::query::data::canonical_digest::query_authoritative_relation_record_digest(
                    &right.1,
                ),
            )
        })
    });
    let mut seen = std::collections::BTreeSet::new();
    keyed_relations
        .into_iter()
        .filter_map(|(_, record)| seen.insert(record.relation_id).then_some(record))
        .collect()
}
