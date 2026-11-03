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
    pub(crate) fn export_artifact_original(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Artifact, Denial> {
        crate::execution::check_request(execution).map_err(Denial::Request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(failure) = state.damage {
            return Err(Denial::StorageUnavailable(failure));
        }
        if let Some(commit) = state
            .position
            .as_ref()
            .and_then(|position| position.resident())
        {
            return Artifact::from_positioned_request(&commit, execution);
        }
        let EnvelopeBacking::Journal { entry, source, .. } = &state.backing else {
            return Err(Denial::PayloadEncoding {
                detail: "memory envelope has no performed position".into(),
            });
        };
        let failure = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(entry.position),
            cause,
        };
        // The existing port certifies no length before reading. Reserve its
        // complete bounded temporary before asking it to allocate; never guess
        // a size from unrelated current-root content or read uncharged first.
        const LIMIT: u64 =
            crate::durability::canonical_commit_artifact::MAXIMUM_ARTIFACT_BYTES as u64;
        let _read = execution
            .reserve_retained_memory(LIMIT)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let read = source.read_commit(entry.position, LIMIT);
        let result = match read {
            Err(cause) => Err(Denial::StorageUnavailable(failure(cause))),
            Ok(bytes) => {
                use crate::durability::canonical_commit_artifact::ControlledFrameCopyDenial as Copy;
                Artifact::copy_frame_original(&bytes, execution)
                    .map_err(|cause| match cause {
                        Copy::Request(cause) => Denial::Request(cause),
                        Copy::Allocation(cause) => Denial::Allocation(cause),
                        Copy::Frame(cause) => Denial::Frame(cause),
                    })
                    .and_then(|artifact| {
                        if artifact.position() != entry.position
                            || artifact.commit_id() != entry.commit.commit_id
                            || artifact.payload_digest().as_bytes() != &entry.payload_digest
                        {
                            return Err(Denial::StorageUnavailable(failure(
                                Cause::IntegrityMismatch,
                            )));
                        }
                        Ok(artifact)
                    })
            }
        };
        if matches!(&result, Err(Denial::Frame(cause)) if frame_proves_damage(cause)) {
            state.damage = Some(failure(Cause::Damaged));
        }
        if let Err(Denial::StorageUnavailable(failure)) = &result
            && failure.cause.is_proven_damage()
        {
            state.damage = Some(*failure);
        }
        match result {
            Err(cause) => Err(cause),
            Ok(artifact) => {
                crate::execution::check_request(execution).map_err(Denial::Request)?;
                Ok(artifact)
            }
        }
    }
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
            return Err(Denial::PayloadEncoding {
                detail: "memory envelope has no performed position".to_owned(),
            });
        };
        let failure = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(entry.position),
            cause,
        };
        let result = source
            .read_commit(entry.position, 1 << 30)
            .map_err(failure)
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

fn frame_proves_damage(
    cause: &crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactFrameRefusal,
) -> bool {
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactFrameRefusal as Frame;
    match cause {
        Frame::UnsupportedVersion { .. } | Frame::SizeLimit { .. } => false,
        Frame::UnrecognizedMagic
        | Frame::TruncatedHeader { .. }
        | Frame::TruncatedPayload { .. }
        | Frame::TrailingBytes { .. }
        | Frame::DigestMismatch { .. } => true,
    }
}
