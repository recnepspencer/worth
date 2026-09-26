use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId};

use super::{
    relations::register_platform_entity, WorkflowInstanceLayout, INSTANCE_ASPECT, INSTANCE_ENTITY,
};
use crate::domain_computation::primary_graph::schema_layout::{
    invalid_member, planned_field_locator,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

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
    let inherited_steps = planned_field_locator(INSTANCE_ASPECT, "inherited-steps")?;
    let deadline = planned_field_locator(INSTANCE_ASPECT, "deadline")?;
    let inherited_evidence_bytes =
        planned_field_locator(INSTANCE_ASPECT, "inherited-evidence-bytes")?;
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
        .optional("inherited-steps", ScalarAspectType::UInt64)
        .optional("deadline", ScalarAspectType::UInt64)
        .optional("inherited-evidence-bytes", ScalarAspectType::UInt64)
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
            inherited_steps,
            deadline,
            inherited_evidence_bytes,
            identity_index_id: DerivedIndexId(0),
        },
    ))
}
