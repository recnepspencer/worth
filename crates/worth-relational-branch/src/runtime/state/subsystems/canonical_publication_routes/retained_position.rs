//! Retain admitted position provenance without retaining its envelope payload.
use super::position_admission::CanonicalPositionAdmission;
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;
use std::sync::{Arc, Weak};

#[derive(Debug, Clone)]
pub struct RetainedCanonicalPosition {
    position: PatchStreamPosition,
    envelope: Weak<CanonicalCommitEnvelope>,
    // LAST: release the actual Weak before its shared fixed allocation bill.
    _control_allocation: Option<Arc<crate::history::data::CanonicalEnvelopeControlAllocation>>,
}
impl RetainedCanonicalPosition {
    /// The sealed input binds this exact body allocation to its position.
    pub fn from_positioned(commit: &PositionedCanonicalCommit) -> Self {
        Self {
            position: commit.position(),
            envelope: Arc::downgrade(commit.canonical_arc()),
            _control_allocation: commit.envelope().control_allocation.clone(),
        }
    }
    pub fn position(&self) -> PatchStreamPosition {
        self.position
    }
    pub fn resident(&self) -> Option<PositionedCanonicalCommit> {
        CanonicalPositionAdmission::retained(self).map(PositionedCanonicalCommit::admit)
    }
    pub(super) fn upgrade_parts(
        &self,
    ) -> Option<(PatchStreamPosition, Arc<CanonicalCommitEnvelope>)> {
        self.envelope
            .upgrade()
            .map(|envelope| (self.position, envelope))
    }
}
