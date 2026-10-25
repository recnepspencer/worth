//! Appended native proposal target-input vocabulary; existing kind identities stay fixed.
use super::{
    relations::{owned_fact_integrity, register_platform_entity},
    WorkflowCapturedInputLayout,
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
    indexes::DerivedIndexId,
    schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId},
};

pub(super) fn lower(
    registry: RelationalSchemaRegistry,
    schema: &SchemaId,
    version: SchemaVersionId,
    first: KindId,
    proposal: KindId,
    identity: AspectIdentity,
    subjects: Vec<KindId>,
) -> Result<(RelationalSchemaRegistry, WorkflowCapturedInputLayout), Denial> {
    let kind = |offset| {
        first
            .0
            .checked_add(offset)
            .map(KindId)
            .ok_or_else(kind_space_exhausted)
    };
    let entity_kind = kind(38)?;
    let membership = kind(39)?;
    let resource = kind(40)?;
    let aspect = "workflow-captured-input";
    let shape = aspects()
        .struct_fields()
        .required("identity", ScalarAspectType::String)
        .required("node", ScalarAspectType::String)
        .required("binding", ScalarAspectType::String)
        .required("input-type", ScalarAspectType::String)
        .required("input", ScalarAspectType::String)
        .required("key", ScalarAspectType::String)
        .required("source-kind", ScalarAspectType::UInt64)
        .optional("source", ScalarAspectType::String)
        .optional("source-partition", ScalarAspectType::String)
        .optional("preconditions", ScalarAspectType::String)
        .finish()
        .map_err(|_| invalid_member(aspect))?;
    let registry = register_platform_entity(
        registry,
        schema,
        version,
        "worth-query-workflow-captured-input",
        entity_kind,
        aspect,
        identity,
        shape,
    )?;
    let registry = register_relation(
        registry,
        schema,
        version,
        "worth-query-workflow-proposal-input",
        membership,
        proposal,
        entity_kind,
        owned_fact_integrity(),
        None,
    )?;
    let registry = register_relation_to_kinds(
        registry,
        schema,
        version,
        "worth-query-workflow-input-resource",
        resource,
        entity_kind,
        subjects,
        ApplicationRelationIntegrity::new(
            ApplicationRelationEndpoints::new(
                false,
                ApplicationRelationCrossContextPolicy::AllowExplicit,
            ),
            ApplicationRelationCardinality::new(None, Some(1), None, None, None, Some(1)),
            ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
        ),
        None,
    )?;
    Ok((
        registry,
        WorkflowCapturedInputLayout {
            entity_kind,
            membership,
            resource,
            identity_index_id: DerivedIndexId(0),
            identity: planned_field_locator(aspect, "identity")?,
            node: planned_field_locator(aspect, "node")?,
            binding: planned_field_locator(aspect, "binding")?,
            input_type: planned_field_locator(aspect, "input-type")?,
            input: planned_field_locator(aspect, "input")?,
            key: planned_field_locator(aspect, "key")?,
            source_kind: planned_field_locator(aspect, "source-kind")?,
            source: planned_field_locator(aspect, "source")?,
            source_partition: planned_field_locator(aspect, "source-partition")?,
            preconditions: planned_field_locator(aspect, "preconditions")?,
        },
    ))
}
