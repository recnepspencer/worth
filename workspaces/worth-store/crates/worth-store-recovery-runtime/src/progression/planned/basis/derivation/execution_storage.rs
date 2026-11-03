//! Original-request-funded custody of the completed execution payload.
use super::super::{
    CandidateMaterializationCost, PlanningMemoryDenial, PlanningResidentAllowance,
    RecoveryPublicationPlan, RecoveryQuiescencePlan, RecoveryStagingLayoutPlan,
};

pub(crate) struct ExecutionBasisState {
    pub(crate) staging: RecoveryStagingLayoutPlan,
    pub(crate) publication: RecoveryPublicationPlan,
    pub(crate) quiescence: RecoveryQuiescencePlan,
    pub(crate) candidate_materialization: CandidateMaterializationCost,
    pub(crate) root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    pub(crate) planning_construction_peak: u64,
}

pub(super) struct ExecutionBasisBacking(Vec<ExecutionBasisState>);
pub(crate) struct ExecutionBasis(Vec<ExecutionBasisState>);

impl ExecutionBasisBacking {
    pub(super) fn prepare(
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<Self, PlanningMemoryDenial> {
        let payload = PlanningResidentAllowance::slot_bytes::<ExecutionBasisState>(1)?;
        let combined = payload
            .checked_mul(2)
            .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX })?;
        allowance.transient(combined)?;
        let values = allowance.reserve::<ExecutionBasisState>(1)?;
        allowance.transient(payload)?;
        Ok(Self(values))
    }

    pub(super) fn finish(
        mut self,
        mut state: ExecutionBasisState,
        allowance: &PlanningResidentAllowance,
    ) -> ExecutionBasis {
        state.planning_construction_peak = allowance.peak();
        self.0.push(state);
        ExecutionBasis(self.0)
    }
}

impl ExecutionBasis {
    pub(crate) fn state(&self) -> &ExecutionBasisState {
        &self.0[0]
    }

    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        PlanningResidentAllowance::vector_bytes(&self.0)
            .ok()?
            .checked_add(self.state().staging.owned_heap_bytes()?)?
            .checked_add(self.state().publication.owned_heap_bytes()?)
    }

    pub(crate) fn into_state(mut self) -> ExecutionBasisState {
        self.0
            .pop()
            .expect("execution owner contains its admitted payload")
    }
}
