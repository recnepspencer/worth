use std::sync::{Arc, Mutex};

use crate::runtime::RuntimeComplexityCounters;

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeInstrumentation {
    pub(crate) complexity_counters: Arc<Mutex<RuntimeComplexityCounters>>,
    basis_counters: Arc<Mutex<crate::branch::RelationalBranchBasisCostCounters>>,
    external_retention_terminals:
        Arc<crate::history::retention::RelationalExternalRetentionTerminalAccounting>,
}

impl RuntimeInstrumentation {
    pub(crate) fn preparation_round(&self) -> Self {
        Self {
            complexity_counters: Arc::default(),
            basis_counters: Arc::default(),
            external_retention_terminals: self.external_retention_terminals.clone(),
        }
    }
    pub(crate) fn new() -> Self {
        Self {
            complexity_counters: Arc::new(Mutex::new(RuntimeComplexityCounters::default())),
            basis_counters: Arc::new(Mutex::new(
                crate::branch::RelationalBranchBasisCostCounters::default(),
            )),
            external_retention_terminals: Arc::new(Default::default()),
        }
    }

    pub(crate) fn fork(&self) -> Self {
        Self {
            complexity_counters: Arc::new(Mutex::new(
                self.complexity_counters
                    .lock()
                    .expect("complexity counter lock poisoned")
                    .clone(),
            )),
            basis_counters: Arc::new(Mutex::new(self.basis_counters())),
            external_retention_terminals: Arc::new(Default::default()),
        }
    }

    pub(crate) fn count(&self, update: impl FnOnce(&mut RuntimeComplexityCounters)) {
        update(
            &mut self
                .complexity_counters
                .lock()
                .expect("complexity counter lock poisoned"),
        );
    }

    pub(crate) fn count_basis(
        &self,
        update: impl FnOnce(&mut crate::branch::RelationalBranchBasisCostCounters),
    ) {
        update(
            &mut self
                .basis_counters
                .lock()
                .expect("basis counter lock poisoned"),
        );
    }

    pub(crate) fn basis_counters(&self) -> crate::branch::RelationalBranchBasisCostCounters {
        let mut counters = *self
            .basis_counters
            .lock()
            .expect("basis counter lock poisoned");
        let terminals = self.external_retention_terminals.snapshot();
        counters.external_retention_releases = counters
            .external_retention_releases
            .saturating_add(terminals.total());
        counters.external_retention_drop_releases = counters
            .external_retention_drop_releases
            .saturating_add(terminals.dropped_releases);
        counters
    }

    pub(crate) fn external_retention_terminal_accounting(
        &self,
    ) -> Arc<crate::history::retention::RelationalExternalRetentionTerminalAccounting> {
        Arc::clone(&self.external_retention_terminals)
    }
}
