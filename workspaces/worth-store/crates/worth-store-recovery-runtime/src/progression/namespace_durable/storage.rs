//! Original-request-funded storage of the actual published namespace payload.
use super::{NamespaceDurableState, RecoveryPublicationExpectation};
use crate::entry::{
    PhysicalRecoveryPublicationCounters, PhysicalRecoveryPublicationSettlementLedger,
};
use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};

pub(crate) struct NamespaceDurablePayload {
    pub(crate) state: NamespaceDurableState,
    pub(crate) expectation: RecoveryPublicationExpectation,
    pub(crate) publication_counters: PhysicalRecoveryPublicationCounters,
    pub(crate) publication_settlement: PhysicalRecoveryPublicationSettlementLedger,
}

pub(crate) struct NamespaceDurableBacking(Vec<NamespaceDurablePayload>);

impl NamespaceDurableBacking {
    pub(crate) fn prepare(
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<Self, PlanningMemoryDenial> {
        Ok(Self(allowance.reserve::<NamespaceDurablePayload>(1)?))
    }
    #[cfg(test)]
    pub(crate) fn backing_bytes(&self) -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::vector_bytes(&self.0)
    }
    pub(crate) fn finish(
        mut self,
        payload: NamespaceDurablePayload,
    ) -> super::NamespaceDurablePhysicalRecovery {
        self.0.push(payload);
        super::NamespaceDurablePhysicalRecovery { backing: self }
    }
    pub(super) const fn payload(&self) -> &NamespaceDurablePayload {
        &self.0.as_slice()[0]
    }
    pub(super) fn into_payload(mut self) -> NamespaceDurablePayload {
        self.0
            .pop()
            .expect("published namespace contains its admitted payload")
    }
}
