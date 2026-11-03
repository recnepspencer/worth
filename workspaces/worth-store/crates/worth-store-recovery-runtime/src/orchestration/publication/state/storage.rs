//! Original-request-funded custody of the actual publication state.
use super::{PublicationPayload, PublicationState};
use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};

pub(crate) struct PublicationStateBacking(Vec<PublicationPayload>);
impl PublicationStateBacking {
    pub(crate) fn prepare(
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<Self, PlanningMemoryDenial> {
        Ok(Self(allowance.reserve::<PublicationPayload>(1)?))
    }
    #[cfg(test)]
    pub(crate) fn backing_bytes(&self) -> Result<u64, PlanningMemoryDenial> {
        PlanningResidentAllowance::vector_bytes(&self.0)
    }
    pub(super) fn finish(mut self, payload: PublicationPayload) -> PublicationState {
        assert!(self.0.is_empty() && self.0.capacity() >= 1);
        self.0.push(payload);
        PublicationState { backing: self }
    }
    pub(super) const fn payload(&self) -> &PublicationPayload {
        &self.0.as_slice()[0]
    }
    pub(super) fn into_payload(mut self) -> PublicationPayload {
        assert_eq!(self.0.len(), 1, "sealed payload owner contains one value");
        self.0.remove(0)
    }
}
