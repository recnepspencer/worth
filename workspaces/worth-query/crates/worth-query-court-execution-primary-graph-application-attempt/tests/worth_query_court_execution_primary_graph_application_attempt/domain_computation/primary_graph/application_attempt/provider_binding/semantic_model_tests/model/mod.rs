use std::sync::Arc;

use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::DeleteEntityIntent;
use worth_relational::facade::transactions::DeleteRelationIntent;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::EntityReference;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationMutationIntent;
use worth_relational::facade::transactions::RelationSpec;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;

use crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::world::values;
use crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::world::MixedEffectAxes;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_binding::WorthQueryPreparedApplicationProviderAttempt;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalEffectAction;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalEffectStep;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoweringObservation {
    pub steps: Vec<StepObservation>,
    pub intents: Vec<MutationIntent>,
    pub emissions: Vec<EmissionObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepObservation {
    pub family: String,
    pub action: WorthQueryProvisionalEffectAction,
    pub symbolic_dependencies: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmissionObservation {
    pub effect: &'static str,
    pub payload: String,
}

pub fn observe(prepared: WorthQueryPreparedApplicationProviderAttempt) -> LoweringObservation {
    let (_workflow_origin_rest, _local_action_settlement_rest, _required_output_demand_rest, _installed_read_scopes_rest, _facts_rest, _application_projection_rest, _original_read_rest, _consumed_outputs_rest, effects, _preimage_demand_rest, _conditional_definition_rest, _retain_output_demand_observation_rest, _retain_client_observation_rest, _producer_required_invariants_rest, _output_currentness_facts_rest,) = prepared.into_parts_for_test();
    let steps = effects.expected_steps_for_test().iter().map(observe_step).collect();
    let intents = effects.batch_for_test().intents.clone();
    let emissions = effects.into_emissions_for_test();
    let emissions = emissions.into_emissions();
    LoweringObservation {
        steps,
        intents,
        emissions: emissions
            .iter()
            .map(|emission| EmissionObservation {
                effect: emission.effect_for_test(),
                payload: emission
                    .payload_for_test::<String>()
                    .expect("fixture emission has String payload")
                    .clone(),
            })
            .collect(),
    }
}

pub fn assert_created_partition(
    prepared: WorthQueryPreparedApplicationProviderAttempt,
    expected: PartitionId,
) {
    for intent in &(*prepared.effects_for_test()).batch_for_test().intents {
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

pub fn observe_step(step: &WorthQueryProvisionalEffectStep) -> StepObservation {
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

pub fn expected_model(axes: &MixedEffectAxes) -> LoweringObservation {
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
    }
}

pub fn expected_alternate_model(axes: &MixedEffectAxes) -> LoweringObservation {
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
    }
}

pub fn relation_create_step() -> StepObservation {
    step(
        create("application-create-relation:20:owned-edge"),
        &[
            "application-create-entity:10:created-alpha",
            "application-create-entity:16:created-beta",
        ],
    )
}

pub fn first_entity_create_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: axes.created_kind,
        client_key: ClientKey::raw("created-alpha"),
        fields: AspectFieldPatch::from(values([(&axes.created_field, "born")])),
    }))
}

pub fn second_entity_create_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: axes.second_created_kind,
        client_key: ClientKey::raw("created-beta"),
        fields: AspectFieldPatch::default(),
    }))
}

pub fn update_intent(axes: &MixedEffectAxes) -> MutationIntent {
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

pub fn relation_create_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: axes.relation_kind,
        client_key: ClientKey::raw("owned-edge"),
        source: axes.created_reference.clone(),
        target: axes.second_created_reference.clone(),
        fields: AspectFieldPatch::default(),
    }))
}

pub fn delete_relation_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id: axes.deleted_relation,
    }))
}

pub fn delete_entity_intent(axes: &MixedEffectAxes) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
        entity_id: axes.deleted_entity,
    }))
}

pub fn emission(effect: &'static str, payload: &str) -> EmissionObservation {
    EmissionObservation {
        effect,
        payload: payload.to_owned(),
    }
}

pub fn step(action: WorthQueryProvisionalEffectAction, dependencies: &[&str]) -> StepObservation {
    StepObservation {
        family: "mutation".to_owned(),
        action,
        symbolic_dependencies: dependencies
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}

pub fn create(identity: &str) -> WorthQueryProvisionalEffectAction {
    WorthQueryProvisionalEffectAction::Create {
        symbolic_identity: Arc::from(identity),
    }
}

pub fn replace(identity: &str) -> WorthQueryProvisionalEffectAction {
    WorthQueryProvisionalEffectAction::Replace {
        target_identity: Arc::from(identity),
    }
}

pub fn retire(identity: &str) -> WorthQueryProvisionalEffectAction {
    WorthQueryProvisionalEffectAction::Retire {
        target_identity: Arc::from(identity),
    }
}

pub fn field_identity(
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

pub fn entity_identity(entity: EntityId, kind: u32) -> String {
    format!(
        "application-entity:{}:{}:{}:kind:{kind}",
        entity.partition_value(),
        entity.local_slot_value(),
        entity.generation_value(),
    )
}

pub fn relation_identity(axes: &MixedEffectAxes) -> String {
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
