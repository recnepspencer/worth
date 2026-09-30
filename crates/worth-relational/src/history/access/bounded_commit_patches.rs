//! Finite inspection of one immutable committed patch, without selecting a branch head.

use std::num::NonZeroUsize;

use crate::history::data::CommitId;
use crate::publication::patch::data::PublishedAuthoritativeRecordPatch;

use super::HistoryAccess;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedCanonicalCommitPatchDenial {
    CommitUnavailable,
    WorkExhausted { required_records: usize },
}

impl HistoryAccess<'_> {
    /// Read exactly one canonical commit's changed records. A caller must
    /// budget every record before copying any patch evidence. This does not
    /// admit a snapshot or imply that the commit was part of World Performed.
    pub fn bounded_canonical_commit_patches(
        &self,
        commit_id: CommitId,
        maximum_records: NonZeroUsize,
    ) -> Result<Vec<PublishedAuthoritativeRecordPatch>, BoundedCanonicalCommitPatchDenial> {
        let envelope = self
            .runtime
            .history
            .commit_artifact(commit_id)
            .map(|artifact| artifact.envelope().clone())
            .or_else(|| self.runtime.history.canonical_envelope(commit_id))
            .ok_or(BoundedCanonicalCommitPatchDenial::CommitUnavailable)?;
        let patches = &envelope.patch.authoritative_record_patches;
        if patches.len() > maximum_records.get() {
            return Err(BoundedCanonicalCommitPatchDenial::WorkExhausted {
                required_records: patches.len(),
            });
        }
        Ok(patches.clone())
    }
}
