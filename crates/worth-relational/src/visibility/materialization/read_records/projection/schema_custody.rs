//! Published schema custody, with an explicit empty historical reconstruction basis.
use crate::schema::data::RelationalSchemaRegistry;
use std::sync::Arc;
#[derive(Clone, Debug)]
pub(super) enum ProjectionSchemaCustody {
    Published(Arc<crate::branch::RelationalBranchRootSchemaAuthority>),
    EmptyHistorical(Arc<RelationalSchemaRegistry>),
}
impl ProjectionSchemaCustody {
    pub(super) fn registry(&self) -> &RelationalSchemaRegistry {
        match self {
            Self::Published(schema) => schema.registry(),
            Self::EmptyHistorical(schema) => schema,
        }
    }
}
