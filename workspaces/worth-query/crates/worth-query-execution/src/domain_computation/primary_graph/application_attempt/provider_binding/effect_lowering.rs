use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent,
    EntityReference, EntitySpec, MutationIntent, RelationMutationIntent, RelationSpec,
    UpdateEntityFieldsIntent,
};

use super::{
    progression_denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationEmission,
    WorthQueryApplicationObservedFact, WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::{
    WorthQueryProvisionalEffectAction, WorthQueryProvisionalEffectStep,
};

mod optional_field_patch;

#[derive(Clone)]
pub(super) enum WorthQueryLoweredProviderEffect {
    Mutation {
        steps: Vec<WorthQueryProvisionalEffectStep>,
        intent: MutationIntent,
    },
    Emission(WorthQueryApplicationEmission),
}

struct WorthQueryCreateRelationEffect {
    kind: worth_relational::facade::identity::KindId,
    key: String,
    from: EntityReference,
    to: EntityReference,
}

pub(super) fn lower_provider_effect(
    facts: &[WorthQueryApplicationObservedFact],
    symbols: &BTreeMap<EntityReference, Arc<str>>,
    mutation_partition: worth_relational::facade::identity::PartitionId,
    effect: WorthQueryApplicationRealizedEffect,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    match effect {
        WorthQueryApplicationRealizedEffect::CreateEntity {
            kind, key, fields, ..
        } => lower_create_entity(mutation_partition, kind, key, fields),
        WorthQueryApplicationRealizedEffect::UpdateEntity {
            entity_id, fields, ..
        } => lower_update_entity(facts, entity_id, fields),
        WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
            entity_id,
            fields,
            ..
        } => optional_field_patch::lower(facts, entity_id, fields),
        WorthQueryApplicationRealizedEffect::DeleteEntity { entity_id } => {
            lower_delete_entity(facts, entity_id)
        }
        WorthQueryApplicationRealizedEffect::CreateRelation {
            kind,
            key,
            from,
            to,
        } => lower_create_relation(
            symbols,
            mutation_partition,
            WorthQueryCreateRelationEffect {
                kind,
                key,
                from,
                to,
            },
        ),
        WorthQueryApplicationRealizedEffect::DeleteRelation { relation_id } => {
            lower_delete_relation(facts, relation_id)
        }
        WorthQueryApplicationRealizedEffect::Emit(emission) => {
            Ok(WorthQueryLoweredProviderEffect::Emission(emission))
        }
    }
}

fn lower_create_entity(
    mutation_partition: worth_relational::facade::identity::PartitionId,
    kind: worth_relational::facade::identity::KindId,
    key: String,
    fields: BTreeMap<AspectFieldLocator, AspectValue>,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    mutation(
        vec![effect_step(WorthQueryProvisionalEffectAction::Create {
            symbolic_identity: created_entity_symbol(kind, &key),
        })?],
        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: mutation_partition,
            kind_id: kind,
            client_key: worth_relational::facade::symbols::ClientKey::raw(key),
            fields: AspectFieldPatch::from(fields),
        })),
    )
}

fn lower_update_entity(
    facts: &[WorthQueryApplicationObservedFact],
    entity_id: EntityId,
    fields: BTreeMap<AspectFieldLocator, AspectValue>,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    let steps = fields
        .keys()
        .map(|locator| {
            let target = field_fact_identity(facts, entity_id, locator)?;
            effect_step(WorthQueryProvisionalEffectAction::Replace {
                target_identity: target.into(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    mutation(
        steps,
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id,
                fields: AspectFieldPatch::from(fields),
            },
        )),
    )
}

fn lower_delete_entity(
    facts: &[WorthQueryApplicationObservedFact],
    entity_id: EntityId,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    mutation(
        vec![effect_step(WorthQueryProvisionalEffectAction::Retire {
            target_identity: entity_fact_identity(facts, entity_id)?.into(),
        })?],
        MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
            entity_id,
        })),
    )
}

