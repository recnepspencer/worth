use crate::facade::config::RelationalRuntimeProfile;
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::{durable_runtime_with_test_schema, runtime_with_test_schema_profile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeHarnessMode {
    InMemory(RelationalRuntimeProfile),
    Persisted,
}

pub(crate) fn build_runtime(mode: RuntimeHarnessMode) -> RelationalRuntime {
    match mode {
        RuntimeHarnessMode::InMemory(profile) => runtime_with_test_schema_profile(profile),
        RuntimeHarnessMode::Persisted => durable_runtime_with_test_schema(),
    }
}
