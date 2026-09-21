use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType, StructAspectShape};
use worth_query_installation::facade::{
    ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
    ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints, ApplicationRelationIntegrity,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::schema::{
    AspectBinding, DeclaredAspectContractBinding, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};

use super::{
    WorkflowConnectionLayout, WorkflowDefinitionLayout, WorkflowInstanceLayout,
    WorkflowLineageLayout, WorkflowNodeLayout, CONNECTION_ASPECT, CONNECTION_ENTITY,
    DEFINITION_ASPECT, DEFINITION_ENTITY, INSTANCE_ASPECT, INSTANCE_ENTITY, LINEAGE_ASPECT,
    LINEAGE_ENTITY, NODE_ASPECT, NODE_ENTITY,
};
use crate::domain_computation::primary_graph::schema_layout::{
    invalid_member, kind_space_exhausted, planned_field_locator, register_entity, valid_aspect_key,
    valid_field_key,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

pub(super) fn lower_lineage(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorkflowLineageLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let workflow_identity = planned_field_locator(LINEAGE_ASPECT, "identity")?;
    let spec = planned_field_locator(LINEAGE_ASPECT, "spec")?;
    let protocol_version = planned_field_locator(LINEAGE_ASPECT, "protocol-version")?;
    let shape = aspects()
        .struct_fields()
        .required("identity", ScalarAspectType::String)
        .required("spec", ScalarAspectType::String)
        .required("protocol-version", ScalarAspectType::UInt64)
        .finish()
        .map_err(|_| invalid_member(LINEAGE_ASPECT))?;
    let registry = register_platform_entity(
        registry,
        schema_id,
        version,
        LINEAGE_ENTITY,
        kind,
        LINEAGE_ASPECT,
        identity,
        shape,
    )?;
    Ok((
        registry,
        WorkflowLineageLayout {
            entity_kind: kind,
            identity: workflow_identity,
            spec,
            protocol_version,
            identity_index_id: DerivedIndexId(0),
        },
    ))
}

pub(super) fn lower_definition(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorkflowDefinitionLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let content_identity = planned_field_locator(DEFINITION_ASPECT, "content-identity")?;
    let program_revision = planned_field_locator(DEFINITION_ASPECT, "program-revision")?;
    let maximum_nodes = planned_field_locator(DEFINITION_ASPECT, "maximum-nodes")?;
    let maximum_connections = planned_field_locator(DEFINITION_ASPECT, "maximum-connections")?;
    let maximum_effects = planned_field_locator(DEFINITION_ASPECT, "maximum-effects")?;
    let maximum_component_depth =
        planned_field_locator(DEFINITION_ASPECT, "maximum-component-depth")?;
    let maximum_canonical_bytes =
        planned_field_locator(DEFINITION_ASPECT, "maximum-canonical-bytes")?;
    let shape = aspects()
        .struct_fields()
        .required("content-identity", ScalarAspectType::String)
        .required("program-revision", ScalarAspectType::String)
        .required("maximum-nodes", ScalarAspectType::UInt64)
        .required("maximum-connections", ScalarAspectType::UInt64)
        .required("maximum-effects", ScalarAspectType::UInt64)
        .required("maximum-component-depth", ScalarAspectType::UInt64)
        .required("maximum-canonical-bytes", ScalarAspectType::UInt64)
        .finish()
        .map_err(|_| invalid_member(DEFINITION_ASPECT))?;
    let registry = register_platform_entity(
        registry,
        schema_id,
        version,
        DEFINITION_ENTITY,
        kind,
        DEFINITION_ASPECT,
        identity,
        shape,
    )?;
    Ok((
        registry,
        WorkflowDefinitionLayout {
            entity_kind: kind,
            content_identity,
            program_revision,
            maximum_nodes,
            maximum_connections,
            maximum_effects,
            maximum_component_depth,
            maximum_canonical_bytes,
            content_identity_index_id: DerivedIndexId(0),
        },
    ))
}

pub(super) fn lower_node(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<(RelationalSchemaRegistry, WorkflowNodeLayout), WorthQueryPrimaryGraphInstallationDenial>
{
    let path = planned_field_locator(NODE_ASPECT, "path")?;
    let node_kind = planned_field_locator(NODE_ASPECT, "kind")?;
    let member = planned_field_locator(NODE_ASPECT, "member")?;
    let input_type = planned_field_locator(NODE_ASPECT, "input-type")?;
    let parameter_type = planned_field_locator(NODE_ASPECT, "parameter-type")?;
    let result_type = planned_field_locator(NODE_ASPECT, "result-type")?;
    let assessment_binding = planned_field_locator(NODE_ASPECT, "assessment-binding")?;
    let capability_type = planned_field_locator(NODE_ASPECT, "capability-type")?;
    let approval_operation = planned_field_locator(NODE_ASPECT, "approval-operation")?;
    let approval_capability_identity =
        planned_field_locator(NODE_ASPECT, "approval-capability-identity")?;
    let requires_authority = planned_field_locator(NODE_ASPECT, "requires-authority")?;
    let shape = aspects()
        .struct_fields()
        .required("path", ScalarAspectType::String)
        .required("kind", ScalarAspectType::UInt64)
        .required("member", ScalarAspectType::String)
        .optional("input-type", ScalarAspectType::String)
        .optional("parameter-type", ScalarAspectType::String)
        .optional("result-type", ScalarAspectType::String)
        .optional("assessment-binding", ScalarAspectType::String)
        .optional("capability-type", ScalarAspectType::String)
        .optional("approval-operation", ScalarAspectType::String)
        .optional("approval-capability-identity", ScalarAspectType::String)
        .required("requires-authority", ScalarAspectType::Bool)
        .finish()
        .map_err(|_| invalid_member(NODE_ASPECT))?;
    let registry = register_platform_entity(
        registry,
        schema_id,
        version,
        NODE_ENTITY,
        kind,
        NODE_ASPECT,
        identity,
        shape,
    )?;
    Ok((
        registry,
        WorkflowNodeLayout {
            entity_kind: kind,
            path,
            kind: node_kind,
            member,
            input_type,
            parameter_type,
            result_type,
            assessment_binding,
            capability_type,
            approval_operation,
            approval_capability_identity,
            requires_authority,
        },
    ))
}

pub(super) fn lower_connection(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorkflowConnectionLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let family = planned_field_locator(CONNECTION_ASPECT, "family")?;
    let variant = planned_field_locator(CONNECTION_ASPECT, "variant")?;
    let shape = aspects()
        .struct_fields()
        .required("family", ScalarAspectType::UInt64)
        .required("variant", ScalarAspectType::UInt64)
        .finish()
        .map_err(|_| invalid_member(CONNECTION_ASPECT))?;
    let registry = register_platform_entity(
        registry,
        schema_id,
        version,
        CONNECTION_ENTITY,
        kind,
        CONNECTION_ASPECT,
        identity,
        shape,
    )?;
    Ok((
        registry,
        WorkflowConnectionLayout {
            entity_kind: kind,
            family,
            variant,
        },
    ))
}

pub(super) fn lower_instance(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorkflowInstanceLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let identity_field = planned_field_locator(INSTANCE_ASPECT, "identity")?;
    let protocol_version = planned_field_locator(INSTANCE_ASPECT, "protocol-version")?;
    let program_revision = planned_field_locator(INSTANCE_ASPECT, "program-revision")?;
    let definition_content_identity =
        planned_field_locator(INSTANCE_ASPECT, "definition-content-identity")?;
    let subject_partition = planned_field_locator(INSTANCE_ASPECT, "subject-partition")?;
    let subject_slot = planned_field_locator(INSTANCE_ASPECT, "subject-slot")?;
    let subject_generation = planned_field_locator(INSTANCE_ASPECT, "subject-generation")?;
    let state = planned_field_locator(INSTANCE_ASPECT, "state")?;
    let shape = aspects()
        .struct_fields()
        .required("identity", ScalarAspectType::String)
        .required("protocol-version", ScalarAspectType::UInt64)
        .required("program-revision", ScalarAspectType::String)
        .required("definition-content-identity", ScalarAspectType::String)
        .required("subject-partition", ScalarAspectType::UInt64)
        .required("subject-slot", ScalarAspectType::UInt64)
        .required("subject-generation", ScalarAspectType::UInt64)
        .required("state", ScalarAspectType::UInt64)
        .finish()
        .map_err(|_| invalid_member(INSTANCE_ASPECT))?;
    let registry = register_platform_entity(
        registry,
        schema_id,
        version,
        INSTANCE_ENTITY,
        kind,
        INSTANCE_ASPECT,
        identity,
        shape,
    )?;
    Ok((
        registry,
        WorkflowInstanceLayout {
            entity_kind: kind,
            identity: identity_field,
            protocol_version,
            program_revision,
            definition_content_identity,
            subject_partition,
            subject_slot,
            subject_generation,
            state,
            identity_index_id: DerivedIndexId(0),
        },
    ))
}

pub(super) fn register_platform_entity(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    name: &str,
    kind: KindId,
    aspect: &str,
    identity: AspectIdentity,
    shape: StructAspectShape,
) -> Result<RelationalSchemaRegistry, WorthQueryPrimaryGraphInstallationDenial> {
    let contract = aspects()
        .contract()
        .for_key(valid_aspect_key(aspect)?)
        .identified_by(identity)
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(shape);
    register_entity(
        registry,
        schema_id,
        version,
        name,
        kind,
        vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: valid_field_key(aspect)?,
            },
            contract,
        }],
    )
}

