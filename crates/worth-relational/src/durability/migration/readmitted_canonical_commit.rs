use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;

/// Current recovery input retaining its exact positioned canonical admission.
#[derive(Debug, Clone)]
pub(crate) struct ReadmittedCanonicalCommit {
    positioned: PositionedCanonicalCommit,
}

impl ReadmittedCanonicalCommit {
    pub(crate) fn exact(positioned: PositionedCanonicalCommit) -> Self {
        Self { positioned }
    }

    pub(crate) fn readmit_current(
        position: PatchStreamPosition,
        canonical: CanonicalCommitEnvelope,
    ) -> Result<Self, String> {
        crate::runtime::readmit_positioned_canonical_commit(
            position,
            std::sync::Arc::new(canonical),
        )
        .map(Self::exact)
    }

    pub(crate) fn position(&self) -> PatchStreamPosition {
        self.positioned.position()
    }
    pub(crate) fn envelope(&self) -> &CanonicalCommitEnvelope {
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
