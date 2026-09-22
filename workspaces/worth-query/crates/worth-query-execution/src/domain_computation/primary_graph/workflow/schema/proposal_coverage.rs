use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId};

use super::{relations::register_platform_entity, WorkflowProposalCoverageLayout};
use crate::domain_computation::primary_graph::schema_layout::{
    invalid_member, planned_field_locator,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

const ENTITY: &str = "worth-query-workflow-proposal-coverage";
const ASPECT: &str = "workflow-proposal-coverage";

pub(super) fn lower_proposal_coverage(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorkflowProposalCoverageLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let coverage_identity = planned_field_locator(ASPECT, "identity")?;
    let selector = planned_field_locator(ASPECT, "selector")?;
    let subject_partition = planned_field_locator(ASPECT, "subject-partition")?;
    let subject_slot = planned_field_locator(ASPECT, "subject-slot")?;
    let subject_generation = planned_field_locator(ASPECT, "subject-generation")?;
    let shape = aspects()
        .struct_fields()
        .required("identity", ScalarAspectType::String)
        .required("selector", ScalarAspectType::String)
        .required("subject-partition", ScalarAspectType::UInt64)
        .required("subject-slot", ScalarAspectType::UInt64)
        .required("subject-generation", ScalarAspectType::UInt64)
        .finish()
        .map_err(|_| invalid_member(ASPECT))?;
    let registry = register_platform_entity(
        registry, schema_id, version, ENTITY, kind, ASPECT, identity, shape,
    )?;
    Ok((
        registry,
        WorkflowProposalCoverageLayout {
            entity_kind: kind,
            identity: coverage_identity,
            selector,
            subject_partition,
            subject_slot,
            subject_generation,
        },
    ))
}
