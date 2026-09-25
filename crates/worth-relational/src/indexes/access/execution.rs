use std::collections::BTreeSet;

use worth_foundational::facade::AspectFieldLocator;

use crate::indexes::data::{DerivedIndexEntries, DerivedIndexGenerationId};
use crate::query::data::{
    reduce_query_fragments, QueryExecutionOutcome, QueryFragmentCounters, QueryScope,
    QueryWorkerFragment, SnapshotPinnedQueryPlan,
};
use crate::runtime::RelationalRuntime;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::visibility::materialization::read_records::{
    entity_query_locus_comparison_key, relation_query_locus_comparison_key,
};

use super::super::projected_field_values::IndexProjectionSource;
use super::scratch::{index_query_scratch_for_runtime, retain_index_query_scratch};

pub(crate) fn execute_index_backed_query_from_generation(
    runtime: &RelationalRuntime,
    plan: &SnapshotPinnedQueryPlan,
    generation_id: DerivedIndexGenerationId,
) -> Option<QueryExecutionOutcome> {
    let generation = runtime.indexes.generation(generation_id)?;
    let exact_projection = runtime.read_truth().project_snapshot(&plan.snapshot)?;
    let source = IndexProjectionSource::exact(&exact_projection)
        .expect("commit execution projection must carry an exact basis");
    match (&plan.packet.scope, &generation.entries) {
        (
            QueryScope::EntityFieldEquals {
                field_locator,
                value,
                partition_scope,
            },
            DerivedIndexEntries::EntityField(entries),
        ) => {
            let expected = query_value_comparison_key(value);
            execute_entity_index_lookup(
                runtime,
                plan,
                &source,
                field_locator,
                partition_scope.as_ref().map(|scope| &**scope),
                entries
                    .get(&expected)
                    .into_iter()
                    .flatten()
                    .copied()
                    .collect(),
                1,
                |record| {
                    entity_query_locus_comparison_key(record, field_locator)
                        == Some(expected.clone())
                },
            )
        }
        (
            QueryScope::EntityFieldAnyOf {
                field_locator,
                values,
                partition_scope,
            },
            DerivedIndexEntries::EntityField(entries),
        ) => {
            let values = QueryScope::canonical_value_scope(values.as_ref());
            let selected = values
                .iter()
                .map(AuthoritativeFieldComparisonKey::from_aspect_value)
                .collect::<BTreeSet<_>>();
            let candidate_ids = values
                .iter()
                .flat_map(|value| {
                    entries
                        .get(&AuthoritativeFieldComparisonKey::from_aspect_value(value))
                        .into_iter()
                        .flatten()
                        .copied()
                })
                .collect::<BTreeSet<_>>();
            execute_entity_index_lookup(
                runtime,
                plan,
                &source,
                field_locator,
                partition_scope.as_ref().map(|scope| &**scope),
                candidate_ids,
                values.len(),
                |record| {
                    entity_query_locus_comparison_key(record, field_locator)
                        .is_some_and(|value| selected.contains(&value))
                },
            )
        }
        (
            QueryScope::RelationFieldEquals {
                field_locator,
                value,
                partition_scope,
            },
            DerivedIndexEntries::RelationField(entries),
        ) => {
            let expected = query_value_comparison_key(value);
            execute_relation_index_lookup(
                runtime,
                plan,
                &source,
                field_locator,
                partition_scope.as_ref().map(|scope| &**scope),
                entries
                    .get(&expected)
                    .into_iter()
                    .flatten()
                    .copied()
                    .collect(),
                1,
                |record| {
                    relation_query_locus_comparison_key(record, field_locator)
                        == Some(expected.clone())
                },
            )
        }
        (
            QueryScope::RelationFieldAnyOf {
                field_locator,
                values,
                partition_scope,
            },
            DerivedIndexEntries::RelationField(entries),
        ) => {
            let values = QueryScope::canonical_value_scope(values.as_ref());
            let selected = values
                .iter()
                .map(AuthoritativeFieldComparisonKey::from_aspect_value)
                .collect::<BTreeSet<_>>();
            let candidate_ids = values
                .iter()
                .flat_map(|value| {
                    entries
                        .get(&AuthoritativeFieldComparisonKey::from_aspect_value(value))
                        .into_iter()
                        .flatten()
                        .copied()
                })
                .collect::<BTreeSet<_>>();
            execute_relation_index_lookup(
                runtime,
                plan,
                &source,
                field_locator,
                partition_scope.as_ref().map(|scope| &**scope),
                candidate_ids,
                values.len(),
                |record| {
                    relation_query_locus_comparison_key(record, field_locator)
                        .is_some_and(|value| selected.contains(&value))
                },
            )
        }
        _ => None,
    }
}

fn query_value_comparison_key(
    value: &worth_foundational::facade::AspectValue,
) -> AuthoritativeFieldComparisonKey {
    AuthoritativeFieldComparisonKey::from_aspect_value(value)
}

