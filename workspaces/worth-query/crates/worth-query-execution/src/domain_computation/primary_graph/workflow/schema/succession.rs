//! A completed chain has at most one fresh successor; old native kind IDs stay stable.
use crate::domain_computation::primary_graph::{
    schema_layout::{kind_space_exhausted, register_relation},
    WorthQueryPrimaryGraphInstallationDenial,
};
use worth_relational::facade::{
    identity::KindId,
    schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId},
};
pub(super) fn register(
    registry: RelationalSchemaRegistry,
    schema: &SchemaId,
    version: SchemaVersionId,
    first_kind: KindId,
    instance: KindId,
) -> Result<(RelationalSchemaRegistry, KindId), WorthQueryPrimaryGraphInstallationDenial> {
    let kind = KindId(
        first_kind
            .0
            .checked_add(34)
            .ok_or_else(kind_space_exhausted)?,
    );
    let registry = register_relation(
        registry,
        schema,
        version,
        "worth-query-workflow-instance-succeeded-from",
        kind,
        instance,
        instance,
        &super::relations::single_successor_integrity().into(),
        None,
    )?;
    Ok((registry, kind))
}