fn lower_create_relation(
    symbols: &BTreeMap<EntityReference, Arc<str>>,
    mutation_partition: worth_relational::facade::identity::PartitionId,
    effect: WorthQueryCreateRelationEffect,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    let from = remap_created_reference(effect.from, mutation_partition);
    let to = remap_created_reference(effect.to, mutation_partition);
    let dependencies = [&from, &to]
        .into_iter()
        .filter_map(|reference| symbols.get(reference).cloned());
    let step = effect_step(WorthQueryProvisionalEffectAction::Create {
        symbolic_identity: format!(
            "application-create-relation:{}:{}",
            effect.kind.as_u32(),
            effect.key
        )
        .into(),
    })?
    .with_symbolic_dependencies(dependencies)
    .map_err(|_| progression_denial())?;
    mutation(
        vec![step],
        MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: mutation_partition,
            kind_id: effect.kind,
            client_key: worth_relational::facade::symbols::ClientKey::raw(effect.key),
            source: from,
            target: to,
            fields: AspectFieldPatch::default(),
        })),
    )
}

fn lower_delete_relation(
    facts: &[WorthQueryApplicationObservedFact],
    relation_id: RelationId,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    mutation(
        vec![effect_step(WorthQueryProvisionalEffectAction::Retire {
            target_identity: relation_fact_identity(facts, relation_id)?.into(),
        })?],
        MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
            relation_id,
        })),
    )
}

pub(super) fn created_entity_symbols(
    effects: &[WorthQueryApplicationRealizedEffect],
    mutation_partition: worth_relational::facade::identity::PartitionId,
) -> BTreeMap<EntityReference, Arc<str>> {
    effects
        .iter()
        .filter_map(|effect| {
            let WorthQueryApplicationRealizedEffect::CreateEntity { kind, key, .. } = effect else {
                return None;
            };
            let reference = EntityReference::Created(
                worth_relational::facade::transactions::CreatedEntityRef {
                    partition_id: mutation_partition,
                    kind_id: *kind,
                    client_key: worth_relational::facade::symbols::ClientKey::raw(key.clone()),
                },
            );
            Some((reference, created_entity_symbol(*kind, key)))
        })
        .collect()
}

fn remap_created_reference(
    reference: EntityReference,
    mutation_partition: worth_relational::facade::identity::PartitionId,
) -> EntityReference {
    match reference {
        EntityReference::Created(mut created) => {
            created.partition_id = mutation_partition;
            EntityReference::Created(created)
        }
        existing => existing,
    }
}

fn mutation(
    steps: Vec<WorthQueryProvisionalEffectStep>,
    intent: MutationIntent,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    Ok(WorthQueryLoweredProviderEffect::Mutation { steps, intent })
}

fn created_entity_symbol(kind: worth_relational::facade::identity::KindId, key: &str) -> Arc<str> {
    Arc::from(format!("application-create-entity:{}:{key}", kind.as_u32()))
}

fn effect_step(
    action: WorthQueryProvisionalEffectAction,
) -> Result<WorthQueryProvisionalEffectStep, WorthQueryApplicationAttemptDenial> {
    WorthQueryProvisionalEffectStep::new("mutation", action).map_err(|_| progression_denial())
}

fn field_fact_identity(
    facts: &[WorthQueryApplicationObservedFact],
    entity_id: EntityId,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    facts
        .iter()
        .find(|fact| {
            matches!(
                fact,
                WorthQueryApplicationObservedFact::Field {
                    entity_id: observed,
                    locator: observed_locator,
                    ..
                }
                | WorthQueryApplicationObservedFact::AbsentField {
                    entity_id: observed,
                    locator: observed_locator,
                    ..
                } if *observed == entity_id && observed_locator == locator
            )
        })
        .map(WorthQueryApplicationObservedFact::locator_identity)
        .ok_or_else(progression_denial)
}

fn entity_fact_identity(
    facts: &[WorthQueryApplicationObservedFact],
    entity_id: EntityId,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    facts
        .iter()
        .find(|fact| {
            matches!(
                fact,
                WorthQueryApplicationObservedFact::Entity {
                    entity_id: observed,
                    ..
                } if *observed == entity_id
            )
        })
        .map(WorthQueryApplicationObservedFact::locator_identity)
        .ok_or_else(progression_denial)
}

fn relation_fact_identity(
    facts: &[WorthQueryApplicationObservedFact],
    relation_id: RelationId,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    facts
        .iter()
        .find(|fact| match fact {
            WorthQueryApplicationObservedFact::Relation {
                matching_relations, ..
            } => matching_relations.contains(&relation_id),
            WorthQueryApplicationObservedFact::Adjacency { relations, .. } => relations
                .iter()
                .any(|relation| relation.relation_id == relation_id),
            _ => false,
        })
        .map(WorthQueryApplicationObservedFact::locator_identity)
        .ok_or_else(progression_denial)
}
