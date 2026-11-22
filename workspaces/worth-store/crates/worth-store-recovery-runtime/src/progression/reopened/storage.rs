//! Original-request-funded custody of the actual freshly reopened payload.
use super::ReopenedPhysicalRecovery;
use crate::entry::{
    PhysicalRecoveryPublicationCounters, PhysicalRecoveryPublicationSettlementLedger,
    PhysicalRecoveryReopenCounters,
};
use crate::progression::{NamespaceDurableState, RecoveryPublicationExpectation};
use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};
use worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen;
use worth_store_recovery_physics::VerifiedEffectiveReleaseHeadRosterV14;

pub(crate) struct ReopenedPayload {
    pub(crate) state: NamespaceDurableState<crate::progression::CustodyState>,
    pub(crate) expectation: RecoveryPublicationExpectation,
    pub(crate) publication_counters: PhysicalRecoveryPublicationCounters,
    pub(crate) publication_settlement: PhysicalRecoveryPublicationSettlementLedger,
    pub(crate) reopened: Option<CompletedPhysicalRecoveryFreshReopen>,
    pub(crate) reopen_counters: PhysicalRecoveryReopenCounters,
}

pub(crate) struct ReopenedBacking {
    payload: Vec<ReopenedPayload>,
    pending_effective: Vec<VerifiedEffectiveReleaseHeadRosterV14>,
    handoff: crate::orchestration::RecoveryHandoffBacking,
}
impl ReopenedBacking {
    pub(crate) fn prepare(
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<Self, PlanningMemoryDenial> {
        Ok(Self {
            payload: allowance.reserve::<ReopenedPayload>(1)?,
            pending_effective: allowance.reserve::<VerifiedEffectiveReleaseHeadRosterV14>(1)?,
            handoff: crate::orchestration::RecoveryHandoffBacking::prepare(allowance)?,
        })
    }
    pub(crate) fn required_bytes() -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::slot_bytes::<ReopenedPayload>(1)?
            .checked_add(PlanningResidentAllowance::slot_bytes::<
                VerifiedEffectiveReleaseHeadRosterV14,
            >(1)?)
            .and_then(|bytes| {
                bytes.checked_add(
                    crate::orchestration::RecoveryHandoffBacking::required_bytes().ok()?,
                )
            })
            .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX })
    }
    pub(crate) fn backing_bytes(&self) -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::vector_bytes(&self.payload)?
            .checked_add(PlanningResidentAllowance::vector_bytes(
                &self.pending_effective,
            )?)
            .and_then(|bytes| bytes.checked_add(self.handoff.backing_bytes().ok()?))
            .ok_or(PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX })
    }
    pub(crate) fn take_pending_effective_slot(
        &mut self,
    ) -> Vec<VerifiedEffectiveReleaseHeadRosterV14> {
        let slot = std::mem::take(&mut self.pending_effective);
        assert!(slot.is_empty() && slot.capacity() >= 1);
        slot
    }
    pub(crate) fn finish(mut self, payload: ReopenedPayload) -> ReopenedPhysicalRecovery {
        assert!(self.payload.is_empty() && self.payload.capacity() >= 1);
        // Non-pending routes do not consume the prepaid effective-roster slot;
        // release it immediately so retained-memory sampling matches live backing.
        self.pending_effective = Vec::new();
        self.payload.push(payload);
        ReopenedPhysicalRecovery { backing: self }
    }
    pub(super) const fn payload(&self) -> &ReopenedPayload {
        &self.payload.as_slice()[0]
    }
    pub(super) fn payload_backing_bytes(&self) -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::vector_bytes(&self.payload)
    }
    pub(super) fn into_handoff_parts(
        mut self,
    ) -> (
        ReopenedPayload,
        crate::orchestration::RecoveryHandoffBacking,
    ) {
        assert_eq!(self.payload.len(), 1, "one actual reopened payload");
        (
            self.payload.pop().expect("actual reopened payload"),
            self.handoff,
        )
    }
    pub(super) fn take_fresh_reopen(&mut self) -> CompletedPhysicalRecoveryFreshReopen {
        self.payload[0]
            .reopened
            .take()
            .expect("cleanup entry consumes fresh reopen exactly once")
    }
    #[cfg(feature = "certification-test-authority")]
    pub(super) fn retain_cleanup_media_handle_for_certification(&mut self) -> bool {
        self.payload[0]
            .state
            .authority
            .media
            .certification_retain_cleanup_media_handle()
    }
    pub(super) fn release_admitted_wal(&mut self) {
        self.payload[0].state.integrity.release_admitted_wal();
    }
}
