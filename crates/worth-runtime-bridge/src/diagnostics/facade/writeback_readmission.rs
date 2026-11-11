//! Fresh installation separates diagnostics from every prior runtime alias.
use super::BridgeDiagnosticsFacade;
use std::sync::{Arc, RwLock};
impl BridgeDiagnosticsFacade {
    pub(crate) fn for_authored_installation(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            state: Arc::new(RwLock::new(Default::default())),
        }
    }
    pub(crate) fn readmit_writeback_diagnostics(
        &self,
        owner: &worth_signal::facade::durability::SignalDurableOwner<(), (), (), (), ()>,
    ) {
        self.state
            .write()
            .expect("bridge diagnostics lock poisoned")
            .readmit_writeback_diagnostics(owner.binding().owner());
    }
}
