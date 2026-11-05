//! Install Native's current-ownership storage only for opted-in producers.
use super::{
    relational_schema_denial, RelationalSchemaRegistry, WorthQueryPrimaryGraphInstallationDenial,
};

pub(in crate::domain_computation::primary_graph) fn install_current_publication_ownership(
    registry: RelationalSchemaRegistry,
) -> Result<RelationalSchemaRegistry, WorthQueryPrimaryGraphInstallationDenial> {
    let basis = registry
        .authoritative_schema_basis()
        .map_err(relational_schema_denial)?
        .ok_or_else(|| {
            relational_schema_denial(
                worth_relational::facade::schema::SchemaRegistryError::inconsistent_schema_basis(
                    "current publication ownership requires the installed schema basis",
                ),
            )
        })?;
    worth_relational::facade::schema::register_current_publication_storage(
        registry, &basis.0, basis.1,
    )
    .map_err(relational_schema_denial)
}
