//! Verify the original journal frame and admit it against the retained index row.
use crate::durability::canonical_commit_artifact::verified_frame::VerifiedCanonicalCommitFrame;
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableColdSource;
use crate::history::commit::envelope_residency::source_readmission::original_frame_stop;
use crate::history::commit::RelationalCommitIndexEntry;
use crate::history::data::PositionedCanonicalCommit;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

pub(crate) fn readmit_journal_commit(
    entry: &RelationalCommitIndexEntry,
    source: &dyn RelationalDurableColdSource,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    limit: u64,
) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
    let failure = |cause| RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Envelope(entry.position),
        cause,
    };
    let read_limit = limit.min(
        crate::durability::canonical_commit_artifact::frame_contract::MAXIMUM_ARTIFACT_BYTES as u64,
    );
    let reservation = execution
        .reserve_retained_memory(read_limit)
        .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
    let bytes = source
        .read_commit(entry.position, read_limit)
        .map_err(failure)?;
    let artifact = VerifiedCanonicalCommitFrame::copy_frame_original(&bytes, execution)
        .map_err(original_frame_stop)?;
    drop(bytes);
    drop(reservation);
    artifact.readmit_indexed(entry, execution)
}
