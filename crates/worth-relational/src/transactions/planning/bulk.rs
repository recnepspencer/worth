//! Canonical planning for bulk mutation batches.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::authority::intent_merge::canonical_intent_key;
use crate::symbols::data::{ClientKey, ClientKeySymbolPolicy, StringInterner};
use crate::transactions::data::{
    bulk_lineage_plan_digest, bulk_naming_plan_digest, bulk_provenance_plan_digest,
    BulkMutationLineagePlan, BulkMutationLocalityFootprint, BulkMutationNamingPlan,
    BulkMutationProvenancePlan, BulkMutationScope, CreateIntent, EntityMutationIntent,
    MutationIntent, PlannedLineageTransition, RelationMutationIntent, TransactionId,
    WorkerIntentBatch,
};

pub(crate) fn canonical_bulk_mutation_intents(
    batches: &[WorkerIntentBatch],
    client_key_symbol_policy: ClientKeySymbolPolicy,
    interner: StringInterner,
) -> Vec<MutationIntent> {
    let mut intents = batches
        .iter()
        .flat_map(|batch| batch.intents.iter().cloned())
        .collect::<Vec<_>>();
    normalize_intents_for_bulk_plan(&mut intents, client_key_symbol_policy, interner);
    intents.sort_by_key(canonical_intent_key);
    intents
}

pub(crate) fn bulk_mutation_scope(intents: &[MutationIntent]) -> BulkMutationScope {
    let mut saw_entity_create = false;
    let mut saw_relation_create = false;
    let mut saw_topology_rewrite = false;

    for intent in intents {
        match intent {
            MutationIntent::Create(CreateIntent::Entity(_))
            | MutationIntent::Create(CreateIntent::EntityAspects(_))
            | MutationIntent::Create(CreateIntent::BulkEntities(_)) => {
                saw_entity_create = true;
            }
            MutationIntent::Create(CreateIntent::Relation(_))
            | MutationIntent::Create(CreateIntent::RelationAspects(_))
            | MutationIntent::Create(CreateIntent::BulkRelations(_)) => {
                saw_relation_create = true;
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(_))
            | MutationIntent::Entity(EntityMutationIntent::Delete(_))
            | MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(_))
            | MutationIntent::Relation(RelationMutationIntent::Delete(_)) => {
                saw_topology_rewrite = true;
            }
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(_))
            | MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Entity(EntityMutationIntent::Revalidate(_))
            | MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(_)) => {}
            MutationIntent::Materialization(_) => {
                saw_topology_rewrite = true;
            }
        }
    }

    if saw_topology_rewrite {
        BulkMutationScope::TopologyRegionRewrite
    } else if saw_entity_create && saw_relation_create {
        BulkMutationScope::BulkMixedMutation
    } else if saw_relation_create {
        BulkMutationScope::BulkRelationCreate
    } else {
        BulkMutationScope::BulkEntityCreate
    }
}

pub(crate) fn bulk_mutation_locality(intents: &[MutationIntent]) -> BulkMutationLocalityFootprint {
    let mut touched_partitions = BTreeSet::new();
    let mut cross_partition_relation_count = 0usize;
    let mut entity_target_count = 0usize;
    let mut relation_target_count = 0usize;

    for intent in intents {
        intent.seed_touched_partitions(&mut touched_partitions);
        match intent {
            MutationIntent::Create(CreateIntent::Entity(_))
            | MutationIntent::Create(CreateIntent::EntityAspects(_))
            | MutationIntent::Entity(EntityMutationIntent::UpdateFields(_))
            | MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Entity(EntityMutationIntent::Replace(_))
            | MutationIntent::Entity(EntityMutationIntent::Delete(_))
            | MutationIntent::Entity(EntityMutationIntent::Revalidate(_)) => {
                entity_target_count += 1;
            }
            MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                entity_target_count += spec.field_patches.len();
            }
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                relation_target_count += 1;
                if spec.source.partition_id() != spec.target.partition_id() {
                    cross_partition_relation_count += 1;
                }
            }
            MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                relation_target_count += 1;
                if spec.source.partition_id() != spec.target.partition_id() {
                    cross_partition_relation_count += 1;
                }
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                relation_target_count += spec.endpoints.len();
                cross_partition_relation_count += spec
                    .endpoints
                    .iter()
                    .filter(|(source, target)| source.partition_id() != target.partition_id())
                    .count();
            }
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(_))
            | MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Relation(RelationMutationIntent::Delete(_)) => {
                relation_target_count += 1;
            }
            MutationIntent::Materialization(intent) => match intent.record() {
                crate::transactions::data::RecordRef::Entity(_) => entity_target_count += 1,
                crate::transactions::data::RecordRef::Relation(_) => relation_target_count += 1,
            },
        }
    }

    BulkMutationLocalityFootprint {
        touched_partitions: touched_partitions.into_iter().collect::<Vec<_>>().into(),
        cross_partition_relation_count,
        entity_target_count,
        relation_target_count,
    }
}

