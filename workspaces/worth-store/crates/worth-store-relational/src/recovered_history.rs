//! The journal tail a recovering home hands to Relational: every durable
//! position after the selected checkpoint's coverage, in order, as untrusted
//! artifact bytes. Store checks only its own journal frame and the
//! artifact frame; Relational readmits the commits.

use worth_relational::facade::durability::{
    RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactFrameRefusal,
};
use worth_store::physical_runtime::{
    CanonicalCommitJournalDenial, CanonicalCommitReadFailure, ServingPhysicalRuntime,
};

/// The largest artifact one journal read admits. A larger record is refused
/// rather than allocated.
const MAXIMUM_ARTIFACT_BYTES: u64 = 256 << 20;

/// Why the recovered journal could not be handed to Relational.
#[derive(Debug)]
#[non_exhaustive]
pub enum RelationalStoreJournalRefusal {
    /// The serving Store would not lend its journal.
    Unavailable(CanonicalCommitJournalDenial),
    /// The journal or one of its records could not be read or failed Store's
    /// own frame, linkage or digest checks.
    Unreadable {
        position: Option<u64>,
        failure: CanonicalCommitReadFailure,
    },
    /// A journal record's payload is not an intact Relational artifact frame.
    ArtifactFrame {
        position: u64,
        refusal: RelationalCanonicalCommitArtifactFrameRefusal,
    },
    /// The selected checkpoint covers positions the journal does not hold.
    CheckpointAheadOfJournal { covered: u64, head: u64 },
}

/// Every artifact after `covered`, oldest first.
pub(crate) fn journal_tail(
    serving: &ServingPhysicalRuntime,
    covered: u64,
) -> Result<Vec<RelationalCanonicalCommitArtifact>, RelationalStoreJournalRefusal> {
    let journal = serving
        .canonical_commits()
        .map_err(RelationalStoreJournalRefusal::Unavailable)?;
    let head = journal
        .head()
        .map_err(|failure| RelationalStoreJournalRefusal::Unreadable {
            position: None,
            failure,
        })?
        .map_or(0, |entry| entry.position());
    if covered > head {
        return Err(RelationalStoreJournalRefusal::CheckpointAheadOfJournal { covered, head });
    }
    let mut tail = Vec::new();
    for position in covered + 1..=head {
        let payload = journal
            .read(position, MAXIMUM_ARTIFACT_BYTES)
            .map_err(|failure| RelationalStoreJournalRefusal::Unreadable {
                position: Some(position),
                failure,
            })?;
        let artifact = RelationalCanonicalCommitArtifact::from_untrusted_bytes(
            payload.into_bytes(),
        )
        .map_err(|refusal| RelationalStoreJournalRefusal::ArtifactFrame { position, refusal })?;
        tail.push(artifact);
    }
    Ok(tail)
}
