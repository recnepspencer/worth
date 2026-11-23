//! Match the active configuration to this preparation operation's counters.
use super::RelationalPreparationRuntime;
use crate::authority::commit::phases::prepare::working_state_inputs::WorkingStatePreparation;

impl<'a> From<&'a RelationalPreparationRuntime> for WorkingStatePreparation<'a> {
    fn from(runtime: &'a RelationalPreparationRuntime) -> Self {
        Self {
            adjacency_policy: &runtime.config.storage.adjacency_policy,
            instrumentation: &runtime.services.instrumentation,
        }
    }
}
