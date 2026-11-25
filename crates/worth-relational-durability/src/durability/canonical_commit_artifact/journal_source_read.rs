//! Verify the original journal frame and admit it against the retained index row.
use crate::durability::canonical_commit_artifact::verified_frame::VerifiedCanonicalCommitFrame;
use branch::history::commit::envelope_residency::source_readmission::original_frame_stop;
use branch::history::commit::index_entry::RelationalCommitIndexEntry;
use branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable,
};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::durability::medium::owned_read_stop::owned_stop;
use worth_relational_storage::internal_port::durability::medium::read_protection::RelationalDurableReadProtection;

pub fn readmit_journal_commit(
    entry: &RelationalCommitIndexEntry,
    source: &std::sync::Arc<dyn RelationalDurableColdSource>,
    protection: Option<&RelationalDurableReadProtection>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    limit: u64,
) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
    let failure = |cause| RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Envelope(entry.position),
        cause,
    };
    let read_limit = limit.min(
        contracts::durability::canonical_commit_artifact::frame_contract::MAXIMUM_ARTIFACT_BYTES
            as u64,
    );
    if let Some(protection) = protection {
        // The protected owned read charges the actual record bytes, never the
        // complete bounded temporary a provider without length proof needs.
        let bytes = protection
            .read_commit_owned_at(source, execution, entry.position, read_limit)
            .map_err(|cause| owned_stop(cause, RelationalStorageSubject::Envelope(entry.position)))?;
        let artifact = VerifiedCanonicalCommitFrame::from_owned_original(bytes, execution)
            .map_err(original_frame_stop)?;
        return artifact.readmit_indexed(entry, execution);
    }
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
