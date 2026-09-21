use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId};

use super::relations::register_platform_entity;
use super::WorkflowProposalLayout;
use crate::domain_computation::primary_graph::schema_layout::{
    invalid_member, planned_field_locator,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

const PROPOSAL_ENTITY: &str = "worth-query-workflow-proposal";
const PROPOSAL_ASPECT: &str = "workflow-proposal";

pub(super) fn lower_proposal(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorkflowProposalLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let identity_field = planned_field_locator(PROPOSAL_ASPECT, "identity")?;
    let protocol_version = planned_field_locator(PROPOSAL_ASPECT, "protocol-version")?;
    let operation = planned_field_locator(PROPOSAL_ASPECT, "operation")?;
    let input_type = planned_field_locator(PROPOSAL_ASPECT, "input-type")?;
    let input_identity = planned_field_locator(PROPOSAL_ASPECT, "input-identity")?;
    let source_identity = planned_field_locator(PROPOSAL_ASPECT, "source-identity")?;
    let node_path = planned_field_locator(PROPOSAL_ASPECT, "node-path")?;
    let shape = aspects()
        .struct_fields()
        .required("identity", ScalarAspectType::String)
        .required("protocol-version", ScalarAspectType::UInt64)
        .required("operation", ScalarAspectType::String)
        .required("input-type", ScalarAspectType::String)
        .required("input-identity", ScalarAspectType::String)
        .optional("source-identity", ScalarAspectType::String)
        .required("node-path", ScalarAspectType::String)
        .finish()
        .map_err(|_| invalid_member(PROPOSAL_ASPECT))?;
    let registry = register_platform_entity(
        registry,
        schema_id,
        version,
        PROPOSAL_ENTITY,
        kind,
        PROPOSAL_ASPECT,
        identity,
        shape,
    )?;
    Ok((
        registry,
        WorkflowProposalLayout {
            entity_kind: kind,
            identity: identity_field,
            protocol_version,
            operation,
            input_type,
            input_identity,
            source_identity,
            node_path,
            identity_index_id: DerivedIndexId(0),
        },
    ))
}
