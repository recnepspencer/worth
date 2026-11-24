//! Authenticate the original history member against the retained Branch source.
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::history::commit::RelationalCommitEnvelopeResidency;
use crate::history::data::PositionedCanonicalCommit;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};

/// Source admission validates original bytes even when the logical body is warm.
pub(crate) fn readmit_original_positioned(
    residency: &RelationalCommitEnvelopeResidency,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
    crate::execution::request_custody::check_request(execution)
        .map_err(RelationalRecoveryStop::Request)?;
    if let Some(failure) = residency.original_source_damage() {
        return Err(failure.into());
    }
    let (entry, _) = residency
        .source_placement()
        .ok_or_else(|| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(
                residency
                    .captured_position()
                    .unwrap_or(crate::publication::patch::data::PatchStreamPosition(0)),
            ),
            cause: Cause::SourceUnavailable,
        })?;
    // Only Journal needs a new prefix read. Selected uses its retained cut.
    let journal = residency.journal_placement().map(|(_, source)| source);
    let protection = journal
        .as_ref()
        .map(|source| {
            crate::durability::medium::RelationalDurableReadProtection::capture_when_available(
                source, execution,
            )
            .map_err(|cause| {
                crate::durability::medium::owned_read_stop::owned_stop(
                    cause,
                    RelationalStorageSubject::Envelope(entry.position),
                )
            })
        })
        .transpose()?;
    let artifact = crate::durability::canonical_commit_artifact::retained_frame_reads::require_original_artifact(&residency, execution, journal.as_ref().zip(protection.as_ref()))?;
    let result = artifact.readmit_indexed(&entry, execution);
    if let Err(stop) = &result {
        residency.remember_original_failure(stop);
    }
    result
}
