//! Export journal frames through the same cell and sticky damage state as body reads.
use super::{EnvelopeBacking, RelationalCommitEnvelopeResidency};
use crate::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
use crate::durability::canonical_commit_artifact::verified_frame::VerifiedCanonicalCommitFrame as Artifact;
use crate::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};

impl RelationalCommitEnvelopeResidency {
    /// Strict capture reads original frames only; resident body encoding is forbidden.
    pub(crate) fn require_original_artifact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: Option<(
            &std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
            &crate::durability::medium::RelationalDurableReadProtection,
        )>,
    ) -> Result<Artifact, crate::durability::data::recovery_stop::RelationalRecoveryStop> {
        crate::execution::request_custody::check_request(execution).map_err(Stop::Request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(failure) = state.damage {
            return Err(failure.into());
        }
        let position = state.position.as_ref().map_or(
            crate::publication::patch::data::PatchStreamPosition(0),
            |position| position.position(),
        );
        let result = (|| {
            let failure = |cause| {
                crate::durability::data::recovery_stop::RelationalRecoveryStop::from(
                    RelationalStorageUnavailable {
                        subject: RelationalStorageSubject::Envelope(position),
                        cause,
                    },
                )
            };
            if let EnvelopeBacking::SelectedCheckpoint { entry, directory } = &state.backing {
                return Self::selected_artifact(entry, directory, execution, 1 << 30);
            }
            let bytes = match &state.backing {
                EnvelopeBacking::Memory {
                    original: Some(bytes),
                    ..
                } => bytes.clone(),
                EnvelopeBacking::Memory { .. } => return Err(failure(Cause::SourceUnavailable)),
                EnvelopeBacking::SelectedCheckpoint { .. } => {
                    unreachable!("selected source handled before byte dispatch")
                }
                EnvelopeBacking::Journal { entry, source, .. } => {
                    let (actual, protection) =
                        original.ok_or_else(|| failure(Cause::SourceUnavailable))?;
                    if !std::sync::Arc::ptr_eq(actual, source) {
                        return Err(failure(Cause::SourceUnavailable));
                    }
                    protection
                        .read_commit_owned_at(actual, execution, entry.position, 1 << 30)
                        .map_err(|cause| {
                            crate::durability::medium::owned_read_stop::owned_stop(
                                cause,
                                RelationalStorageSubject::Envelope(entry.position),
                            )
                        })?
                }
            };
            let artifact =
                Artifact::from_owned_original(bytes, execution).map_err(original_frame_stop)?;
            if let EnvelopeBacking::Journal { entry, .. } = &state.backing {
                Self::check_frame(entry, &artifact)?;
            }
            Ok(artifact)
        })();
        if let Err(stop) = &result {
            state.remember_original_failure(stop);
        }
        result
    }

    pub(crate) fn export_artifact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Artifact, Denial> {
        crate::execution::request_custody::check_request(execution).map_err(Denial::Request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(failure) = state.damage {
            return Err(Denial::StorageUnavailable(failure));
        }
        if let EnvelopeBacking::Memory {
            original: Some(bytes),
            ..
        } = &state.backing
        {
            return Artifact::from_owned_original(bytes.clone(), execution)
                .map_err(frame_copy_denial);
        }
        if let EnvelopeBacking::SelectedCheckpoint { entry, directory } = &state.backing {
            let result = Self::selected_artifact(
                entry,
                directory,
                execution,
                crate::durability::canonical_commit_artifact::frame_contract::MAXIMUM_ARTIFACT_BYTES
                    as u64,
            );
            if let Err(stop) = &result {
                state.remember_original_failure(stop);
            }
            return result.map_err(export_stop);
        }
        if let EnvelopeBacking::Memory { .. } = &state.backing
            && let Some(commit) = state
                .position
                .as_ref()
                .and_then(|position| position.resident())
        {
            return crate::durability::canonical_commit_artifact::frame_encoding::encode_canonical_frame_for_request(&commit, execution);
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
            crate::durability::canonical_commit_artifact::frame_contract::MAXIMUM_ARTIFACT_BYTES
                as u64;
        let _read = execution
            .reserve_retained_memory(LIMIT)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let read = source.read_commit(entry.position, LIMIT);
        let result = match read {
            Err(cause) => Err(Denial::StorageUnavailable(failure(cause))),
            Ok(bytes) => {
                use crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial as Copy;
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
                crate::execution::request_custody::check_request(execution)
                    .map_err(Denial::Request)?;
                Ok(artifact)
            }
        }
    }
}

fn export_stop(stop: Stop) -> Denial {
    match stop {
        Stop::Durability(error) => match error.storage_unavailable {
            Some(failure) => Denial::StorageUnavailable(failure),
            None => Denial::Readmission(Stop::Durability(error)),
        },
        Stop::Request(cause) => Denial::Readmission(Stop::Request(cause)),
        Stop::AdmissionImage(cause) => Denial::Readmission(Stop::AdmissionImage(cause)),
        Stop::Allocation(cause) => Denial::Readmission(Stop::Allocation(cause)),
        Stop::Content(cause) => Denial::Readmission(Stop::Content(cause)),
    }
}

fn frame_proves_damage(
    cause: &crate::durability::canonical_commit_artifact::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal,
) -> bool {
    use crate::durability::canonical_commit_artifact::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal as Frame;
    match cause {
        Frame::UnsupportedVersion(_) | Frame::SizeLimit { .. } => false,
        Frame::InvalidVersion
        | Frame::InvalidPayloadShape
        | Frame::UnrecognizedMagic
        | Frame::TruncatedHeader { .. }
        | Frame::TruncatedPayload { .. }
        | Frame::TrailingBytes { .. }
        | Frame::DigestMismatch { .. } => true,
    }
}

fn frame_copy_denial(
    cause: crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial,
) -> Denial {
    use crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial as Copy;
    match cause {
        Copy::Frame(cause) => Denial::Frame(cause),
        Copy::Request(cause) => Denial::Request(cause),
        Copy::Allocation(cause) => Denial::Allocation(cause),
    }
}

pub(super) fn original_frame_stop(
    cause: crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial,
) -> Stop {
    use crate::durability::canonical_commit_artifact::frame_contract::ControlledFrameCopyDenial as Copy;
    match cause {
        Copy::Request(cause) => Stop::Request(cause),
        Copy::Allocation(cause) => Stop::Allocation(cause),
        Copy::Frame(cause) => {
            crate::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause)
                .into()
        }
    }
}
