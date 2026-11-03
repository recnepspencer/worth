use worth_foundational::facade::{aspects, AspectFieldLocator, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::schema::{
    AspectBinding, DeclaredAspectContractBinding, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};

use super::{
    planned_field_locator, register_entity, valid_aspect_key, valid_field_key,
    WorthQueryPrimaryGraphInstallationDenial,
};

const ENTITY: &str = "worth-query-provider-idempotency";
const ASPECT: &str = "idempotency";
const KEY_FIELD: &str = "key";
const INTENT_FIELD: &str = "intent";
const OUTCOME_IDENTITY_FIELD: &str = "outcome-identity";
const EMITTED_EFFECT_COUNT_FIELD: &str = "emitted-effect-count";

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryProviderIdempotencyLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) key_locator: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) intent_locator: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) outcome_identity_locator: AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) emitted_effect_count_locator:
        AspectFieldLocator,
    pub(in crate::domain_computation::primary_graph) key_index_id: DerivedIndexId,
}

pub(super) fn lower_provider_idempotency(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    schema_version_id: SchemaVersionId,
    entity_kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (
        RelationalSchemaRegistry,
        WorthQueryProviderIdempotencyLayout,
    ),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let key_locator = planned_field_locator(ASPECT, KEY_FIELD)?;
    let intent_locator = planned_field_locator(ASPECT, INTENT_FIELD)?;
    let outcome_identity_locator = planned_field_locator(ASPECT, OUTCOME_IDENTITY_FIELD)?;
    let emitted_effect_count_locator = planned_field_locator(ASPECT, EMITTED_EFFECT_COUNT_FIELD)?;
    let shape = aspects()
        .struct_fields()
        .required(KEY_FIELD, ScalarAspectType::String)
        .required(INTENT_FIELD, ScalarAspectType::String)
        .required(OUTCOME_IDENTITY_FIELD, ScalarAspectType::UInt64)
        .required(EMITTED_EFFECT_COUNT_FIELD, ScalarAspectType::UInt64)
        .finish()
        .map_err(|_| super::invalid_member(ASPECT))?;
    let contract = aspects()
        .contract()
        .for_key(valid_aspect_key(ASPECT)?)
        .identified_by(identity)
        .at_revision(aspects().vocabulary().revision(3))
        .struct_aspect(shape);
    let registry = register_entity(
        registry,
        schema_id,
        schema_version_id,
        ENTITY,
        entity_kind,
        vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: valid_field_key(ASPECT)?,
            },
            contract,
        }],
    )?;
    Ok((
        registry,
        WorthQueryProviderIdempotencyLayout {
            entity_kind,
            key_locator,
            intent_locator,
            outcome_identity_locator,
            emitted_effect_count_locator,
            key_index_id: DerivedIndexId(0),
        },
    ))
}
