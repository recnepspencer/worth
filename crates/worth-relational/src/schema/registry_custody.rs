//! Custody of the exact schema registry issued at a genuine observation.
use super::data::RelationalSchemaRegistry;

/// Retains the registry already issued at the genuine observation boundary.
/// The runtime-only route has no selected branch and retains its config owner;
/// branch observations always retain the selected root's registry directly.
#[derive(Clone)]
pub(crate) enum SchemaRegistryCustody {
    Runtime(std::sync::Arc<crate::config::data::RelationalRuntimeConfig>),
    Selected(crate::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry),
}

impl SchemaRegistryCustody {
    pub(crate) fn registry(&self) -> &RelationalSchemaRegistry {
        match self {
            Self::Runtime(config) => &config.schema.registry,
            Self::Selected(registry) => registry,
        }
    }
}
