#[cfg(test)]
use crate::capabilities::DurabilityRead;
use crate::durability::access::retained_recovery_plan::retained_recovery_plan;
use crate::durability::data::{RecoveryPlan, RecoveryVerificationMode};
use crate::runtime::RelationalRuntime;

pub struct DurabilityAccess<'runtime> {
    pub(in crate::durability) runtime: &'runtime RelationalRuntime,
}

impl<'runtime> DurabilityAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    /// Plan reconstruction from the retained canonical log and checkpoints.
    /// Compacted transition gaps are read fallibly from the owned medium;
    /// native capture need not retain a duplicate checkpoint image. A home
    /// reopens through [`Self::recovery_plan_from_source`].
    pub fn recovery_plan(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        verification_mode: RecoveryVerificationMode,
    ) -> Result<RecoveryPlan, crate::durability::data::RelationalRecoveryStop> {
        execution.check_live().map_err(|stop| {
            crate::durability::data::RelationalRecoveryStop::Request(stop.into())
        })?;
        retained_recovery_plan(self.runtime, execution, verification_mode)
    }

    /// Readmit what a durable medium recovered for this home and plan its
    /// reconstruction. Every artifact is readmitted, the tail must continue
    /// exactly from the checkpoint's coverage, and any refusal stops recovery
    /// before service. Pass the plan to `durability_recovery().recover`.
    pub fn recovery_plan_from_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: crate::durability::medium::RelationalDurableRecoverySource,
        verification_mode: RecoveryVerificationMode,
    ) -> Result<RecoveryPlan, crate::durability::data::RelationalRecoveryStop> {
        execution.check_live().map_err(|stop| {
            crate::durability::data::RelationalRecoveryStop::Request(stop.into())
        })?;
        super::medium_recovery_plan::medium_recovery_plan(
            execution,
            self.runtime,
            source,
            verification_mode,
        )
    }

    pub fn durable_commit_count(&self) -> usize {
        self.runtime.durability.log_len()
    }

    #[cfg(test)]
    pub(crate) fn durable_log(
        &self,
    ) -> Vec<std::sync::Arc<crate::history::data::PositionedCanonicalCommit>> {
        DurabilityRead::durable_log(self.runtime)
    }
}