fn execute_entity_index_lookup(
    runtime: &RelationalRuntime,
    plan: &SnapshotPinnedQueryPlan,
    source: &IndexProjectionSource<'_, '_>,
    _field_locator: &AspectFieldLocator,
    partition_scope: Option<&[crate::identity::data::PartitionId]>,
    candidate_ids: BTreeSet<crate::identity::data::EntityId>,
    target_count: usize,
    include: impl Fn(&crate::storage::data::EntityReadRecord) -> bool,
) -> Option<QueryExecutionOutcome> {
    let partition_scope =
        partition_scope.map(|partitions| partitions.iter().copied().collect::<BTreeSet<_>>());
    let runtime_id = runtime.runtime_instance_id();
    let mut scratch = index_query_scratch_for_runtime(runtime, true);
    let mut entities = scratch.entity_buffer(candidate_ids.len());
    let mut touched_partition_ids = BTreeSet::new();
    for entity_id in candidate_ids {
        if partition_scope
            .as_ref()
            .is_some_and(|partitions| !partitions.contains(&entity_id.partition_id))
        {
            continue;
        }
        let Some(record) = source.with_entity(entity_id, Clone::clone) else {
            continue;
        };
        if include(&record) {
            touched_partition_ids.insert(record.entity_id.partition_id);
            entities.push(record);
        }
    }
    let touched_partitions = touched_partition_ids.len();
    let entity_count = entities.len();
    scratch.remember_entity_capacity(entity_count);
    retain_index_query_scratch(runtime_id, &scratch);
    Some(build_index_query_execution(
        plan,
        entities,
        Vec::new(),
        entity_count.max(target_count),
        entity_count,
        0,
        touched_partitions,
    ))
}

fn execute_relation_index_lookup(
    runtime: &RelationalRuntime,
    plan: &SnapshotPinnedQueryPlan,
    source: &IndexProjectionSource<'_, '_>,
    _field_locator: &AspectFieldLocator,
    partition_scope: Option<&[crate::identity::data::PartitionId]>,
    candidate_ids: BTreeSet<crate::identity::data::RelationId>,
    target_count: usize,
    include: impl Fn(&crate::storage::data::RelationReadRecord) -> bool,
) -> Option<QueryExecutionOutcome> {
    let partition_scope =
        partition_scope.map(|partitions| partitions.iter().copied().collect::<BTreeSet<_>>());
    let runtime_id = runtime.runtime_instance_id();
    let mut scratch = index_query_scratch_for_runtime(runtime, false);
    let mut relations = scratch.relation_buffer(candidate_ids.len());
    let mut touched_partition_ids = BTreeSet::new();
    for relation_id in candidate_ids {
        if partition_scope
            .as_ref()
            .is_some_and(|partitions| !partitions.contains(&relation_id.partition_id))
        {
            continue;
        }
        let Some(record) = source.with_relation(relation_id, Clone::clone) else {
            continue;
        };
        if include(&record) {
            touched_partition_ids.insert(record.relation_id.partition_id);
            relations.push(record);
        }
    }
    let touched_partitions = touched_partition_ids.len();
    let relation_count = relations.len();
    scratch.remember_relation_capacity(relation_count);
    retain_index_query_scratch(runtime_id, &scratch);
    Some(build_index_query_execution(
        plan,
        Vec::new(),
        relations,
        relation_count.max(target_count),
        0,
        relation_count,
        touched_partitions,
    ))
}

fn build_index_query_execution(
    plan: &SnapshotPinnedQueryPlan,
    entities: Vec<crate::storage::data::EntityReadRecord>,
    relations: Vec<crate::storage::data::RelationReadRecord>,
    target_count: usize,
    authoritative_entity_records_emitted: usize,
    authoritative_relation_records_emitted: usize,
    touched_partitions: usize,
) -> QueryExecutionOutcome {
    let result = reduce_query_fragments(
        plan.packet.execution_shape,
        plan.packet.ordering,
        vec![QueryWorkerFragment {
            plan_key: plan.packet.plan_key,
            fragment_key: crate::query::data::deterministic_query_fragment_key(
                plan.packet.plan_key,
                0,
            ),
            ordering: plan.packet.ordering,
            entities,
            relations,
            counters: QueryFragmentCounters {
                target_count,
                authoritative_entity_records_emitted,
                authoritative_relation_records_emitted,
                touched_partitions,
            },
            traversal_basis: None,
        }],
    );
    QueryExecutionOutcome {
        plan: plan.clone(),
        complexity: crate::query::data::QueryComplexitySummary {
            packet_count: 1,
            fragment_count: 1,
            touched_partitions,
            target_count,
            authoritative_entity_records_emitted,
            authoritative_relation_records_emitted,
        },
        result,
    }
}
