//! Both later handoff phases are admitted before any recovery effects.
use super::completion::PendingCompletion;
use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};
use worth_store::physical_runtime::RecoveredPhysicalRuntimeCore;
pub(crate) struct RecoveryHandoffBacking {
    pub(super) pending: Vec<PendingCompletion>,
    pub(super) core: Vec<RecoveredPhysicalRuntimeCore>,
}
impl RecoveryHandoffBacking {
    pub(crate) fn prepare(
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<Self, PlanningMemoryDenial> {
        Ok(Self {
            pending: allowance.reserve(1)?,
            core: allowance.reserve(1)?,
        })
    }
    pub(crate) fn required_bytes() -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::slot_bytes::<PendingCompletion>(1)?
            .checked_add(PlanningResidentAllowance::slot_bytes::<
                RecoveredPhysicalRuntimeCore,
            >(1)?)
            .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX })
    }
    pub(crate) fn backing_bytes(&self) -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::vector_bytes(&self.pending)?
            .checked_add(PlanningResidentAllowance::vector_bytes(&self.core)?)
            .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX })
    }
}
#[cfg(test)]
#[path = "backing_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
