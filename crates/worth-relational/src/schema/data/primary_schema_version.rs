//! Scalar primary identity comes from the same basis as durable authority.
use super::{RelationalSchemaRegistry, SchemaVersionId};

impl RelationalSchemaRegistry {
    /// O(kinds): preserve the projection's absent/mixed-basis fallback without
    /// building entity, relation, integrity-plan or content projections.
    pub(crate) fn primary_schema_version_id(&self) -> SchemaVersionId {
        self.authoritative_schema_basis()
            .ok()
            .flatten()
            .map(|(_, version)| version)
            .unwrap_or(SchemaVersionId(0))
    }
}
