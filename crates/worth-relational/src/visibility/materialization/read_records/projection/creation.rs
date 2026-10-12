//! Capture immutable projection metadata while the native owner is borrowed.
use super::VisibilityProjectionView;
use crate::runtime::RelationalRuntime;
impl VisibilityProjectionView {
    pub(crate) fn new(
        runtime: &RelationalRuntime,
        basis: crate::visibility::snapshot_states::SnapshotStateBasis,
    ) -> Self {
        let schema = match basis.root() {
            Some(root) => super::schema_custody::ProjectionSchemaCustody::Published(
                root.retained_schema_authority(),
            ),
            None => super::schema_custody::ProjectionSchemaCustody::EmptyHistorical(
                std::sync::Arc::new(runtime.config.schema.registry.clone()),
            ),
        };
        Self {
            runtime_identity: runtime.runtime_instance_id(),
            current_version: runtime.current_version_id(),
            instrumentation: runtime.services.instrumentation.clone(),
            symbols: runtime.services.symbols.interner_snapshot(),
            schema,
            basis,
        }
    }
}
