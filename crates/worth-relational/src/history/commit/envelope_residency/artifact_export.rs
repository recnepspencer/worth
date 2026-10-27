//! Export journal frames through the same cell and sticky damage state as body reads.
use super::{EnvelopeBacking, RelationalCommitEnvelopeResidency};
use crate::durability::canonical_commit_artifact::{
    RelationalCanonicalCommitArtifact as Artifact,
    RelationalCanonicalCommitArtifactEncodeDenial as Denial,
};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};

impl RelationalCommitEnvelopeResidency {
    pub(crate) fn export_artifact(
        &self,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Artifact, Denial> {
        policy.check_live()?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(failure) = state.damage {
            return Err(Denial::StorageUnavailable(failure));
        }
        if let Some(commit) = state
            .position
            .as_ref()
            .and_then(|position| position.resident())
        {
            return Artifact::from_positioned(&commit, policy);
        }
        let EnvelopeBacking::Journal { entry, source, .. } = &state.backing else {
            unreachable!("memory history retains its performed body")
        };
        let failure = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(entry.position),
            cause,
        };
        let result = source
            .read_commit(entry.position, 1 << 30)
            .map_err(&failure)
            .and_then(|bytes| {
                Artifact::from_untrusted_bytes(bytes).map_err(|_| failure(Cause::Damaged))
            })
            .and_then(|artifact| {
                if artifact.position() != entry.position
                    || artifact.commit_id() != entry.commit.commit_id
                    || artifact.payload_digest().as_bytes() != &entry.payload_digest
                {
                    return Err(failure(Cause::IntegrityMismatch));
                }
                Ok(artifact)
            });
        match result {
            Ok(artifact) => {
                policy.check_live()?;
                Ok(artifact)
            }
            Err(failure) => {
                if failure.cause.is_proven_damage() {
                    state.damage = Some(failure);
                }
                Err(Denial::StorageUnavailable(failure))
            }
        }
    }
}