pub(crate) fn bulk_mutation_naming(intents: &[MutationIntent]) -> BulkMutationNamingPlan {
    let mut normalized_client_keys = Vec::new();
    for intent in intents {
        match intent {
            MutationIntent::Create(CreateIntent::Entity(spec)) => {
                normalized_client_keys.push(spec.client_key.clone());
            }
            MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
                normalized_client_keys.push(spec.client_key.clone());
            }
            MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                normalized_client_keys.extend(spec.client_keys.iter().cloned());
            }
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                normalized_client_keys.push(spec.client_key.clone());
            }
            MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                normalized_client_keys.push(spec.client_key.clone());
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                normalized_client_keys.extend(spec.client_keys.iter().cloned());
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                normalized_client_keys.push(spec.replacement.client_key.clone());
            }
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(_))
            | MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Entity(EntityMutationIntent::Delete(_))
            | MutationIntent::Entity(EntityMutationIntent::Revalidate(_))
            | MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(_))
            | MutationIntent::Relation(RelationMutationIntent::Delete(_)) => {}
            MutationIntent::Materialization(_) => {}
        }
    }
    normalized_client_keys.sort();

    BulkMutationNamingPlan {
        naming_digest: bulk_naming_plan_digest(&normalized_client_keys),
        normalized_client_keys: Arc::<[ClientKey]>::from(normalized_client_keys),
    }
}

pub(crate) fn bulk_mutation_lineage(intents: &[MutationIntent]) -> BulkMutationLineagePlan {
    let mut transitions = Vec::new();
    for intent in intents {
        match intent {
            MutationIntent::Create(CreateIntent::Entity(spec)) => {
                transitions.push(PlannedLineageTransition::CreateEntity {
                    partition_id: spec.partition_id,
                    kind_id: spec.kind_id,
                    client_key: spec.client_key.clone(),
                });
            }
            MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
                transitions.push(PlannedLineageTransition::CreateEntity {
                    partition_id: spec.partition_id,
                    kind_id: spec.kind_id,
                    client_key: spec.client_key.clone(),
                });
            }
            MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                for client_key in &spec.client_keys {
                    transitions.push(PlannedLineageTransition::CreateEntity {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: client_key.clone(),
                    });
                }
            }
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                transitions.push(PlannedLineageTransition::CreateRelation {
                    partition_id: spec.partition_id,
                    kind_id: spec.kind_id,
                    source: spec.source.clone(),
                    target: spec.target.clone(),
                    client_key: spec.client_key.clone(),
                });
            }
            MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                transitions.push(PlannedLineageTransition::CreateRelation {
                    partition_id: spec.partition_id,
                    kind_id: spec.kind_id,
                    source: spec.source.clone(),
                    target: spec.target.clone(),
                    client_key: spec.client_key.clone(),
                });
            }
            MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                for (client_key, (source, target)) in
                    spec.client_keys.iter().zip(spec.endpoints.iter())
                {
                    transitions.push(PlannedLineageTransition::CreateRelation {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        source: source.clone(),
                        target: target.clone(),
                        client_key: client_key.clone(),
                    });
                }
            }
            MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                transitions.push(PlannedLineageTransition::ReplaceEntity {
                    entity_id: spec.entity_id,
                    replacement_partition_id: spec.replacement.partition_id,
                    replacement_kind_id: spec.replacement.kind_id,
                    replacement_client_key: spec.replacement.client_key.clone(),
                });
            }
            MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
                transitions.push(PlannedLineageTransition::DeleteEntity {
                    entity_id: spec.entity_id,
                });
            }
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                transitions.push(PlannedLineageTransition::UpdateRelationEndpoints {
                    relation_id: spec.relation_id,
                    source: spec.source.clone(),
                    target: spec.target.clone(),
                });
            }
            MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
                transitions.push(PlannedLineageTransition::DeleteRelation {
                    relation_id: spec.relation_id,
                });
            }
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(_))
            | MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Entity(EntityMutationIntent::Revalidate(_))
            | MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(_))
            | MutationIntent::Materialization(_) => {}
        }
    }

    BulkMutationLineagePlan {
        lineage_scope_digest: bulk_lineage_plan_digest(&transitions),
        transitions: transitions.into(),
    }
}

pub(crate) fn bulk_mutation_provenance(
    transaction_id: TransactionId,
    target_branch: Option<crate::history::data::BranchId>,
    batches: &[WorkerIntentBatch],
) -> BulkMutationProvenancePlan {
    let batch_name = format!("transaction-{}", transaction_id.0);
    let worker_batch_names = batches
        .iter()
        .map(|batch| batch.name.clone())
        .collect::<Vec<_>>();
    let worker_partition_keys = batches
        .iter()
        .map(|batch| batch.partition_key.clone())
        .collect::<Vec<_>>();
    let worker_local_only_flags = batches
        .iter()
        .map(|batch| batch.worker_local_only)
        .collect::<Vec<_>>();
    let provenance_digest = bulk_provenance_plan_digest(
        transaction_id,
        target_branch.as_ref(),
        &batch_name,
        &worker_batch_names,
        &worker_partition_keys,
        &worker_local_only_flags,
    );

    BulkMutationProvenancePlan {
        batch_name,
        target_branch,
        worker_batch_names: worker_batch_names.into(),
        worker_partition_keys: worker_partition_keys.into(),
        worker_local_only_flags: worker_local_only_flags.into(),
        provenance_digest,
    }
}

fn normalize_intents_for_bulk_plan(
    intents: &mut [MutationIntent],
    client_key_symbol_policy: ClientKeySymbolPolicy,
    mut interner: StringInterner,
) {
    if !client_key_symbol_policy.interns_requested_strings() {
        return;
    }

    let mut raw_values = BTreeSet::new();
    for intent in intents.iter() {
        intent.collect_raw_client_keys(&mut raw_values);
    }
    for raw in raw_values {
        interner.intern(&raw);
    }
    for intent in intents {
        intent.normalize_client_keys(&mut interner, client_key_symbol_policy);
    }
}
