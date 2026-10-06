use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::{EntityId, RelationId};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};

use super::canonical_digest::{
    entity_record_bytes, entity_record_digest_checked, query_result_reduction_digest_checked,
    relation_record_bytes, relation_record_digest_checked,
};
use super::{
    CanonicalQueryResult, DeterministicQueryFragmentKey, QueryExecutionShape,
    QueryOrderingContract, QueryWorkerFragment, TraversalEntityVisitKey, TraversalRelationVisitKey,
};

type EntityVisits = BTreeMap<(TraversalEntityVisitKey, String, usize), EntityReadRecord>;
type RelationVisits = BTreeMap<(TraversalRelationVisitKey, String, usize), RelationReadRecord>;

/// A fallible, ordered reduction for leased reads. Its callback admits each
/// intermediate before growth and checkpoints every record and digest chunk.
pub(crate) fn reduce_query_fragments_checked<E>(
    execution_shape: QueryExecutionShape,
    ordering: QueryOrderingContract,
    fragments: Vec<QueryWorkerFragment>,
    mut check: impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<CanonicalQueryResult, E> {
    let mut intermediate_bytes = 0_u64;
    let mut ordered =
        BTreeMap::<(DeterministicQueryFragmentKey, usize), QueryWorkerFragment>::new();
    for (ordinal, fragment) in fragments.into_iter().enumerate() {
        claim(
            &mut intermediate_bytes,
            (std::mem::size_of::<QueryWorkerFragment>() as u64).saturating_add(256),
            &mut check,
        )?;
        ordered.insert((fragment.fragment_key, ordinal), fragment);
    }
    let (entities, relations) = match ordering {
        QueryOrderingContract::CanonicalEntityIdOrder => {
            let entities = reduce_ordered_entities(
                ordered.into_values(),
                &mut intermediate_bytes,
                &mut check,
            )?;
            (entities, Vec::new())
        }
        QueryOrderingContract::CanonicalRelationIdOrder => {
            let relations = reduce_ordered_relations(
                ordered.into_values(),
                &mut intermediate_bytes,
                &mut check,
            )?;
            (Vec::new(), relations)
        }
        QueryOrderingContract::CanonicalRecordRefOrder => {
            reduce_ordered_both(ordered.into_values(), &mut intermediate_bytes, &mut check)?
        }
        QueryOrderingContract::CanonicalTraversalOrder => {
            reduce_traversal(ordered.into_values(), &mut intermediate_bytes, &mut check)?
        }
    };
    let reduction_digest = query_result_reduction_digest_checked(
        ordering,
        &entities,
        &relations,
        |work, temporary| check(work, intermediate_bytes.saturating_add(temporary)),
    )?;
    Ok(CanonicalQueryResult {
        execution_shape,
        ordering,
        entities,
        relations,
        reduction_digest,
    })
}

fn reduce_ordered_both<E>(
    fragments: impl IntoIterator<Item = QueryWorkerFragment>,
    intermediate_bytes: &mut u64,
    check: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<(Vec<EntityReadRecord>, Vec<RelationReadRecord>), E> {
    let mut entities = BTreeMap::<EntityId, Vec<EntityReadRecord>>::new();
    let mut relations = BTreeMap::<RelationId, Vec<RelationReadRecord>>::new();
    for fragment in fragments {
        for record in fragment.entities {
            claim(
                intermediate_bytes,
                entity_record_bytes(&record).saturating_add(256),
                check,
            )?;
            entities.entry(record.entity_id).or_default().push(record);
        }
        for record in fragment.relations {
            claim(
                intermediate_bytes,
                relation_record_bytes(&record).saturating_add(256),
                check,
            )?;
            relations
                .entry(record.relation_id)
                .or_default()
                .push(record);
        }
    }
    let entity_count = entities.values().map(Vec::len).sum();
    let relation_count = relations.values().map(Vec::len).sum();
    claim(
        intermediate_bytes,
        inline_bytes::<EntityReadRecord>(entity_count),
        check,
    )?;
    claim(
        intermediate_bytes,
        inline_bytes::<RelationReadRecord>(relation_count),
        check,
    )?;
    let mut ordered_entities = Vec::with_capacity(entity_count);
    let mut ordered_relations = Vec::with_capacity(relation_count);
    ordered_entities.extend(entities.into_values().flatten());
    ordered_relations.extend(relations.into_values().flatten());
    Ok((ordered_entities, ordered_relations))
}

fn reduce_ordered_entities<E>(
    fragments: impl IntoIterator<Item = QueryWorkerFragment>,
    intermediate_bytes: &mut u64,
    check: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<Vec<EntityReadRecord>, E> {
    let mut by_id = BTreeMap::<EntityId, Vec<EntityReadRecord>>::new();
    for fragment in fragments {
        for record in fragment.entities {
            claim(
                intermediate_bytes,
                entity_record_bytes(&record).saturating_add(256),
                check,
            )?;
            by_id.entry(record.entity_id).or_default().push(record);
        }
    }
    let count = by_id.values().map(Vec::len).sum::<usize>();
    claim(
        intermediate_bytes,
        inline_bytes::<EntityReadRecord>(count),
        check,
    )?;
    let mut ordered = Vec::with_capacity(count);
    ordered.extend(by_id.into_values().flatten());
    Ok(ordered)
}

fn reduce_ordered_relations<E>(
    fragments: impl IntoIterator<Item = QueryWorkerFragment>,
    intermediate_bytes: &mut u64,
    check: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<Vec<RelationReadRecord>, E> {
    let mut by_id = BTreeMap::<RelationId, Vec<RelationReadRecord>>::new();
    for fragment in fragments {
        for record in fragment.relations {
            claim(
                intermediate_bytes,
                relation_record_bytes(&record).saturating_add(256),
                check,
            )?;
            by_id.entry(record.relation_id).or_default().push(record);
        }
    }
    let count = by_id.values().map(Vec::len).sum::<usize>();
    claim(
        intermediate_bytes,
        inline_bytes::<RelationReadRecord>(count),
        check,
    )?;
    let mut ordered = Vec::with_capacity(count);
    ordered.extend(by_id.into_values().flatten());
    Ok(ordered)
}

fn reduce_traversal<E>(
    fragments: impl IntoIterator<Item = QueryWorkerFragment>,
    intermediate_bytes: &mut u64,
    check: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<(Vec<EntityReadRecord>, Vec<RelationReadRecord>), E> {
    let mut keyed_entities = EntityVisits::new();
    let mut keyed_relations = RelationVisits::new();
    let mut unkeyed_entities = Vec::new();
    let mut unkeyed_relations = Vec::new();
    let mut ordinal = 0;
    for fragment in fragments {
        if let Some(basis) = fragment.traversal_basis {
            for (key, record) in basis.entity_visit_keys.into_iter().zip(fragment.entities) {
                let digest = entity_record_digest_checked(&record, |work, temporary| {
                    check(work, intermediate_bytes.saturating_add(temporary))
                })?;
                claim(
                    intermediate_bytes,
                    entity_record_bytes(&record).saturating_add(256),
                    check,
                )?;
                keyed_entities.insert((key, digest, ordinal), record);
                ordinal += 1;
            }
            for (key, record) in basis
                .relation_visit_keys
                .into_iter()
                .zip(fragment.relations)
            {
                let digest = relation_record_digest_checked(&record, |work, temporary| {
                    check(work, intermediate_bytes.saturating_add(temporary))
                })?;
                claim(
                    intermediate_bytes,
                    relation_record_bytes(&record).saturating_add(256),
                    check,
                )?;
                keyed_relations.insert((key, digest, ordinal), record);
                ordinal += 1;
            }
        } else {
            for record in fragment.entities {
                claim(
                    intermediate_bytes,
                    entity_record_bytes(&record).saturating_mul(2),
                    check,
                )?;
                unkeyed_entities.push(record);
            }
            for record in fragment.relations {
                claim(
                    intermediate_bytes,
                    relation_record_bytes(&record).saturating_mul(2),
                    check,
                )?;
                unkeyed_relations.push(record);
            }
        }
    }
    let entities = if keyed_entities.is_empty() {
        dedupe_entities(unkeyed_entities, intermediate_bytes, check)?
    } else {
        dedupe_entities(keyed_entities.into_values(), intermediate_bytes, check)?
    };
    let relations = if keyed_relations.is_empty() {
        dedupe_relations(unkeyed_relations, intermediate_bytes, check)?
    } else {
        dedupe_relations(keyed_relations.into_values(), intermediate_bytes, check)?
    };
    Ok((entities, relations))
}

fn dedupe_entities<E>(
    records: impl IntoIterator<Item = EntityReadRecord>,
    intermediate_bytes: &mut u64,
    check: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<Vec<EntityReadRecord>, E> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for record in records {
        claim(
            intermediate_bytes,
            (std::mem::size_of::<EntityId>() * 16 + 128) as u64,
            check,
        )?;
        if seen.insert(record.entity_id) {
            claim(
                intermediate_bytes,
                (std::mem::size_of::<EntityReadRecord>() as u64).saturating_mul(2),
                check,
            )?;
            result.push(record);
        }
    }
    Ok(result)
}

fn dedupe_relations<E>(
    records: impl IntoIterator<Item = RelationReadRecord>,
    intermediate_bytes: &mut u64,
    check: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<Vec<RelationReadRecord>, E> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for record in records {
        claim(
            intermediate_bytes,
            (std::mem::size_of::<RelationId>() * 16 + 128) as u64,
            check,
        )?;
        if seen.insert(record.relation_id) {
            claim(
                intermediate_bytes,
                (std::mem::size_of::<RelationReadRecord>() as u64).saturating_mul(2),
                check,
            )?;
            result.push(record);
        }
    }
    Ok(result)
}

fn claim<E>(
    total: &mut u64,
    additional: u64,
    check: &mut impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<(), E> {
    let next = total.saturating_add(additional);
    check(1, next)?;
    *total = next;
    Ok(())
}

fn inline_bytes<T>(count: usize) -> u64 {
    count
        .checked_mul(std::mem::size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .unwrap_or(u64::MAX)
}
