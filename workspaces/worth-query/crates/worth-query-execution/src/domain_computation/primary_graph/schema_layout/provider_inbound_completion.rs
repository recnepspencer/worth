//! Query-owned Relational record co-committing occurrence consumption and terminal effect posture.

use worth_foundational::facade::{aspects, AspectFieldLocator, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{
    AspectBinding, DeclaredAspectContractBinding, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};

use super::{
    invalid_member, planned_field_locator, register_entity, valid_aspect_key, valid_field_key,
    WorthQueryPrimaryGraphInstallationDenial,
};

const ENTITY: &str = "worth-query-inbound-completion";
const ASPECT: &str = "inbound-completion";

#[derive(Clone, Debug)]
pub struct WorthQueryInboundCompletionLayout {
    pub kind: KindId,
    pub correlation: AspectFieldLocator,
    pub family: AspectFieldLocator,
    pub operation: AspectFieldLocator,
    pub audience: AspectFieldLocator,
    pub source: AspectFieldLocator,
    pub expires_at: AspectFieldLocator,
    pub key_epoch: AspectFieldLocator,
    pub message: AspectFieldLocator,
    pub protocol: AspectFieldLocator,
    pub version: AspectFieldLocator,
    pub meaning_digest: AspectFieldLocator,
    pub payload: AspectFieldLocator,
    pub original_commit: AspectFieldLocator,
    pub original_branch: AspectFieldLocator,
    pub original_entity: AspectFieldLocator,
    pub original_incarnation: AspectFieldLocator,
    pub terminal: AspectFieldLocator,
    pub provenance_kind: AspectFieldLocator,
    pub transport_attempt: AspectFieldLocator,
    pub transport_observation: AspectFieldLocator,
}

pub(super) fn lower_provider_inbound_completion(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    schema_version_id: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorthQueryInboundCompletionLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let shape = aspects()
        .struct_fields()
        .required("correlation", ScalarAspectType::String)
        .required("family", ScalarAspectType::String)
        .required("operation", ScalarAspectType::String)
        .required("audience", ScalarAspectType::String)
        .required("source", ScalarAspectType::String)
        .required("expires-at", ScalarAspectType::UInt64)
        .required("key-epoch", ScalarAspectType::UInt64)
        .required("message", ScalarAspectType::String)
        .required("protocol", ScalarAspectType::String)
        .required("version", ScalarAspectType::UInt64)
        .required("meaning-digest", ScalarAspectType::String)
        .required("payload", ScalarAspectType::String)
        .required("original-commit", ScalarAspectType::UInt64)
        .required("original-branch", ScalarAspectType::String)
        .required("original-entity", ScalarAspectType::String)
        .required("original-incarnation", ScalarAspectType::UInt64)
        .required("terminal", ScalarAspectType::String)
        .required("provenance-kind", ScalarAspectType::String)
        .required("transport-attempt", ScalarAspectType::String)
        .required("transport-observation", ScalarAspectType::String);
    let contract = aspects()
        .contract()
        .for_key(valid_aspect_key(ASPECT)?)
        .identified_by(identity)
        .at_revision(aspects().vocabulary().revision(3))
        .struct_aspect(shape.finish().map_err(|_| invalid_member(ASPECT))?);
    let registry = register_entity(
        registry,
        schema_id,
        schema_version_id,
        ENTITY,
        kind,
        vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: valid_field_key(ASPECT)?,
            },
            contract,
        }],
    )?;
    let locator = |field| planned_field_locator(ASPECT, field);
    Ok((
        registry,
        WorthQueryInboundCompletionLayout {
            kind,
            correlation: locator("correlation")?,
            family: locator("family")?,
            operation: locator("operation")?,
            audience: locator("audience")?,
            source: locator("source")?,
            expires_at: locator("expires-at")?,
            key_epoch: locator("key-epoch")?,
            message: locator("message")?,
            protocol: locator("protocol")?,
            version: locator("version")?,
            meaning_digest: locator("meaning-digest")?,
            payload: locator("payload")?,
            original_commit: locator("original-commit")?,
            original_branch: locator("original-branch")?,
            original_entity: locator("original-entity")?,
            original_incarnation: locator("original-incarnation")?,
            terminal: locator("terminal")?,
            provenance_kind: locator("provenance-kind")?,
            transport_attempt: locator("transport-attempt")?,
            transport_observation: locator("transport-observation")?,
        },
    ))
}
