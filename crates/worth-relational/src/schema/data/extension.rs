//! Registry composition installs physical sources before their dependent bindings.
use super::{RelationalSchemaRegistry, SchemaRegistryError};

impl RelationalSchemaRegistry {
    /// Existing authorities cannot be replaced. Preserve genuine shared entry
    /// handles while validating memberships only after their complete sources.
    pub fn extend(
        mut self,
        additions: RelationalSchemaRegistry,
    ) -> Result<Self, SchemaRegistryError> {
        let derived = additions.derived_relations;
        let domains = additions.projected_member_domains;
        for registration in additions.entity_kinds.into_values() {
            self = self.register_entity_kind(registration)?;
        }
        for (identity, entry) in domains {
            if self
                .projected_member_domains
                .insert(identity, entry)
                .is_some()
            {
                return Err(SchemaRegistryError::inconsistent_schema_basis(
                    "projected member domain is already registered",
                ));
            }
        }
        crate::schema::source_catalog::install(
            self,
            additions.relation_kinds.into_values(),
            derived
                .into_values()
                .map(crate::schema::source_catalog::DerivedCatalogEntry::Retained),
        )
        .map_err(|denial| {
            SchemaRegistryError::inconsistent_schema_basis(format!(
                "source catalog admission denied: {denial:?}"
            ))
        })
    }
}
