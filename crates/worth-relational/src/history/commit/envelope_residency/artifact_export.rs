//! Export journal frames through the same cell and sticky damage state as body reads.
use super::{EnvelopeBacking, RelationalCommitEnvelopeResidency};
use crate::durability::canonical_commit_artifact::{
    RelationalCanonicalCommitArtifact as Artifact,
    RelationalCanonicalCommitArtifactEncodeDenial as Denial,
};
use crate::durability::data::RelationalRecoveryStop as Stop;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};

impl RelationalCommitEnvelopeResidency {
    /// Capture metadata reads the digest the original Memory frame records,
    /// never re-encoding or re-hashing it: the capture verifies each frame
    /// once, when it embeds it, against the digest this returns.
    pub(crate) fn original_memory_recorded_digest(&self) -> Result<[u8; 32], Stop> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let EnvelopeBacking::Memory {
            original: Some(bytes),
            ..
        } = &state.backing
        else {
            return Err(crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                "original Memory artifact unavailable or damaged",
            )
            .into());
        };
        Artifact::recorded_payload_digest(bytes.bytes()).map_err(|cause| {
            original_frame_stop(
                crate::durability::canonical_commit_artifact::ControlledFrameCopyDenial::Frame(
                    cause,
                ),
            )
        })
    }
    /// Strict capture reads original frames only; resident body encoding is forbidden.
    pub(crate) fn require_original_artifact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: Option<(
            &std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
            &crate::durability::medium::RelationalDurableReadProtection,
        )>,
    ) -> Result<Artifact, crate::durability::data::RelationalRecoveryStop> {
        crate::execution::check_request(execution).map_err(Stop::Request)?;
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
                crate::durability::data::RelationalRecoveryStop::from(
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
                            crate::durability::checkpoint_history::capture::owned_stop(
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
        crate::execution::check_request(execution).map_err(Denial::Request)?;
        let journal = {
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
                    crate::durability::canonical_commit_artifact::MAXIMUM_ARTIFACT_BYTES as u64,
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
                return Artifact::from_positioned_request(&commit, execution);
            }
            let EnvelopeBacking::Journal { entry, source, .. } = &state.backing else {
                return Err(Denial::PayloadEncoding {
                    detail: "memory envelope has no performed position".into(),
                });
            };
            (entry.position, std::sync::Arc::clone(source))
        };
        let captured = match crate::durability::medium::RelationalDurableReadProtection::capture_journal_when_available(
            &journal.1, execution,
        ) {
            Ok(protection) => Ok(Some((journal.1, protection))),
            Err(
                crate::durability::medium::RelationalDurableOwnedReadRefusal::UnsupportedProvider,
            ) => Ok(None),
            Err(cause) => Err(owned_export_denial(cause, journal.0)),
        };
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(failure) = state.damage {
            return Err(Denial::StorageUnavailable(failure));
        }
        if let EnvelopeBacking::SelectedCheckpoint { entry, directory } = &state.backing {
            let result = Self::selected_artifact(
                entry,
                directory,
                execution,
                crate::durability::canonical_commit_artifact::MAXIMUM_ARTIFACT_BYTES as u64,
            );
            if let Err(stop) = &result {
                state.remember_original_failure(stop);
            }
            return result.map_err(export_stop);
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
        const LIMIT: u64 =
            crate::durability::canonical_commit_artifact::MAXIMUM_ARTIFACT_BYTES as u64;
        let result = match captured {
            Err(cause) => Err(cause),
            Ok(Some((actual, protection))) if std::sync::Arc::ptr_eq(&actual, source) => {
                match protection.read_commit_owned_at(&actual, execution, entry.position, LIMIT) {
                    Err(cause) => Err(owned_export_denial(cause, entry.position)),
                    Ok(bytes) => Artifact::from_owned_original(bytes, execution)
                        .map_err(frame_copy_denial)
                        .and_then(|artifact| verify_exported_entry(entry, artifact, failure)),
                }
            }
            Ok(_) => {
                // Legacy cold-source fixtures without protected owned reads certify
                // no length before reading; reserve the bounded temporary first.
                let _read = execution
                    .reserve_retained_memory(LIMIT)
                    .map_err(|cause| Denial::Request(cause.into()))?;
                match source.read_commit(entry.position, LIMIT) {
                    Err(cause) => Err(Denial::StorageUnavailable(failure(cause))),
                    Ok(bytes) => Artifact::copy_frame_original(&bytes, execution)
                        .map_err(frame_copy_denial)
                        .and_then(|artifact| verify_exported_entry(entry, artifact, failure)),
                }
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
}

fn verify_exported_entry(
    entry: &super::RelationalCommitIndexEntry,
    artifact: Artifact,
    failure: impl Fn(Cause) -> RelationalStorageUnavailable,
) -> Result<Artifact, Denial> {
    if artifact.position() != entry.position
        || artifact.commit_id() != entry.commit.commit_id
        || artifact.payload_digest().as_bytes() != &entry.payload_digest
    {
        return Err(Denial::StorageUnavailable(failure(
            Cause::IntegrityMismatch,
        )));
    }
    Ok(artifact)
}

fn owned_export_denial(
    cause: crate::durability::medium::RelationalDurableOwnedReadRefusal,
    position: crate::publication::patch::data::PatchStreamPosition,
) -> Denial {
    use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
    match cause {
        Refusal::Request(cause) => Denial::Request(cause),
        Refusal::Buffer(cause) => Denial::from(cause),
        Refusal::Storage(cause) => Denial::StorageUnavailable(RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(position),
            cause,
        }),
        Refusal::CanonicalFrame(cause) => Denial::Frame(cause),
        Refusal::ForeignSource | Refusal::ForeignProtection | Refusal::Busy | Refusal::Bounds => {
            Denial::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(position),
                cause: Cause::SourceUnavailable,
            })
        }
        Refusal::UnsupportedProvider | Refusal::TransitionFrame(_) => {
            export_stop(crate::durability::checkpoint_history::capture::owned_stop(
                cause,
                RelationalStorageSubject::Envelope(position),
            ))
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
    cause: &crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactFrameRefusal,
) -> bool {
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactFrameRefusal as Frame;
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
    cause: crate::durability::canonical_commit_artifact::ControlledFrameCopyDenial,
) -> Denial {
    use crate::durability::canonical_commit_artifact::ControlledFrameCopyDenial as Copy;
    match cause {
        Copy::Frame(cause) => Denial::Frame(cause),
        Copy::Request(cause) => Denial::Request(cause),
        Copy::Allocation(cause) => Denial::Allocation(cause),
    }
}

pub(super) fn original_frame_stop(
    cause: crate::durability::canonical_commit_artifact::ControlledFrameCopyDenial,
) -> Stop {
    use crate::durability::canonical_commit_artifact::ControlledFrameCopyDenial as Copy;
    match cause {
        Copy::Request(cause) => Stop::Request(cause),
        Copy::Allocation(cause) => Stop::Allocation(cause),
        Copy::Frame(cause) => {
            crate::durability::data::RelationalNativeCheckpointCaptureDenial::ArtifactFrame(cause)
                .into()
        }
    }
}
