use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId};

use super::{relations::register_platform_entity, WorkflowApprovalLayout};
use crate::domain_computation::primary_graph::schema_layout::{
    invalid_member, planned_field_locator,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

const APPROVAL_ENTITY: &str = "worth-query-workflow-approval";
const APPROVAL_ASPECT: &str = "workflow-approval";

pub(super) fn lower_approval(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorkflowApprovalLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let field = |name| planned_field_locator(APPROVAL_ASPECT, name);
    let layout = WorkflowApprovalLayout {
        entity_kind: kind,
        identity: field("identity")?,
        protocol_version: field("protocol-version")?,
        instance_identity: field("instance-identity")?,
        definition_content_identity: field("definition-content-identity")?,
        program_revision: field("program-revision")?,
        decision: field("decision")?,
        approval_operation: field("approval-operation")?,
        target_operation: field("target-operation")?,
        installed_capability_identity: field("installed-capability-identity")?,
        approver_partition: field("approver-partition")?,
        approver_slot: field("approver-slot")?,
        approver_generation: field("approver-generation")?,
        scope_partition: field("scope-partition")?,
        scope_slot: field("scope-slot")?,
        scope_generation: field("scope-generation")?,
        grant_partition: field("grant-partition")?,
        grant_slot: field("grant-slot")?,
        grant_generation: field("grant-generation")?,
        authorization_decision: field("authorization-decision")?,
        authorization_request: field("authorization-request")?,
        authorization_principal: field("authorization-principal")?,
        capability_authority_identity: field("capability-authority-identity")?,
        authorization_lineage: field("authorization-lineage")?,
        authorization_support: field("authorization-support")?,
        authorization_dependencies: field("authorization-dependencies")?,
        action: field("action")?,
        purpose: field("purpose")?,
        validity_timeline: field("validity-timeline")?,
        authorization_sample: field("authorization-sample")?,
        expiry: field("expiry")?,
    };
    let mut shape = aspects()
        .struct_fields()
        .required("identity", ScalarAspectType::String)
        .required("protocol-version", ScalarAspectType::UInt64)
        .required("instance-identity", ScalarAspectType::String)
        .required("definition-content-identity", ScalarAspectType::String)
        .required("program-revision", ScalarAspectType::String)
        .required("decision", ScalarAspectType::String)
        .required("approval-operation", ScalarAspectType::String)
        .required("target-operation", ScalarAspectType::String)
        .required("installed-capability-identity", ScalarAspectType::String);
    for name in [
        "approver-partition",
        "approver-slot",
        "approver-generation",
        "scope-partition",
        "scope-slot",
        "scope-generation",
        "grant-partition",
        "grant-slot",
        "grant-generation",
    ] {
        shape = shape.required(name, ScalarAspectType::UInt64);
    }
    let shape = shape
        .required("authorization-decision", ScalarAspectType::String)
        .required("authorization-request", ScalarAspectType::String)
        .required("authorization-principal", ScalarAspectType::String)
        .required("capability-authority-identity", ScalarAspectType::String)
        .required("authorization-lineage", ScalarAspectType::String)
        .required("authorization-support", ScalarAspectType::String)
        .required("authorization-dependencies", ScalarAspectType::String)
        .required("action", ScalarAspectType::String)
        .required("purpose", ScalarAspectType::String)
        .required("validity-timeline", ScalarAspectType::String)
        .required("authorization-sample", ScalarAspectType::String)
        .required("expiry", ScalarAspectType::UInt64)
        .finish()
        .map_err(|_| invalid_member(APPROVAL_ASPECT))?;
    let registry = register_platform_entity(
        registry,
        schema_id,
        version,
        APPROVAL_ENTITY,
        kind,
        APPROVAL_ASPECT,
        identity,
        shape,
    )?;
    Ok((registry, layout))
}
