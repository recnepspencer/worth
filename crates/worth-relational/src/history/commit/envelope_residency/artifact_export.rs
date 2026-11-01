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
    /// Capture metadata is read from the original Memory frame, never re-encoded.
    pub(crate) fn original_memory_payload_digest(
        &self,
    ) -> Result<[u8; 32], crate::durability::data::DurabilityError> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let fail = || {
            crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                "original Memory artifact unavailable or damaged",
            )
        };
        let EnvelopeBacking::Memory {
            original: Some(bytes),
            ..
        } = &state.backing
        else {
            return Err(fail());
        };
        let artifact = Artifact::from_owned_bytes(bytes.clone()).map_err(|_| fail())?;
        Ok(*artifact.payload_digest().as_bytes())
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
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let failure = |cause| {
            crate::durability::data::RelationalRecoveryStop::from(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(state.position.as_ref().map_or(
                    crate::publication::patch::data::PatchStreamPosition(0),
                    |position| position.position(),
                )),
                cause,
            })
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
                    .map_err(crate::durability::checkpoint_history::capture::owned_stop)?
            }
        };
        Artifact::from_owned_bytes(bytes).map_err(|_| failure(Cause::Damaged))
    }

    pub(crate) fn export_artifact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Artifact, Denial> {
        let policy = execution.allocation_policy();
        policy.check_live()?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(failure) = state.damage {
            return Err(Denial::StorageUnavailable(failure));
        }
        if let EnvelopeBacking::Memory {
            original: Some(bytes),
            ..
        } = &state.backing
        {
            // Verify the original immutable frame; never encode another copy.
            return Artifact::from_owned_bytes(bytes.clone()).map_err(|_| {
                Denial::PayloadEncoding {
                    detail: "retained original Memory artifact failed verification".into(),
                }
            });
        }
        if let EnvelopeBacking::SelectedCheckpoint { entry, directory } = &state.backing {
            return Self::selected_artifact(entry, directory, execution, 1 << 30)
                .map_err(Denial::Readmission);
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
