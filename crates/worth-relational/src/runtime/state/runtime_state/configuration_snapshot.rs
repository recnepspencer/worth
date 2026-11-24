//! One retained receiver configuration and its actual compiled schema inventory.
use crate::config::data::runtime_config::RelationalRuntimeConfig;
use crate::runtime::state::subsystems::schema_contract_runtime::SchemaContractRuntimeSubsystem;
use std::sync::Arc;

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

impl RelationalRuntimeConfigurationSnapshot {
    /// A semantic generation is not an identity for a replaced receiver pair.
    pub(crate) fn same_epoch(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.config, &other.config)
            && Arc::ptr_eq(
                &self.schema_contract_runtime,
                &other.schema_contract_runtime,
            )
    }
}
