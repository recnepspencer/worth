use std::sync::{Arc, RwLock};

use crate::config::data::RelationalExecutionModel;
use crate::durability::data::DurabilityMode;
use crate::runtime::{RelationalRuntimeConfig, SchemaContractRuntimeSubsystem};

/// The configuration one relational runtime is operating under, together with
/// the schema contract runtime derived from it.
///
/// Both travel as one value so a reader can never observe a registry and a
/// contract runtime that were lowered from different configurations.
#[derive(Debug, Clone)]
pub(crate) struct RelationalRuntimeConfigurationSnapshot {
    pub(crate) config: Arc<RelationalRuntimeConfig>,
    pub(crate) schema_contract_runtime: Arc<SchemaContractRuntimeSubsystem>,
}

/// The one authority for a relational runtime's configuration.
///
/// The owner reconfigures through this cell rather than through exclusive
/// access to the runtime state, because an exclusive runtime borrow does not
/// prove exclusive state access: independently borrowable services read the
/// same configuration concurrently from other threads. Every change replaces
/// the snapshot as one value, so a service holding a snapshot keeps reading a
/// coherent configuration for the whole operation and no second copy has to be
/// kept in step.
#[derive(Debug)]
pub(crate) struct RelationalRuntimeConfiguration {
    state: Arc<RwLock<RelationalRuntimeConfigurationSnapshot>>,
}

/// Cloneable read binding carried by narrow runtime-owned services.
#[derive(Debug, Clone)]
pub(crate) struct RelationalRuntimeConfigurationBinding {
    state: Arc<RwLock<RelationalRuntimeConfigurationSnapshot>>,
}

impl RelationalRuntimeConfiguration {
    pub(in crate::runtime) fn new(
        config: RelationalRuntimeConfig,
        schema_contract_runtime: SchemaContractRuntimeSubsystem,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(RelationalRuntimeConfigurationSnapshot {
                config: Arc::new(config),
                schema_contract_runtime: Arc::new(schema_contract_runtime),
            })),
        }
    }

    pub(crate) fn binding(&self) -> RelationalRuntimeConfigurationBinding {
        RelationalRuntimeConfigurationBinding {
            state: Arc::clone(&self.state),
        }
    }

    /// The configuration in force right now.
    ///
    /// One read acquisition and two pointer clones. Hoist it once per operation
    /// rather than reaching for it per field: the returned value stays coherent
    /// for as long as the caller holds it, and no lock is held after it returns.
    pub(crate) fn snapshot(&self) -> RelationalRuntimeConfigurationSnapshot {
        self.read().clone()
    }

    pub(in crate::runtime) fn set_execution_model(
        &self,
        execution_model: RelationalExecutionModel,
    ) {
        let mut state = self.write();
        Arc::make_mut(&mut state.config).execution.execution_model = execution_model;
    }

    pub(in crate::runtime) fn set_durability_mode(&self, mode: DurabilityMode) {
        let mut state = self.write();
        Arc::make_mut(&mut state.config).durability.policy.mode = mode;
    }

    /// Owner-side configuration edit reserved for runtime test support.
    #[cfg(test)]
    pub(in crate::runtime) fn update(&self, edit: impl FnOnce(&mut RelationalRuntimeConfig)) {
        let mut state = self.write();
        edit(Arc::make_mut(&mut state.config));
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, RelationalRuntimeConfigurationSnapshot> {
        self.state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> std::sync::RwLockWriteGuard<'_, RelationalRuntimeConfigurationSnapshot> {
        self.state
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl RelationalRuntimeConfigurationBinding {
    /// Hold the configuration epoch through preparation or publication. The owner
    /// takes the matching write guard across initial branch and schema cutover.
    pub(crate) fn operation(
        &self,
    ) -> std::sync::RwLockReadGuard<'_, RelationalRuntimeConfigurationSnapshot> {
        self.state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(in crate::runtime) fn initial_installation(
        &self,
    ) -> std::sync::RwLockWriteGuard<'_, RelationalRuntimeConfigurationSnapshot> {
        self.state
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn snapshot(&self) -> RelationalRuntimeConfigurationSnapshot {
        self.state
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}
