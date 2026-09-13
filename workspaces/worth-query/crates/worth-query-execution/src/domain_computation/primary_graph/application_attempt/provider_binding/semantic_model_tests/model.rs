use std::sync::Arc;

use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent,
    EntityReference, EntitySpec, MutationIntent, RelationMutationIntent, RelationSpec,
    UpdateEntityFieldsIntent,
};

use super::world::{values, MixedEffectAxes};
use crate::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryPreparedApplicationProviderAttempt;
use crate::domain_computation::{
    WorthQueryProvisionalEffectAction, WorthQueryProvisionalEffectStep,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LoweringObservation {
    steps: Vec<StepObservation>,
    intents: Vec<MutationIntent>,
    emissions: Vec<EmissionObservation>,
    retained_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StepObservation {
    family: String,
    action: WorthQueryProvisionalEffectAction,
    symbolic_dependencies: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EmissionObservation {
    effect: &'static str,
    payload: String,
}

pub(super) fn observe(
    prepared: WorthQueryPreparedApplicationProviderAttempt,
) -> LoweringObservation {
    let WorthQueryPreparedApplicationProviderAttempt { effects, .. } = prepared;
    let steps = effects.expected_steps().iter().map(observe_step).collect();
    let intents = effects.batch().intents.clone();
    let emissions = effects.into_emissions();
    let (emissions, retained_bytes) = emissions.into_parts();
    LoweringObservation {
        steps,
        intents,
        emissions: emissions
            .iter()
            .map(|emission| EmissionObservation {
                effect: emission.effect(),
                payload: emission
                    .payload::<String>()
                    .expect("fixture emission has String payload")
                    .clone(),
            })
            .collect(),
        retained_bytes,
    }
}

pub(super) fn assert_created_partition(
    prepared: WorthQueryPreparedApplicationProviderAttempt,
    expected: PartitionId,
) {
    for intent in &prepared.effects.batch().intents {
        match intent {
            MutationIntent::Create(CreateIntent::Entity(spec)) => {
                assert_eq!(spec.partition_id, expected);
            }
            MutationIntent::Create(CreateIntent::Relation(spec)) => {
                assert_eq!(spec.partition_id, expected);
                for endpoint in [&spec.source, &spec.target] {
                    if let EntityReference::Created(created) = endpoint {
                        assert_eq!(created.partition_id, expected);
                    }
                }
            }
            _ => {}
        }
    }
}

fn observe_step(step: &WorthQueryProvisionalEffectStep) -> StepObservation {
    assert!(step.artifact_dependencies().is_empty());
    assert!(step.proposal_basis().is_none());
    StepObservation {
        family: step.effect_family().to_owned(),
        action: step.action().clone(),
        symbolic_dependencies: step
            .symbolic_dependencies()
            .iter()
            .map(|identity| identity.to_string())
            .collect(),
    }
}

pub(super) fn expected_model(axes: &MixedEffectAxes, retained_bytes: u64) -> LoweringObservation {
    LoweringObservation {
        steps: vec![
            step(create("application-create-entity:10:created-alpha"), &[]),
            step(
                replace(&field_identity(axes.update_entity, &axes.alpha_field)),
                &[],
            ),
            step(
                replace(&field_identity(axes.update_entity, &axes.beta_field)),
                &[],
            ),
            step(create("application-create-entity:16:created-beta"), &[]),
            relation_create_step(),
            step(retire(&relation_identity(axes)), &[]),
            step(retire(&entity_identity(axes.deleted_entity, 12)), &[]),
        ],
        intents: vec![
            first_entity_create_intent(axes),
            update_intent(axes),
            second_entity_create_intent(axes),
            relation_create_intent(axes),
            delete_relation_intent(axes),
            delete_entity_intent(axes),
        ],
        emissions: vec![
            emission("AuditNotice", "sent"),
            emission("MetricNotice", "counted"),
        ],
        retained_bytes,
    }
}

pub(super) fn expected_alternate_model(
    axes: &MixedEffectAxes,
    retained_bytes: u64,
) -> LoweringObservation {
    LoweringObservation {
        steps: vec![
            step(retire(&entity_identity(axes.deleted_entity, 12)), &[]),
            relation_create_step(),
            step(create("application-create-entity:16:created-beta"), &[]),
            step(
                replace(&field_identity(axes.update_entity, &axes.alpha_field)),
                &[],
            ),
            step(
                replace(&field_identity(axes.update_entity, &axes.beta_field)),
                &[],
            ),
            step(retire(&relation_identity(axes)), &[]),
            step(create("application-create-entity:10:created-alpha"), &[]),
        ],
        intents: vec![
            delete_entity_intent(axes),
            relation_create_intent(axes),
            second_entity_create_intent(axes),
            update_intent(axes),
            delete_relation_intent(axes),
            first_entity_create_intent(axes),
        ],
        emissions: vec![
            emission("MetricNotice", "counted"),
            emission("AuditNotice", "sent"),
        ],
        retained_bytes,
    }
}

fn relation_create_step() -> StepObservation {
    step(
        create("application-create-relation:20:owned-edge"),
        &[
            "application-create-entity:10:created-alpha",
            "application-create-entity:16:created-beta",
        ],
    )
}

fn first_entity_create_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: axes.created_kind,
        client_key: ClientKey::raw("created-alpha"),
        fields: AspectFieldPatch::from(values([(&axes.created_field, "born")])),
    }))
}

