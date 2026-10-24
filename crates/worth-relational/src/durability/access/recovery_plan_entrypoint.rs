use crate::capabilities::{DurabilityRead, RuntimeConfigSource};
use crate::durability::access::in_memory_recovery_plan::in_memory_recovery_plan;
use crate::durability::access::persisted_recovery_plan::persisted_recovery_plan;
use crate::durability::data::{DurabilityMode, RecoveryPlan, RecoveryVerificationMode};
use crate::runtime::RelationalRuntime;

pub struct DurabilityAccess<'runtime> {
    pub(in crate::durability) runtime: &'runtime RelationalRuntime,
}

impl<'runtime> DurabilityAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub fn recovery_plan(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        verification_mode: RecoveryVerificationMode,
    ) -> Result<RecoveryPlan, crate::durability::data::RelationalRecoveryStop> {
        execution.check_live().map_err(|stop| {
            crate::durability::data::RelationalRecoveryStop::Request(stop.into())
        })?;
        match self.runtime.runtime_config().durability.policy.mode {
            DurabilityMode::InMemoryCanonical => {
                Ok(in_memory_recovery_plan(self.runtime, verification_mode))
            }
            DurabilityMode::PersistedSegmentedLocalFs => {
                persisted_recovery_plan(execution, self.runtime, verification_mode)
            }
        }
    }

    pub fn durable_commit_count(&self) -> usize {
        DurabilityRead::durable_log(self.runtime).len()
    }

    #[cfg(test)]
    pub(crate) fn durable_log(
        &self,
    ) -> Vec<std::sync::Arc<crate::history::data::PositionedCanonicalCommit>> {
        DurabilityRead::durable_log(self.runtime)
    }
}
