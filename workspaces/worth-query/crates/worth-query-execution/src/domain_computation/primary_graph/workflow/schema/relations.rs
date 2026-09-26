use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType, StructAspectShape};
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

#[path = "relations/integrity.rs"]
mod integrity;
pub(super) use integrity::*;

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
    let carried_revision = planned_field_locator(DEFINITION_ASPECT, "carried-revision")?;
    let maximum_nodes = planned_field_locator(DEFINITION_ASPECT, "maximum-nodes")?;
    let maximum_connections = planned_field_locator(DEFINITION_ASPECT, "maximum-connections")?;
    let maximum_effects = planned_field_locator(DEFINITION_ASPECT, "maximum-effects")?;
    let maximum_component_occurrences =
        planned_field_locator(DEFINITION_ASPECT, "maximum-component-occurrences")?;
    let maximum_component_depth =
        planned_field_locator(DEFINITION_ASPECT, "maximum-component-depth")?;
    let maximum_node_provenance =
        planned_field_locator(DEFINITION_ASPECT, "maximum-node-provenance")?;
    let maximum_connection_provenance =
        planned_field_locator(DEFINITION_ASPECT, "maximum-connection-provenance")?;
    let maximum_port_provenance =
        planned_field_locator(DEFINITION_ASPECT, "maximum-port-provenance")?;
    let maximum_canonical_bytes =
        planned_field_locator(DEFINITION_ASPECT, "maximum-canonical-bytes")?;
    let shape = aspects()
        .struct_fields()
        .required("content-identity", ScalarAspectType::String)
        .required("program-revision", ScalarAspectType::String)
        .optional("carried-revision", ScalarAspectType::String)
        .required("maximum-nodes", ScalarAspectType::UInt64)
        .required("maximum-connections", ScalarAspectType::UInt64)
        .required("maximum-effects", ScalarAspectType::UInt64)
        .required("maximum-component-occurrences", ScalarAspectType::UInt64)
        .required("maximum-component-depth", ScalarAspectType::UInt64)
        .required("maximum-node-provenance", ScalarAspectType::UInt64)
        .required("maximum-connection-provenance", ScalarAspectType::UInt64)
        .required("maximum-port-provenance", ScalarAspectType::UInt64)
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
            carried_revision,
            maximum_nodes,
            maximum_connections,
            maximum_effects,
            maximum_component_occurrences,
            maximum_component_depth,
            maximum_node_provenance,
            maximum_connection_provenance,
            maximum_port_provenance,
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
    let operation_binding = planned_field_locator(NODE_ASPECT, "operation-binding")?;
    let parameter_type = planned_field_locator(NODE_ASPECT, "parameter-type")?;
    let result_type = planned_field_locator(NODE_ASPECT, "result-type")?;
    let assessment_binding = planned_field_locator(NODE_ASPECT, "assessment-binding")?;
    let assessment_subject = planned_field_locator(NODE_ASPECT, "assessment-subject")?;
    let assessment_applicability_relation =
        planned_field_locator(NODE_ASPECT, "assessment-applicability-relation")?;
    let assessment_applicability_from =
        planned_field_locator(NODE_ASPECT, "assessment-applicability-from")?;
    let assessment_applicability_to =
        planned_field_locator(NODE_ASPECT, "assessment-applicability-to")?;
    let condition_binding = planned_field_locator(NODE_ASPECT, "condition-binding")?;
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
        .optional("operation-binding", ScalarAspectType::String)
        .optional("parameter-type", ScalarAspectType::String)
        .optional("result-type", ScalarAspectType::String)
        .optional("assessment-binding", ScalarAspectType::String)
        .optional("assessment-subject", ScalarAspectType::String)
        .optional(
            "assessment-applicability-relation",
            ScalarAspectType::String,
        )
        .optional("assessment-applicability-from", ScalarAspectType::String)
        .optional("assessment-applicability-to", ScalarAspectType::String)
        .optional("condition-binding", ScalarAspectType::String)
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
            operation_binding,
            parameter_type,
            result_type,
            assessment_binding,
            assessment_subject,
            assessment_applicability_relation,
            assessment_applicability_from,
            assessment_applicability_to,
            condition_binding,
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
    let retry_reason = planned_field_locator(CONNECTION_ASPECT, "retry-reason")?;
    let retry_maximum_attempts =
        planned_field_locator(CONNECTION_ASPECT, "retry-maximum-attempts")?;
    let shape = aspects()
        .struct_fields()
        .required("family", ScalarAspectType::UInt64)
        .required("variant", ScalarAspectType::UInt64)
        .optional("retry-reason", ScalarAspectType::String)
        .optional("retry-maximum-attempts", ScalarAspectType::UInt64)
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
            retry_reason,
            retry_maximum_attempts,
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
    let branch_occurrence = planned_field_locator(INSTANCE_ASPECT, "branch-occurrence")?;
    let program_revision = planned_field_locator(INSTANCE_ASPECT, "program-revision")?;
    let definition_content_identity =
        planned_field_locator(INSTANCE_ASPECT, "definition-content-identity")?;
    let subject_partition = planned_field_locator(INSTANCE_ASPECT, "subject-partition")?;
    let subject_slot = planned_field_locator(INSTANCE_ASPECT, "subject-slot")?;
    let subject_generation = planned_field_locator(INSTANCE_ASPECT, "subject-generation")?;
    let state = planned_field_locator(INSTANCE_ASPECT, "state")?;
    let resume_node_path = planned_field_locator(INSTANCE_ASPECT, "resume-node-path")?;
    let cancellation_identity = planned_field_locator(INSTANCE_ASPECT, "cancellation-identity")?;
    let shape = aspects()
        .struct_fields()
        .required("identity", ScalarAspectType::String)
        .required("protocol-version", ScalarAspectType::UInt64)
        .required("branch-occurrence", ScalarAspectType::UInt64)
        .required("program-revision", ScalarAspectType::String)
        .required("definition-content-identity", ScalarAspectType::String)
        .required("subject-partition", ScalarAspectType::UInt64)
        .required("subject-slot", ScalarAspectType::UInt64)
        .required("subject-generation", ScalarAspectType::UInt64)
        .required("state", ScalarAspectType::UInt64)
        .optional("resume-node-path", ScalarAspectType::String)
        .optional("cancellation-identity", ScalarAspectType::String)
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
            branch_occurrence,
            program_revision,
            definition_content_identity,
            subject_partition,
            subject_slot,
            subject_generation,
            state,
            resume_node_path,
            cancellation_identity,
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
) -> Result<[KindId; 32], WorthQueryPrimaryGraphInstallationDenial> {
    let mut next = first.0;
    let mut kinds = [first; 32];
    for kind in kinds.iter_mut().skip(1) {
        next = next.checked_add(1).ok_or_else(kind_space_exhausted)?;
        *kind = KindId(next);
    }
    Ok(kinds)
}

#[cfg(test)]
#[path = "relations/tests.rs"]
mod tests;
