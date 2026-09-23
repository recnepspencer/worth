use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId};

use super::relations::register_platform_entity;
use super::WorkflowTransitionLayout;
use crate::domain_computation::primary_graph::schema_layout::{
    invalid_member, planned_field_locator,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

const TRANSITION_ENTITY: &str = "worth-query-workflow-transition";
const TRANSITION_ASPECT: &str = "workflow-transition";

pub(super) fn lower_transition(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorkflowTransitionLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let identity_field = planned_field_locator(TRANSITION_ASPECT, "identity")?;
    let protocol_version = planned_field_locator(TRANSITION_ASPECT, "protocol-version")?;
    let occurrence = planned_field_locator(TRANSITION_ASPECT, "occurrence")?;
    let outcome = planned_field_locator(TRANSITION_ASPECT, "outcome")?;
    let operation_receipt_identity =
        planned_field_locator(TRANSITION_ASPECT, "operation-receipt-identity")?;
    let live_membership_partition =
        planned_field_locator(TRANSITION_ASPECT, "live-membership-partition")?;
    let live_membership_slot = planned_field_locator(TRANSITION_ASPECT, "live-membership-slot")?;
    let live_membership_generation =
        planned_field_locator(TRANSITION_ASPECT, "live-membership-generation")?;
    let shape = aspects()
        .struct_fields()
        .required("identity", ScalarAspectType::String)
        .required("protocol-version", ScalarAspectType::UInt64)
        .required("occurrence", ScalarAspectType::UInt64)
        .required("outcome", ScalarAspectType::UInt64)
        .optional("operation-receipt-identity", ScalarAspectType::String)
        .required("live-membership-partition", ScalarAspectType::UInt64)
        .required("live-membership-slot", ScalarAspectType::UInt64)
        .required("live-membership-generation", ScalarAspectType::UInt64)
        .finish()
        .map_err(|_| invalid_member(TRANSITION_ASPECT))?;
    let registry = register_platform_entity(
        registry,
        schema_id,
        version,
        TRANSITION_ENTITY,
        kind,
        TRANSITION_ASPECT,
        identity,
        shape,
    )?;
    Ok((
        registry,
        WorkflowTransitionLayout {
            entity_kind: kind,
            identity: identity_field,
            protocol_version,
            occurrence,
            outcome,
            operation_receipt_identity,
            live_membership_partition,
            live_membership_slot,
            live_membership_generation,
        },
    ))
}
