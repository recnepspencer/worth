use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, FieldKey, InternedString,
};
use worth_query_declaration::facade::application_schema::ApplicationRetainedEffectBinding;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, RelationId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    planned_single_field_locator, CreatedEntityRef, EntityReference,
};

use super::authoritative_truth::compile_committed_observations;
use super::model::{expected_alternate_model, expected_model, LoweringObservation};
use crate::domain_computation::primary_graph::application_attempt::provider_binding::{
    WorthQueryApplicationEmission, WorthQueryApplicationObservedFact,
    WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::tests::fixture::AccountActivityBinding;

pub(super) struct MixedEffectWorld {
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
    pub(super) effects: Vec<WorthQueryApplicationRealizedEffect>,
    pub(super) alternate_effects: Vec<WorthQueryApplicationRealizedEffect>,
    pub(super) expected: LoweringObservation,
    pub(super) alternate_expected: LoweringObservation,
    pub(super) retained_bytes: u64,
}

pub(super) struct MixedEffectAxes {
    pub(super) update_entity: EntityId,
    pub(super) deleted_entity: EntityId,
    pub(super) relation_from: EntityId,
    pub(super) relation_to: EntityId,
    pub(super) deleted_relation: RelationId,
    pub(super) created_kind: KindId,
    pub(super) second_created_kind: KindId,
    pub(super) relation_kind: KindId,
    pub(super) observed_relation_kind: KindId,
    pub(super) created_field: AspectFieldLocator,
    pub(super) alpha_field: AspectFieldLocator,
    pub(super) beta_field: AspectFieldLocator,
    pub(super) created_reference: EntityReference,
    pub(super) second_created_reference: EntityReference,
}

pub(super) fn mixed_effect_world() -> MixedEffectWorld {
    let created_field = field_locator("State", "Created");
    let alpha_field = field_locator("AlphaState", "Alpha");
    let beta_field = field_locator("BetaState", "Beta");
    let observed = compile_committed_observations(&alpha_field, &beta_field);
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
    let retained_bytes = AccountActivityBinding::retained_bytes(&audit_payload)
        + AccountActivityBinding::retained_bytes(&metric_payload);
    MixedEffectWorld {
        facts: observed.facts,
        effects: mixed_effects(&axes, &audit_payload, &metric_payload),
        alternate_effects: alternate_effects(&axes, audit_payload, metric_payload),
        expected: expected_model(&axes, retained_bytes),
        alternate_expected: expected_alternate_model(&axes, retained_bytes),
        retained_bytes,
    }
}

fn mixed_effects(
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

fn alternate_effects(
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

fn create_alpha(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: axes.created_kind,
        key: "created-alpha".to_owned(),
        fields: values([(&axes.created_field, "born")]),
    }
}

fn create_beta(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::CreateEntity {
        kind: axes.second_created_kind,
        key: "created-beta".to_owned(),
        fields: BTreeMap::new(),
    }
}

fn update(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::UpdateEntity {
        entity: "Account".to_owned(),
        entity_id: axes.update_entity,
        fields: values([(&axes.alpha_field, "one"), (&axes.beta_field, "two")]),
    }
}

fn create_relation(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::CreateRelation {
        kind: axes.relation_kind,
        key: "owned-edge".to_owned(),
        from: axes.created_reference.clone(),
        to: axes.second_created_reference.clone(),
    }
}

fn delete_relation(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::DeleteRelation {
        relation_id: axes.deleted_relation,
    }
}

fn delete_entity(axes: &MixedEffectAxes) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::DeleteEntity {
        entity_id: axes.deleted_entity,
    }
}

fn emission(effect: &'static str, payload: &str) -> WorthQueryApplicationRealizedEffect {
    WorthQueryApplicationRealizedEffect::Emit(WorthQueryApplicationEmission::new::<
        AccountActivityBinding,
    >(effect, payload.to_owned()))
}

pub(super) fn values<const N: usize>(
    entries: [(&AspectFieldLocator, &str); N],
) -> BTreeMap<AspectFieldLocator, AspectValue> {
    entries
        .into_iter()
        .map(|(locator, value)| (locator.clone(), string_value(value)))
        .collect()
}

fn field_locator(aspect: &str, field: &str) -> AspectFieldLocator {
    planned_single_field_locator(
        AspectKey::new(aspect).expect("valid fixture aspect"),
        FieldKey::new(field).expect("valid fixture field"),
    )
}

fn string_value(value: &str) -> AspectValue {
    AspectValue::String(InternedString::from(value.to_owned()))
}

fn created_reference(kind: KindId, key: &str) -> EntityReference {
    EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: kind,
        client_key: ClientKey::raw(key),
    })
}
