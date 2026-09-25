use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId};

use super::{relations::register_platform_entity, WorkflowEvidenceDependencyLayout};
use crate::domain_computation::primary_graph::schema_layout::{
    invalid_member, planned_field_locator,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

const DEPENDENCY_ENTITY: &str = "worth-query-workflow-evidence-dependency";
const DEPENDENCY_ASPECT: &str = "workflow-evidence-dependency";

pub(super) fn lower_evidence_dependency(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    kind: KindId,
    identity: AspectIdentity,
) -> Result<
    (RelationalSchemaRegistry, WorkflowEvidenceDependencyLayout),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let field = |name| planned_field_locator(DEPENDENCY_ASPECT, name);
    let layout = WorkflowEvidenceDependencyLayout {
        entity_kind: kind,
        fact_kind: field("fact-kind")?,
        entity_partition: field("entity-partition")?,
        entity_slot: field("entity-slot")?,
        entity_generation: field("entity-generation")?,
        aspect: field("aspect")?,
        field: field("field")?,
        field_presence: field("field-presence")?,
        relation_kind: field("relation-kind")?,
        direction: field("direction")?,
        native_revision: field("native-revision")?,
        comparison_work_limit: field("comparison-work-limit")?,
    };
    let shape = aspects()
        .struct_fields()
        .required("fact-kind", ScalarAspectType::String)
        .required("entity-partition", ScalarAspectType::UInt64)
        .required("entity-slot", ScalarAspectType::UInt64)
        .required("entity-generation", ScalarAspectType::UInt64)
        .optional("aspect", ScalarAspectType::String)
        .optional("field", ScalarAspectType::String)
        .optional("field-presence", ScalarAspectType::String)
        .optional("relation-kind", ScalarAspectType::UInt64)
        .optional("direction", ScalarAspectType::String)
        .optional("native-revision", ScalarAspectType::UInt64)
        .optional("comparison-work-limit", ScalarAspectType::UInt64)
        .finish()
        .map_err(|_| invalid_member(DEPENDENCY_ASPECT))?;
    let registry = register_platform_entity(
        registry,
        schema_id,
        version,
        DEPENDENCY_ENTITY,
        kind,
        DEPENDENCY_ASPECT,
        identity,
        shape,
    )?;
    Ok((registry, layout))
}
