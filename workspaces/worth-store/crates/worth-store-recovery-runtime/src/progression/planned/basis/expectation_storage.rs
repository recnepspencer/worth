//! Original-request-funded custody of the actual publication expectation.
use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ExpectationPayload {
    pub(super) store: StableStoreIdentity,
    pub(super) checkpoint: Option<PhysicalCheckpointIdentity>,
    pub(super) source_generation: u64,
    pub(super) staging_generation: u64,
    pub(super) plan_identity: [u8; 32],
    pub(super) root_protocol: worth_store::physical_runtime::RecoveryRootProtocolPublicationPlan,
    pub(super) current_selector: worth_store_physical_format::DurableRootSelector,
    pub(super) recovered_root: DurablePhysicalRootManifest,
    pub(super) referenced_artifacts: Box<[RecordArtifactFile]>,
    pub(super) created_artifacts: Box<[RecordArtifactFile]>,
    pub(super) release_topology: Option<RecoveryReleaseTopologyProof>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct RecoveryPublicationExpectation {
    pub(super) backing: ExpectationBacking,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ExpectationBacking(Vec<ExpectationPayload>);
impl ExpectationBacking {
    pub(crate) fn prepare(
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<Self, PlanningMemoryDenial> {
        let bytes = PlanningResidentAllowance::slot_bytes::<ExpectationPayload>(1)?;
        let overflow = || PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX };
        let movement = (std::mem::size_of::<RecoveryPublicationPlan>() as u64)
            .checked_add(bytes.checked_mul(2).ok_or_else(overflow)?)
            .ok_or_else(overflow)?;
        allowance.transient(bytes.checked_add(movement).ok_or_else(overflow)?)?;
        let values = allowance.reserve::<ExpectationPayload>(1)?;
        allowance.transient(movement)?;
        Ok(Self(values))
    }
    pub(super) fn backing_bytes(&self) -> Option<u64> {
        PlanningResidentAllowance::vector_bytes(&self.0).ok()
    }
    pub(super) fn finish(mut self, payload: ExpectationPayload) -> RecoveryPublicationExpectation {
        assert!(self.0.is_empty() && self.0.capacity() >= 1);
        self.0.push(payload);
        RecoveryPublicationExpectation { backing: self }
    }
    pub(super) fn take_release_topology(&mut self) -> Option<RecoveryReleaseTopologyProof> {
        self.0[0].release_topology.take()
    }
}
impl RecoveryPublicationExpectation {
    pub(super) const fn payload(&self) -> &ExpectationPayload {
        &self.backing.0.as_slice()[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expectation_slot_refuses_full_plan_conversion_before_allocating() {
        let payload = std::mem::size_of::<ExpectationPayload>() as u64;
        let plan = std::mem::size_of::<RecoveryPublicationPlan>() as u64;
        let required = 17 + payload + plan + 2 * payload;
        let mut allowance = PlanningResidentAllowance::new(17, required - 1).unwrap();
        assert!(
            matches!(ExpectationBacking::prepare(&mut allowance), Err(PlanningMemoryDenial::RecoveryMemoryBytes { observed }) if observed == required)
        );
        assert_eq!(allowance.used(), 17);
        assert_eq!(allowance.peak(), 17);
    }
    #[test]
    fn expectation_slot_counts_actual_capacity_and_plan_conversion_overlap() {
        let payload = std::mem::size_of::<ExpectationPayload>() as u64;
        let plan = std::mem::size_of::<RecoveryPublicationPlan>() as u64;
        let mut allowance = PlanningResidentAllowance::new(17, u64::MAX).unwrap();
        let backing = ExpectationBacking::prepare(&mut allowance).unwrap();
        let actual = (backing.0.capacity() * std::mem::size_of::<ExpectationPayload>()) as u64;
        assert!(backing.0.is_empty());
        assert!(actual >= payload);
        assert_eq!(backing.backing_bytes(), Some(actual));
        assert_eq!(allowance.used(), 17 + actual);
        assert_eq!(allowance.peak(), 17 + actual + plan + 2 * payload);
        drop(backing);
        allowance.release(actual);
        assert_eq!(allowance.used(), 17);
        assert_eq!(allowance.peak(), 17 + actual + plan + 2 * payload);
    }
}
