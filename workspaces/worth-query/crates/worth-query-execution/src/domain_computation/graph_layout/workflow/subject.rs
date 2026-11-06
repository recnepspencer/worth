//! Native instance-to-subject relation, appended after stable platform kinds.
use worth_relational::facade::{
    identity::KindId,
    schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId},
};
use {
    crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial,
    crate::domain_computation::graph_layout::kind_space_exhausted,
    crate::domain_computation::graph_layout::register_relation_to_kinds,
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
