//! Original-request-funded custody of the actual freshly reopened payload.
use super::ReopenedPhysicalRecovery;
use crate::entry::{
    PhysicalRecoveryPublicationCounters, PhysicalRecoveryPublicationSettlementLedger,
    PhysicalRecoveryReopenCounters,
};
use crate::progression::{NamespaceDurableState, RecoveryPublicationExpectation};
use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};
use worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen;

pub(crate) struct ReopenedPayload {
    pub(crate) state: NamespaceDurableState<crate::progression::CustodyState>,
    pub(crate) expectation: RecoveryPublicationExpectation,
    pub(crate) publication_counters: PhysicalRecoveryPublicationCounters,
    pub(crate) publication_settlement: PhysicalRecoveryPublicationSettlementLedger,
    pub(crate) reopened: Option<CompletedPhysicalRecoveryFreshReopen>,
    pub(crate) reopen_counters: PhysicalRecoveryReopenCounters,
}

pub(crate) struct ReopenedBacking(Vec<ReopenedPayload>);
impl ReopenedBacking {
    pub(crate) fn prepare(
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<Self, PlanningMemoryDenial> {
        Ok(Self(allowance.reserve::<ReopenedPayload>(1)?))
    }
    pub(crate) fn backing_bytes(&self) -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::vector_bytes(&self.0)
    }
    pub(crate) fn finish(mut self, payload: ReopenedPayload) -> ReopenedPhysicalRecovery {
        self.0.push(payload);
        ReopenedPhysicalRecovery { backing: self }
    }
    pub(super) const fn payload(&self) -> &ReopenedPayload {
        &self.0.as_slice()[0]
    }
    pub(super) fn into_payload(mut self) -> ReopenedPayload {
        self.0
            .pop()
            .expect("reopened owner contains its admitted payload")
    }
    pub(super) fn take_fresh_reopen(&mut self) -> CompletedPhysicalRecoveryFreshReopen {
        self.0[0]
            .reopened
            .take()
            .expect("cleanup entry consumes fresh reopen exactly once")
    }
    pub(super) fn release_admitted_wal(&mut self) {
        self.0[0].state.integrity.release_admitted_wal();
    }
}
