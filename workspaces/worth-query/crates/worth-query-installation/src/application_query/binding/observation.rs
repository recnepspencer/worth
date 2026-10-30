//! Read-only custody of the binding selected by the actual installed catalog.
use std::sync::Arc;

use super::WorthQueryCompiledApplicationQueryBinding;

mod inspection;
pub use inspection::WorthQueryInstalledApplicationInspectionContract;

/// Original installed interpretation, without a Native reader or permission.
///
/// Equality identifies the same immutable catalog binding, including its schema
/// generation and complete descriptor. Equal query identities alone cannot
/// produce this correspondence. Consumers cannot construct an observation.
#[derive(Clone)]
pub struct WorthQueryInstalledApplicationQueryBindingObservation {
    binding: Arc<WorthQueryCompiledApplicationQueryBinding>,
}

impl WorthQueryInstalledApplicationQueryBindingObservation {
    pub(in crate::application_query) fn new(
        binding: Arc<WorthQueryCompiledApplicationQueryBinding>,
    ) -> Self {
        Self { binding }
    }

    /// Retains this exact binding's optional inspection meaning. This supplies
    /// no read, disclosure, history or materialization permission.
    pub fn inspection_contract(&self) -> Option<WorthQueryInstalledApplicationInspectionContract> {
        WorthQueryInstalledApplicationInspectionContract::retain(self)
    }
}

impl PartialEq for WorthQueryInstalledApplicationQueryBindingObservation {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.binding, &other.binding)
    }
}

impl Eq for WorthQueryInstalledApplicationQueryBindingObservation {}

impl std::fmt::Debug for WorthQueryInstalledApplicationQueryBindingObservation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryInstalledApplicationQueryBindingObservation")
            .finish_non_exhaustive()
    }
}
