use worth_foundational::facade::{aspects, AspectFieldLocator, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{
    AspectBinding, DeclaredAspectContractBinding, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};

use super::{
    planned_field_locator, register_entity, valid_aspect_key, valid_field_key,
    WorthQueryPrimaryGraphInstallationDenial,
};

const ENTITY: &str = "worth-query-program-activation";
const ASPECT: &str = "activation";
const PROGRAM_REVISION_FIELD: &str = "program-revision";

/// Where the branch program activation record lives in lowered Relational
/// truth.
///
/// The activation record carries the canonical rendering of the program
/// revision this occurrence runs under. The rendering is owner truth read back
/// by comparison against an admitted support roster; nothing reconstructs a
/// revision from it.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryProgramActivationLayout {
    pub(in crate::domain_computation::primary_graph) entity_kind: KindId,
    pub(in crate::domain_computation::primary_graph) program_revision_locator: AspectFieldLocator,
}

pub(super) fn lower_program_activation(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    schema_version_id: SchemaVersionId,
    entity_kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorthQueryProgramActivationLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let program_revision_locator = planned_field_locator(ASPECT, PROGRAM_REVISION_FIELD)?;
    let shape = aspects()
        .struct_fields()
        .required(PROGRAM_REVISION_FIELD, ScalarAspectType::String)
        .finish()
        .map_err(|_| super::invalid_member(ASPECT))?;
    let contract = aspects()
        .contract()
        .for_key(valid_aspect_key(ASPECT)?)
        .identified_by(identity)
        .at_revision(aspects().vocabulary().revision(1))
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
        WorthQueryProgramActivationLayout {
            entity_kind,
            program_revision_locator,
        },
    ))
}