fn second_entity_create_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: axes.second_created_kind,
        client_key: ClientKey::raw("created-beta"),
        fields: AspectFieldPatch::default(),
    }))
}

fn update_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id: axes.update_entity,
            fields: AspectFieldPatch::from(values([
                (&axes.alpha_field, "one"),
                (&axes.beta_field, "two"),
            ])),
        },
    ))
}

fn relation_create_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: axes.relation_kind,
        client_key: ClientKey::raw("owned-edge"),
        source: axes.created_reference.clone(),
        target: axes.second_created_reference.clone(),
        fields: AspectFieldPatch::default(),
    }))
}

fn delete_relation_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id: axes.deleted_relation,
    }))
}

fn delete_entity_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
        entity_id: axes.deleted_entity,
    }))
}

fn emission(effect: &'static str, payload: &str) -> EmissionObservation {
    EmissionObservation {
        effect,
        payload: payload.to_owned(),
    }
}

fn step(action: WorthQueryProvisionalEffectAction, dependencies: &[&str]) -> StepObservation {
    StepObservation {
        family: "mutation".to_owned(),
        action,
        symbolic_dependencies: dependencies
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}

fn create(identity: &str) -> WorthQueryProvisionalEffectAction {
    WorthQueryProvisionalEffectAction::Create {
        symbolic_identity: Arc::from(identity),
    }
}

fn replace(identity: &str) -> WorthQueryProvisionalEffectAction {
    WorthQueryProvisionalEffectAction::Replace {
        target_identity: Arc::from(identity),
    }
}

fn retire(identity: &str) -> WorthQueryProvisionalEffectAction {
    WorthQueryProvisionalEffectAction::Retire {
        target_identity: Arc::from(identity),
    }
}

fn field_identity(
    entity: EntityId,
    locator: &worth_foundational::facade::AspectFieldLocator,
) -> String {
    format!(
        "application-field:{}:{}:{}:{}/{}",
        entity.partition_value(),
        entity.local_slot_value(),
        entity.generation_value(),
        locator.aspect().aspect_key().as_str(),
        locator
            .field_path()
            .fields()
            .first()
            .expect("fixture locator has one field")
            .as_str(),
    )
}

fn entity_identity(entity: EntityId, kind: u32) -> String {
    format!(
        "application-entity:{}:{}:{}:kind:{kind}",
        entity.partition_value(),
        entity.local_slot_value(),
        entity.generation_value(),
    )
}

fn relation_identity(axes: &MixedEffectAxes) -> String {
    format!(
        "application-relation:{}:{}:{}->{}:{}:{}:kind:{}",
        axes.relation_from.partition_value(),
        axes.relation_from.local_slot_value(),
        axes.relation_from.generation_value(),
        axes.relation_to.partition_value(),
        axes.relation_to.local_slot_value(),
        axes.relation_to.generation_value(),
        axes.observed_relation_kind.as_u32(),
    )
}
