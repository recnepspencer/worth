use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{
    AspectBinding, DeclaredAspectContractBinding, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};

use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxLayout;

use super::{
    planned_field_locator, register_entity, valid_aspect_key, valid_field_key,
    WorthQueryPrimaryGraphInstallationDenial,
};

const ENTITY: &str = "worth-query-dispatch-outbox";
const ASPECT: &str = "dispatch-outbox";
const CORRELATION_FIELD: &str = "correlation";
const FAMILY_FIELD: &str = "correlation-family";
const EFFECT_FIELD: &str = "effect";
const PROTOCOL_IDENTITY_FIELD: &str = "protocol-identity";
const PROTOCOL_VERSION_FIELD: &str = "protocol-version";
const MAXIMUM_PAYLOAD_BYTES_FIELD: &str = "maximum-payload-bytes";
const PAYLOAD_FIELD: &str = "payload-hex";
const OUTCOME_IDENTITY_FIELD: &str = "outcome-identity";
const OPERATION_SLOT_FIELD: &str = "operation-slot";

/// Lowers the Query-owned dispatch-outbox entity into the application's own
/// Relational schema so a declared external effect co-commits its dispatch
/// record inside the operation's single transaction.
pub(super) fn lower_provider_dispatch_outbox(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    schema_version_id: SchemaVersionId,
    entity_kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorthQueryDispatchOutboxLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let correlation_locator = planned_field_locator(ASPECT, CORRELATION_FIELD)?;
    let family_locator = planned_field_locator(ASPECT, FAMILY_FIELD)?;
    let effect_locator = planned_field_locator(ASPECT, EFFECT_FIELD)?;
    let protocol_identity_locator = planned_field_locator(ASPECT, PROTOCOL_IDENTITY_FIELD)?;
    let protocol_version_locator = planned_field_locator(ASPECT, PROTOCOL_VERSION_FIELD)?;
    let maximum_payload_bytes_locator = planned_field_locator(ASPECT, MAXIMUM_PAYLOAD_BYTES_FIELD)?;
    let payload_locator = planned_field_locator(ASPECT, PAYLOAD_FIELD)?;
    let outcome_identity_locator = planned_field_locator(ASPECT, OUTCOME_IDENTITY_FIELD)?;
    let operation_slot_locator = planned_field_locator(ASPECT, OPERATION_SLOT_FIELD)?;
    let shape = aspects()
        .struct_fields()
        .required(CORRELATION_FIELD, ScalarAspectType::String)
        .required(FAMILY_FIELD, ScalarAspectType::String)
        .required(EFFECT_FIELD, ScalarAspectType::String)
        .required(PROTOCOL_IDENTITY_FIELD, ScalarAspectType::String)
        .required(PROTOCOL_VERSION_FIELD, ScalarAspectType::UInt64)
        .required(MAXIMUM_PAYLOAD_BYTES_FIELD, ScalarAspectType::UInt64)
        .required(PAYLOAD_FIELD, ScalarAspectType::String)
        .required(OUTCOME_IDENTITY_FIELD, ScalarAspectType::UInt64)
        .required(OPERATION_SLOT_FIELD, ScalarAspectType::String)
        .finish()
        .map_err(|_| super::invalid_member(ASPECT))?;
    let contract = aspects()
        .contract()
        .for_key(valid_aspect_key(ASPECT)?)
        .identified_by(identity)
        .at_revision(aspects().vocabulary().revision(2))
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
        WorthQueryDispatchOutboxLayout {
            entity_kind,
            correlation_locator,
            family_locator,
            effect_locator,
            protocol_identity_locator,
            protocol_version_locator,
            maximum_payload_bytes_locator,
            payload_locator,
            outcome_identity_locator,
            operation_slot_locator,
        },
    ))
}
