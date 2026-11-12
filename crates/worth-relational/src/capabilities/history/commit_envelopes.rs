use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::runtime::RelationalRuntime;

/// Pure observation after an operation has carried the required body custody.
pub(crate) trait CommitEnvelopeSource {
    fn resident_commit_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<std::sync::Arc<CanonicalCommitEnvelope>>;
}
impl CommitEnvelopeSource for RelationalRuntime {
    fn resident_commit_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<std::sync::Arc<CanonicalCommitEnvelope>> {
        self.history.canonical_envelope(commit_id)
    }
}
