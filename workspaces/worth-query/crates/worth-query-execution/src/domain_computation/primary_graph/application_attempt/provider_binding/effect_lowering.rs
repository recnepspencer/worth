use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, DeleteEntityIntent, DeleteRelationIntent,
    EntityMutationIntent, EntityReference, EntitySpec, MutationIntent, RelationMutationIntent,
    RelationSpec, UpdateEntityFieldsIntent,
};

use super::{
    progression_denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationEmission,
    WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::{
    WorthQueryProvisionalEffectAction, WorthQueryProvisionalEffectStep,
};

mod observed_fact_index;
mod optional_field_patch;
pub(super) use observed_fact_index::ObservedFactIndex;

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

pub(super) struct CreatedEffectReferences {
    resolved: BTreeMap<EntityReference, EntityReference>,
    symbols: BTreeMap<EntityReference, Arc<str>>,
}

impl CreatedEffectReferences {
    pub(super) fn new(
        effects: &[WorthQueryApplicationRealizedEffect],
        mutation_partition: worth_relational::facade::identity::PartitionId,
    ) -> Self {
        let mut resolved = BTreeMap::new();
        let mut symbols = BTreeMap::new();
        for effect in effects {
            let WorthQueryApplicationRealizedEffect::CreateEntity {
                kind,
                key,
                partition,
                ..
            } = effect
            else {
                continue;
            };
            let reference = |partition_id| {
                EntityReference::Created(CreatedEntityRef {
                    partition_id,
                    kind_id: *kind,
                    client_key: worth_relational::facade::symbols::ClientKey::raw(key.clone()),
                })
            };
            let authored = reference(
                partition.resolve(worth_relational::facade::identity::PartitionId::main()),
            );
            let lowered = reference(partition.resolve(mutation_partition));
            resolved.insert(authored, lowered.clone());
            symbols.insert(lowered, created_entity_symbol(*kind, key));
        }
        Self { resolved, symbols }
    }

    fn lower(
        &self,
        reference: EntityReference,
    ) -> Result<EntityReference, WorthQueryApplicationAttemptDenial> {
        match reference {
            EntityReference::Created(_) => self
                .resolved
                .get(&reference)
                .cloned()
                .ok_or_else(progression_denial),
            existing => Ok(existing),
        }
    }
}

pub(super) fn lower_provider_effect(
    facts: &ObservedFactIndex<'_>,
    created: &CreatedEffectReferences,
    mutation_partition: worth_relational::facade::identity::PartitionId,
    effect: WorthQueryApplicationRealizedEffect,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    match effect {
        WorthQueryApplicationRealizedEffect::CreateEntity {
            kind,
            key,
            fields,
            partition,
        } => lower_create_entity(partition.resolve(mutation_partition), kind, key, fields),
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
            created,
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
    facts: &ObservedFactIndex<'_>,
    entity_id: EntityId,
    fields: BTreeMap<AspectFieldLocator, AspectValue>,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    let steps = fields
        .keys()
        .map(|locator| {
            let target = facts.field_identity(entity_id, locator)?;
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
    facts: &ObservedFactIndex<'_>,
    entity_id: EntityId,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    mutation(
        vec![effect_step(WorthQueryProvisionalEffectAction::Retire {
            target_identity: facts.entity_identity(entity_id)?.into(),
        })?],
        MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
            entity_id,
        })),
    )
}

fn lower_create_relation(
    created: &CreatedEffectReferences,
    mutation_partition: worth_relational::facade::identity::PartitionId,
    effect: WorthQueryCreateRelationEffect,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    let from = created.lower(effect.from)?;
    let to = created.lower(effect.to)?;
    let dependencies = [&from, &to]
        .into_iter()
        .filter_map(|reference| created.symbols.get(reference).cloned());
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
            partition_id: if from.partition_id() == to.partition_id()
                && (matches!(&from, EntityReference::Created(_))
                    || matches!(&to, EntityReference::Created(_)))
            {
                from.partition_id()
            } else {
                mutation_partition
            },
            kind_id: effect.kind,
            client_key: worth_relational::facade::symbols::ClientKey::raw(effect.key),
            source: from,
            target: to,
            fields: AspectFieldPatch::default(),
        })),
    )
}