pub(super) fn allocate_kinds(
    first: KindId,
) -> Result<[KindId; 28], WorthQueryPrimaryGraphInstallationDenial> {
    let mut next = first.0;
    let mut kinds = [first; 28];
    for kind in kinds.iter_mut().skip(1) {
        next = next.checked_add(1).ok_or_else(kind_space_exhausted)?;
        *kind = KindId(next);
    }
    Ok(kinds)
}

pub(super) fn live_membership_integrity() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(false, ApplicationRelationCrossContextPolicy::Forbid),
        ApplicationRelationCardinality::new(None, Some(1), None, None, None, Some(1)),
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}

pub(super) fn current_definition_integrity() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(false, ApplicationRelationCrossContextPolicy::Forbid),
        ApplicationRelationCardinality::new(Some(1), Some(1), None, Some(1), None, Some(1)),
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}

pub(super) fn connection_endpoint_integrity() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(false, ApplicationRelationCrossContextPolicy::Forbid),
        ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, Some(1)),
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}

pub(super) fn owned_fact_integrity() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(false, ApplicationRelationCrossContextPolicy::Forbid),
        ApplicationRelationCardinality::new(None, None, Some(1), Some(1), None, Some(1)),
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}

pub(super) fn workflow_proposal_integrity() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(false, ApplicationRelationCrossContextPolicy::Forbid),
        ApplicationRelationCardinality::new(None, Some(1), Some(1), Some(1), None, Some(1)),
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}

pub(super) fn assessment_evidence_integrity() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(false, ApplicationRelationCrossContextPolicy::Forbid),
        ApplicationRelationCardinality::new(None, Some(1), Some(1), Some(1), None, Some(1)),
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}

pub(super) fn approval_evidence_integrity() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(false, ApplicationRelationCrossContextPolicy::Forbid),
        ApplicationRelationCardinality::new(Some(1), None, None, None, None, Some(1)),
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}

pub(super) fn approval_proposal_integrity() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::new(false, ApplicationRelationCrossContextPolicy::Forbid),
        ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, Some(1)),
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    )
}

#[cfg(test)]
#[path = "relations/tests.rs"]
mod tests;
