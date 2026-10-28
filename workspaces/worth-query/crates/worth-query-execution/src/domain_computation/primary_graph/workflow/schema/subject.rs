//! Native instance-to-subject relation, appended after stable platform kinds.
use crate::domain_computation::primary_graph::{
    schema_layout::{kind_space_exhausted, register_relation_to_kinds},
    WorthQueryPrimaryGraphInstallationDenial,
};
use worth_relational::facade::{
    identity::KindId,
    schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId},
};
pub(super) fn register(
    registry: RelationalSchemaRegistry,
    schema_id: &SchemaId,
    version: SchemaVersionId,
    first_kind: KindId,
    instance_kind: KindId,
    subject_kinds: Vec<KindId>,
) -> Result<(RelationalSchemaRegistry, KindId), WorthQueryPrimaryGraphInstallationDenial> {
    // Offset 32 is inbound completion; no old platform kind is renumbered.
    let kind = KindId(
        first_kind
            .0
            .checked_add(33)
            .ok_or_else(kind_space_exhausted)?,
    );
    let registry = register_relation_to_kinds(
        registry,
        schema_id,
        version,
        "worth-query-workflow-instance-subject",
        kind,
        instance_kind,
        subject_kinds,
        &super::relations::subject_integrity().into(),
        None,
    )?;
    Ok((registry, kind))
}
