//! Original acknowledged owner evidence; descriptions cannot construct it.
use super::{publication_binding::SignalCommittedAdvanceRecord, SignalBranchAdvanceArtifact};

pub struct SignalCommittedAdvanceEvidence {
    original: SignalCommittedAdvanceRecord,
}
impl SignalCommittedAdvanceEvidence {
    pub(crate) fn from_committed(original: SignalCommittedAdvanceRecord) -> Self {
        Self { original }
    }
    pub fn artifact(&self) -> &SignalBranchAdvanceArtifact {
        self.original.artifact()
    }
    pub fn position(&self) -> u64 {
        self.original.position().get()
    }
}
impl std::fmt::Debug for SignalCommittedAdvanceEvidence {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("SignalCommittedAdvanceEvidence")
            .field("identity", &self.artifact().identity())
            .field("position", &self.position())
            .field("digest", &self.artifact().digest())
            .finish()
    }
}
