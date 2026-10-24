use crate::history::data::{CanonicalCommitEnvelope, CommitId, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalCanonicalPositionAdmissionDenial;

/// A canonical commit that passed current readmission at its stream
/// position. Only Relational readmission doors construct it; holding one
/// confers no runtime authority beyond read access to the admitted commit.
#[derive(Debug, Clone)]
pub struct ReadmittedCanonicalCommit {
    positioned: PositionedCanonicalCommit,
}

impl ReadmittedCanonicalCommit {
    pub(crate) fn exact(positioned: PositionedCanonicalCommit) -> Self {
        Self { positioned }
    }

    pub(crate) fn readmit_current(
        position: PatchStreamPosition,
        canonical: CanonicalCommitEnvelope,
    ) -> Result<Self, RelationalCanonicalPositionAdmissionDenial> {
        crate::runtime::readmit_positioned_canonical_commit(
            position,
            std::sync::Arc::new(canonical),
        )
        .map(Self::exact)
    }

    pub fn position(&self) -> PatchStreamPosition {
        self.positioned.position()
    }
    pub fn commit_id(&self) -> CommitId {
        self.positioned.envelope().commit.commit_id
    }
    pub fn envelope(&self) -> &CanonicalCommitEnvelope {
        self.positioned.envelope()
    }
    pub(crate) fn positioned(&self) -> &PositionedCanonicalCommit {
        &self.positioned
    }
    pub(crate) fn into_positioned(self) -> PositionedCanonicalCommit {
        self.positioned
    }

    #[cfg(test)]
    pub(crate) fn envelope_mut_for_test(&mut self) -> &mut CanonicalCommitEnvelope {
        self.positioned.envelope_mut_for_test()
    }
}
