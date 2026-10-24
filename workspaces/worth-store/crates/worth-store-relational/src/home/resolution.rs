//! The durable fate of one commit, decided from the journal alone, in
//! Relational's own terms: the artifact digest Relational reported for it.

use worth_relational::facade::durability::{
    RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactDigest,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_store::physical_runtime::{OrderedJournalResolution, ServingPhysicalRuntime};

use crate::recovered_history::RelationalStoreJournalRefusal;

/// The largest artifact a resolution reads back.
const MAXIMUM_ARTIFACT_BYTES: u64 = 256 << 20;

/// What the journal says about one position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalStoreCommitFate {
    /// The journal holds this position with this commit's artifact.
    Committed,
    /// The journal ends below this position; the commit did not become
    /// durable and may be submitted again.
    NotCommitted { head: u64 },
    /// The position holds a different commit.
    Conflict,
    /// A publication of this position has an unknown fate in this process;
    /// reopening the home establishes it.
    Unresolved,
}

pub(super) fn resolve(
    serving: &ServingPhysicalRuntime,
    position: PatchStreamPosition,
    digest: RelationalCanonicalCommitArtifactDigest,
) -> Result<RelationalStoreCommitFate, RelationalStoreJournalRefusal> {
    let journal = serving
        .canonical_commits()
        .map_err(RelationalStoreJournalRefusal::Unavailable)?;
    let unreadable = |failure| RelationalStoreJournalRefusal::Unreadable {
        position: Some(position.0),
        failure,
    };
    // The journal decides presence; its own digest covers the whole stored
    // payload, so the commit's identity is checked on the artifact itself.
    match journal.resolve(position.0, [0; 32]).map_err(unreadable)? {
        OrderedJournalResolution::NotCommitted { head } => {
            return Ok(RelationalStoreCommitFate::NotCommitted { head });
        }
        OrderedJournalResolution::Unresolved => return Ok(RelationalStoreCommitFate::Unresolved),
        OrderedJournalResolution::Committed(_) | OrderedJournalResolution::Conflict(_) => {}
    }
    let payload = journal
        .read(position.0, MAXIMUM_ARTIFACT_BYTES)
        .map_err(unreadable)?;
    let artifact = RelationalCanonicalCommitArtifact::from_untrusted_bytes(payload.into_bytes())
        .map_err(|refusal| RelationalStoreJournalRefusal::ArtifactFrame {
            position: position.0,
            refusal,
        })?;
    Ok(if artifact.payload_digest() == digest {
        RelationalStoreCommitFate::Committed
    } else {
        RelationalStoreCommitFate::Conflict
    })
}