fn lower_delete_relation(
    facts: &ObservedFactIndex<'_>,
    relation_id: RelationId,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    mutation(
        vec![effect_step(WorthQueryProvisionalEffectAction::Retire {
            target_identity: facts.relation_identity(relation_id)?.into(),
        })?],
        MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
            relation_id,
        })),
    )
}

fn mutation(
    steps: Vec<WorthQueryProvisionalEffectStep>,
    intent: MutationIntent,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    Ok(WorthQueryLoweredProviderEffect::Mutation { steps, intent })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCreationPartition;
    use worth_relational::facade::identity::{KindId, PartitionId};

    #[test]
    fn appended_workflow_facts_keep_their_context_after_an_issued_application_create() {
        let application_partition = PartitionId::new(3);
        let workflow_partition = PartitionId::new(7);
        let created = |partition_id, kind_id, key: &str| {
            EntityReference::Created(CreatedEntityRef {
                partition_id,
                kind_id: KindId::new(kind_id),
                client_key: worth_relational::facade::symbols::ClientKey::raw(key),
            })
        };
        let effects = vec![
            WorthQueryApplicationRealizedEffect::CreateEntity {
                kind: KindId::new(10),
                key: "cad-feature".to_owned(),
                fields: BTreeMap::new(),
                partition: WorthQueryApplicationCreationPartition::Issued,
            },
            WorthQueryApplicationRealizedEffect::CreateEntity {
                kind: KindId::new(11),
                key: "transition".to_owned(),
                fields: BTreeMap::new(),
                partition: WorthQueryApplicationCreationPartition::Context(workflow_partition),
            },
            WorthQueryApplicationRealizedEffect::CreateRelation {
                kind: KindId::new(12),
                key: "instance-transition".to_owned(),
                from: EntityReference::Existing(EntityId::new(workflow_partition, 1, 1)),
                to: created(workflow_partition, 11, "transition"),
            },
            WorthQueryApplicationRealizedEffect::CreateRelation {
                kind: KindId::new(13),
                key: "cad-membership".to_owned(),
                from: EntityReference::Existing(EntityId::new(application_partition, 2, 1)),
                to: created(PartitionId::main(), 10, "cad-feature"),
            },
        ];
        let references = CreatedEffectReferences::new(&effects, application_partition);
        let facts = ObservedFactIndex::new(&[]);
        let mut lowered = effects.into_iter().map(|effect| {
            lower_provider_effect(&facts, &references, application_partition, effect).unwrap()
        });
        let entity_partition = |lowered| match lowered {
            WorthQueryLoweredProviderEffect::Mutation {
                intent: MutationIntent::Create(CreateIntent::Entity(entity)),
                ..
            } => entity.partition_id,
            _ => panic!("expected an entity create"),
        };
        assert_eq!(
            entity_partition(lowered.next().unwrap()),
            application_partition
        );
        assert_eq!(
            entity_partition(lowered.next().unwrap()),
            workflow_partition
        );
        let relation = |lowered| match lowered {
            WorthQueryLoweredProviderEffect::Mutation {
                intent: MutationIntent::Create(CreateIntent::Relation(relation)),
                ..
            } => relation,
            _ => panic!("expected a relation create"),
        };
        let workflow_relation = relation(lowered.next().unwrap());
        assert_eq!(workflow_relation.partition_id, workflow_partition);
        assert_eq!(workflow_relation.target.partition_id(), workflow_partition);
        let cad_relation = relation(lowered.next().unwrap());
        assert_eq!(cad_relation.partition_id, application_partition);
        assert_eq!(cad_relation.target.partition_id(), application_partition);
    }
}

fn created_entity_symbol(kind: worth_relational::facade::identity::KindId, key: &str) -> Arc<str> {
    Arc::from(format!("application-create-entity:{}:{key}", kind.as_u32()))
}

fn effect_step(
    action: WorthQueryProvisionalEffectAction,
) -> Result<WorthQueryProvisionalEffectStep, WorthQueryApplicationAttemptDenial> {
    WorthQueryProvisionalEffectStep::new("mutation", action).map_err(|_| progression_denial())
}
