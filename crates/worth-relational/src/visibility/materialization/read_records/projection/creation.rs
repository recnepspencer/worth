//! Capture immutable projection metadata while the native owner is borrowed.
use super::VisibilityProjectionView;
use crate::runtime::RelationalRuntime;
impl VisibilityProjectionView {
    pub(crate) fn new(
        runtime: &RelationalRuntime,
        basis: crate::visibility::snapshot_states::SnapshotStateBasis,
    ) -> Self {
        Self::with_symbols(runtime, basis, runtime.services.symbols.clone())
    }

    pub(crate) fn with_symbols(
        runtime: &RelationalRuntime,
        basis: crate::visibility::snapshot_states::SnapshotStateBasis,
        symbols: crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Self {
        let empty_history_schema = basis
            .root()
            .is_none()
            .then(|| std::sync::Arc::new(runtime.config.schema.registry.clone()));
        Self {
            runtime_identity: runtime.runtime_instance_id(),
            current_version: runtime.current_version_id(),
            instrumentation: runtime.services.instrumentation.clone(),
            symbols,
            empty_history_schema,
            resident: None,
            basis,
        }
    }
}
