//! Finite inspection of one immutable committed patch, without selecting a branch head.

use std::num::NonZeroUsize;

use crate::history::data::CommitId;
use crate::publication::patch::data::PublishedAuthoritativeRecordPatch;

use super::HistoryAccess;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedCanonicalCommitPatchDenial {
    CommitUnavailable,
    WorkExhausted { required_records: usize },
    ReadRequest(crate::execution::RelationalExecutionDenialCause),
}

impl HistoryAccess<'_> {
    /// Read exactly one canonical commit's changed records. A caller must
    /// budget every record before copying any patch evidence. This does not
    /// admit a snapshot or imply that the commit was part of World Performed.
    pub fn bounded_canonical_commit_patches(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
        maximum_records: NonZeroUsize,
    ) -> Result<Vec<PublishedAuthoritativeRecordPatch>, BoundedCanonicalCommitPatchDenial> {
        use BoundedCanonicalCommitPatchDenial as Denial;
        crate::execution::check_request(execution).map_err(Denial::ReadRequest)?;
        let envelope = match self.runtime.history.commit_artifact(commit_id) {
            Some(artifact) => artifact
                .demand_envelope(execution, 1 << 30)
                .map_err(|failure| Denial::ReadRequest(failure.into()))?
                .clone(),
            None => self
                .runtime
                .history
                .canonical_envelope(commit_id)
                .ok_or(Denial::CommitUnavailable)?,
        };
        let patches = &envelope.patch.authoritative_record_patches;
        if patches.len() > maximum_records.get() {
            return Err(BoundedCanonicalCommitPatchDenial::WorkExhausted {
                required_records: patches.len(),
            });
        }
        let answer = patches.clone();
        crate::execution::check_request(execution).map_err(Denial::ReadRequest)?;
        Ok(answer)
    }
}
