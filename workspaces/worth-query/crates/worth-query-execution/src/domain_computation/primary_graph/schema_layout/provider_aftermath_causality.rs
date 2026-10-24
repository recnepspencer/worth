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

const ENTITY: &str = "worth-query-aftermath-causality";
const ASPECT: &str = "aftermath-causality";
const KEY_FIELD: &str = "key";
const ROLE_FIELD: &str = "role";
const PARENT_BRANCH_FIELD: &str = "parent-branch";
const PARENT_COMMIT_FIELD: &str = "parent-commit";
const OUTCOME_IDENTITY_FIELD: &str = "outcome-identity";
const SOURCE_WORLD_OWNER_FIELD: &str = "source-world-owner";
const SOURCE_WORLD_COMMIT_FIELD: &str = "source-world-commit";
const SOURCE_BRANCH_FIELD: &str = "source-branch";
const SOURCE_COMMIT_FIELD: &str = "source-commit";
mod lineage;
pub(in crate::domain_computation::primary_graph) use lineage::WorthQueryCorrectionLineageLayout;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryAftermathCausalityLayout {
    pub entity_kind: KindId,
    pub key_locator: AspectFieldLocator,
    pub role_locator: AspectFieldLocator,
    pub parent_branch_locator: AspectFieldLocator,
    pub parent_commit_locator: AspectFieldLocator,
    pub outcome_identity_locator: AspectFieldLocator,
    pub source_world_owner_locator: AspectFieldLocator,
    pub source_world_commit_locator: AspectFieldLocator,
    pub source_branch_locator: AspectFieldLocator,
    pub source_commit_locator: AspectFieldLocator,
    pub key_index_id: DerivedIndexId,
    pub lineage: WorthQueryCorrectionLineageLayout,
}

pub(crate) fn lower_provider_aftermath_causality(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    schema_version_id: SchemaVersionId,
    entity_kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorthQueryAftermathCausalityLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let key_locator = planned_field_locator(ASPECT, KEY_FIELD)?;
    let role_locator = planned_field_locator(ASPECT, ROLE_FIELD)?;
    let parent_branch_locator = planned_field_locator(ASPECT, PARENT_BRANCH_FIELD)?;
    let parent_commit_locator = planned_field_locator(ASPECT, PARENT_COMMIT_FIELD)?;
    let outcome_identity_locator = planned_field_locator(ASPECT, OUTCOME_IDENTITY_FIELD)?;
    let source_world_owner_locator = planned_field_locator(ASPECT, SOURCE_WORLD_OWNER_FIELD)?;
    let source_world_commit_locator = planned_field_locator(ASPECT, SOURCE_WORLD_COMMIT_FIELD)?;
    let source_branch_locator = planned_field_locator(ASPECT, SOURCE_BRANCH_FIELD)?;
    let source_commit_locator = planned_field_locator(ASPECT, SOURCE_COMMIT_FIELD)?;
    let shape = aspects()
        .struct_fields()
        .required(KEY_FIELD, ScalarAspectType::String)
        .required(ROLE_FIELD, ScalarAspectType::UInt64)
        .required(PARENT_BRANCH_FIELD, ScalarAspectType::String)
        .required(PARENT_COMMIT_FIELD, ScalarAspectType::UInt64)
        .required(OUTCOME_IDENTITY_FIELD, ScalarAspectType::UInt64)
        .optional(SOURCE_WORLD_OWNER_FIELD, ScalarAspectType::UInt64)
        .optional(SOURCE_WORLD_COMMIT_FIELD, ScalarAspectType::UInt64)
        .optional(SOURCE_BRANCH_FIELD, ScalarAspectType::String)
        .optional(SOURCE_COMMIT_FIELD, ScalarAspectType::UInt64);
    let (shape, lineage) = lineage::lower(shape)?;
    let shape = shape.finish().map_err(|_| super::invalid_member(ASPECT))?;
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
        WorthQueryAftermathCausalityLayout {
            entity_kind,
            key_locator,
            role_locator,
            parent_branch_locator,
            parent_commit_locator,
            outcome_identity_locator,
            source_world_owner_locator,
            source_world_commit_locator,
            source_branch_locator,
            source_commit_locator,
            key_index_id: DerivedIndexId(0),
            lineage,
        },
    ))
}
