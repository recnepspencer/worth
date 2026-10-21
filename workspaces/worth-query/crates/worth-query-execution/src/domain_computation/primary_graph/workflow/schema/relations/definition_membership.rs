//! Definition ownership and control-graph endpoints retain their original native kind offsets.
use super::{
    connection_endpoint_integrity, current_definition_integrity, definition_start_integrity,
    owned_fact_integrity, WorthQueryPrimaryGraphInstallationDenial,
};
use crate::domain_computation::primary_graph::schema_layout::register_relation;
use worth_relational::facade::{
    identity::KindId,
    schema::{RelationalSchemaRegistry, SchemaId, SchemaVersionId},
};

pub(in crate::domain_computation::primary_graph::workflow::schema) fn register(
    mut registry: RelationalSchemaRegistry,
    schema: &SchemaId,
    version: SchemaVersionId,
    kinds: &[KindId; 32],
) -> Result<RelationalSchemaRegistry, WorthQueryPrimaryGraphInstallationDenial> {
    let relations = [
        (
            "worth-query-workflow-current-definition",
            kinds[0],
            kinds[1],
            current_definition_integrity(),
        ),
        (
            "worth-query-workflow-lineage-definition",
            kinds[0],
            kinds[1],
            owned_fact_integrity(),
        ),
        (
            "worth-query-workflow-definition-node",
            kinds[1],
            kinds[2],
            owned_fact_integrity(),
        ),
        (
            "worth-query-workflow-definition-connection",
            kinds[1],
            kinds[3],
            owned_fact_integrity(),
        ),
        (
            "worth-query-workflow-connection-source",
            kinds[3],
            kinds[2],
            connection_endpoint_integrity(),
        ),
        (
            "worth-query-workflow-connection-target",
            kinds[3],
            kinds[2],
            connection_endpoint_integrity(),
        ),
        (
            "worth-query-workflow-definition-start",
            kinds[1],
            kinds[2],
            definition_start_integrity(),
        ),
    ];
    for (offset, (name, from, to, integrity)) in relations.into_iter().enumerate() {
        registry = register_relation(
            registry,
            schema,
            version,
            name,
            kinds[5 + offset],
            from,
            to,
            integrity,
        )?;
    }
    Ok(registry)
}
