use crate::config::data::RelationalRuntimeConfig;
use crate::runtime::RelationalRuntime;

pub(crate) trait RuntimeConfigSource {
    /// The configuration in force for this operation.
    fn runtime_config(&self) -> &RelationalRuntimeConfig;
}

impl RuntimeConfigSource for RelationalRuntime {
    fn runtime_config(&self) -> &RelationalRuntimeConfig {
        &self.config
    }
}

pub(crate) trait RuntimeIdentitySource {
    fn runtime_name(&self) -> &str;
}

impl RuntimeIdentitySource for RelationalRuntime {
    fn runtime_name(&self) -> &str {
        &self.config.execution.runtime_name
    }
}
