//! Capture immutable projection metadata while the native owner is borrowed.
use super::VisibilityProjectionView;
use crate::runtime::state::subsystems::{
    history::HistorySubsystem,
    services::{instrumentation::RuntimeInstrumentation, symbol_table::RuntimeSymbolTable},
};
use crate::schema::data::RelationalSchemaRegistry;
use crate::visibility::snapshot_states::SnapshotStateBasis;

#[derive(Clone, Copy)]
pub(crate) struct ProjectionInputs<'a> {
    pub(crate) runtime_identity: u64,
    pub(crate) history: &'a HistorySubsystem,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
    pub(crate) symbols: &'a RuntimeSymbolTable,
    pub(crate) empty_history_schema: &'a RelationalSchemaRegistry,
}

impl VisibilityProjectionView {
    pub(crate) fn new<'a>(
        inputs: impl Into<ProjectionInputs<'a>>,
        basis: SnapshotStateBasis,
    ) -> Self {
        let inputs = inputs.into();
        let symbols = inputs.symbols.clone();
        let empty_history_schema = basis
            .root()
            .is_none()
            .then(|| std::sync::Arc::new(inputs.empty_history_schema.clone()));
        Self {
            runtime_identity: inputs.runtime_identity,
            current_version: inputs.history.current_version_id(),
            instrumentation: inputs.instrumentation.clone(),
            symbols,
            empty_history_schema,
            resident: None,
            basis,
        }
    }
}
