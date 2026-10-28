//! Immutable operation correspondence uses real references, including same-candidate creations.
use super::{
    relations::{owned_fact_integrity, register_platform_entity},
    WorkflowOperationOutputLayout,
};
use crate::domain_computation::primary_graph::{
    schema_layout::{
        invalid_member, kind_space_exhausted, planned_field_locator, register_relation,
        register_relation_to_kinds,
    },
    WorthQueryPrimaryGraphInstallationDenial as Denial,
};
use worth_foundational::facade::{aspects, AspectIdentity, ScalarAspectType};
use worth_query_installation::facade::{
    ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
    ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints, ApplicationRelationIntegrity,
};
use worth_relational::facade::{
    identity::KindId,
    schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId},
};

pub(super) fn lower(
    registry: RelationalSchemaRegistry,
    schema: &SchemaId,
    version: SchemaVersionId,
    first: KindId,
    transition: KindId,
    identity: AspectIdentity,
    targets: Vec<KindId>,
) -> Result<(RelationalSchemaRegistry, WorkflowOperationOutputLayout), Denial> {
    let kind = |offset| {
        first
            .0
            .checked_add(offset)
            .map(KindId)
            .ok_or_else(kind_space_exhausted)
    };
    let entity_kind = kind(35)?;
    let membership = kind(36)?;
    let target = kind(37)?;
    let aspect = "workflow-operation-output";
    let shape = aspects()
        .struct_fields()
        .required("role", ScalarAspectType::String)
        .required("entity", ScalarAspectType::String)
        .required("posture", ScalarAspectType::UInt64)
        .finish()
        .map_err(|_| invalid_member(aspect))?;
    let registry = register_platform_entity(
        registry,
        schema,
        version,
        "worth-query-workflow-operation-output",
        entity_kind,
        aspect,
        identity,
        shape,
    )?;
    let registry = register_relation(
        registry,
        schema,
        version,
        "worth-query-workflow-transition-output",
        membership,
        transition,
        entity_kind,
        &owned_fact_integrity().into(),
        None,
    )?;
    let registry = register_relation_to_kinds(
        registry,
        schema,
        version,
        "worth-query-workflow-operation-output-target",
        target,
        entity_kind,
        targets,
        &ApplicationRelationIntegrity::new(
            ApplicationRelationEndpoints::new(
                false,
                ApplicationRelationCrossContextPolicy::AllowExplicit,
            ),
            // Cardinality counts live incidence, not audit-retained edges. A
            // business target may retire; publication/custody and the native
            // decoder preserve exactly one immutable historical reference.
            ApplicationRelationCardinality::new(None, Some(1), None, None, None, Some(1)),
            ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
        )
        .into(),
        None,
    )?;
    Ok((
        registry,
        WorkflowOperationOutputLayout {
            entity_kind,
            membership,
            target,
            role: planned_field_locator(aspect, "role")?,
            entity: planned_field_locator(aspect, "entity")?,
            posture: planned_field_locator(aspect, "posture")?,
        },
    ))
}
