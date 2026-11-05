use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::InternedString;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::identity::RelationId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::planned_single_field_locator;
use worth_relational::facade::transactions::CreatedEntityRef;
use worth_relational::facade::transactions::EntityReference;

use crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::authoritative_truth::compile_committed_observations;
use crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::model::expected_alternate_model;
use crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::model::expected_model;
use crate::domain_computation::primary_graph::application_attempt::provider_binding::semantic_model_tests::model::LoweringObservation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::emission::WorthQueryApplicationEmission;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationRealizedEffect;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountActivityBinding;

pub struct MixedEffectWorld {
    pub facts: Vec<WorthQueryApplicationObservedFact>,
    pub effects: Vec<WorthQueryApplicationRealizedEffect>,
    pub alternate_effects: Vec<WorthQueryApplicationRealizedEffect>,
    pub expected: LoweringObservation,
    pub alternate_expected: LoweringObservation,
}

pub struct MixedEffectAxes {
    pub update_entity: EntityId,
    pub deleted_entity: EntityId,
    pub relation_from: EntityId,
    pub relation_to: EntityId,
    pub deleted_relation: RelationId,
    pub created_kind: KindId,
    pub second_created_kind: KindId,
    pub relation_kind: KindId,
    pub observed_relation_kind: KindId,
    pub created_field: AspectFieldLocator,
    pub alpha_field: AspectFieldLocator,
    pub beta_field: AspectFieldLocator,
    pub created_reference: EntityReference,
    pub second_created_reference: EntityReference,
}

pub fn mixed_effect_world(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> MixedEffectWorld {
    let created_field = field_locator("State", "Created");
    let alpha_field = field_locator("AlphaState", "Alpha");
    let beta_field = field_locator("BetaState", "Beta");
    let observed = compile_committed_observations(execution, &alpha_field, &beta_field);
    let axes = MixedEffectAxes {
        update_entity: observed.update_entity,
        deleted_entity: observed.deleted_entity,
        relation_from: observed.relation_from,
        relation_to: observed.relation_to,
        deleted_relation: observed.deleted_relation,
        created_kind: KindId::new(10),
        second_created_kind: KindId::new(16),
        relation_kind: KindId::new(20),
        observed_relation_kind: KindId::new(31),
        created_field,
        alpha_field,
        beta_field,
        created_reference: created_reference(KindId::new(10), "created-alpha"),
        second_created_reference: created_reference(KindId::new(16), "created-beta"),
    };
    let audit_payload = "sent".to_owned();
    let metric_payload = "counted".to_owned();
    MixedEffectWorld {
        facts: observed.facts,
        effects: mixed_effects(&axes, &audit_payload, &metric_payload),
        alternate_effects: alternate_effects(&axes, audit_payload, metric_payload),
        expected: expected_model(&axes),
        alternate_expected: expected_alternate_model(&axes),
    }
}

pub fn mixed_effects(
    axes: &MixedEffectAxes,
    audit_payload: &str,
    metric_payload: &str,
) -> Vec<WorthQueryApplicationRealizedEffect> {
    vec![
        create_alpha(axes),
        emission("AuditNotice", audit_payload),
        update(axes),
        create_beta(axes),
        create_relation(axes),
        emission("MetricNotice", metric_payload),
        delete_relation(axes),
        delete_entity(axes),
    ]
}

pub fn alternate_effects(
    axes: &MixedEffectAxes,
    audit_payload: String,
    metric_payload: String,
) -> Vec<WorthQueryApplicationRealizedEffect> {
    vec![
        emission("MetricNotice", &metric_payload),
        delete_entity(axes),
        create_relation(axes),
        create_beta(axes),
        update(axes),
        emission("AuditNotice", &audit_payload),
        delete_relation(axes),
        create_alpha(axes),
    ]
}

pub fn create_alpha(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: axes.created_kind,
        key: "created-alpha".to_owned(),
        fields: values([(&axes.created_field, "born")]),
        partition:
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationCreationPartition::Issued,
    }
}

pub fn create_beta(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: axes.second_created_kind,
        key: "created-beta".to_owned(),
        fields: BTreeMap::new(),
        partition:
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationCreationPartition::Issued,
    }
}

pub fn update(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::UpdateEntity {
        entity: "Account".to_owned(),
        entity_id: axes.update_entity,
        fields: values([(&axes.alpha_field, "one"), (&axes.beta_field, "two")]),
    }
}

pub fn create_relation(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::CreateRelation {
        fields: Default::default(),
        kind: axes.relation_kind,
        key: "owned-edge".to_owned(),
        from: axes.created_reference.clone(),
        to: axes.second_created_reference.clone(),
    }
}

pub fn delete_relation(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::DeleteRelation {
        relation_id: axes.deleted_relation,
    }
}

pub fn delete_entity(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::DeleteEntity {
        entity_id: axes.deleted_entity,
    }
}

pub fn emission(effect: &'static str, payload: &str) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::Emit(WorthQueryApplicationEmission::new::<
        AccountActivityBinding,
    >(effect, payload.to_owned()))
}

pub fn values<const N: usize>(
    entries: [(&AspectFieldLocator, &str); N],
) -> BTreeMap<AspectFieldLocator, AspectValue> {
    entries
        .into_iter()
        .map(|(locator, value)| (locator.clone(), string_value(value)))
        .collect()
}

pub fn field_locator(aspect: &str, field: &str) -> AspectFieldLocator {
    planned_single_field_locator(
        AspectKey::new(aspect).expect("valid fixture aspect"),
        FieldKey::new(field).expect("valid fixture field"),
    )
}

pub fn string_value(value: &str) -> AspectValue {
    AspectValue::String(InternedString::from(value.to_owned()))
}

pub fn created_reference(kind: KindId, key: &str) -> EntityReference {
    EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: kind,
        client_key: ClientKey::raw(key),
    })
}
