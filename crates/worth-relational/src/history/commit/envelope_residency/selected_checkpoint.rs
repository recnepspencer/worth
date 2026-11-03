//! Transfer placement in the same cell; selected members never fall back to journals.
use super::*;
use crate::durability::data::RelationalCheckpointDirectory;
pub(crate) struct RetiredCanonicalPlacement {
    backing: Option<EnvelopeBacking>,
}
impl std::fmt::Debug for RetiredCanonicalPlacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let placement = match &self.backing {
            Some(EnvelopeBacking::Memory { .. }) => "Memory",
            Some(EnvelopeBacking::Journal { .. }) => "Journal",
            Some(EnvelopeBacking::SelectedCheckpoint { .. }) => "SelectedCheckpoint",
            None => "Released",
        };
        f.debug_tuple("RetiredCanonicalPlacement")
            .field(&placement)
            .finish()
    }
}
impl RetiredCanonicalPlacement {
    pub(crate) fn has_directory(&self) -> bool {
        matches!(
            self.backing,
            Some(EnvelopeBacking::SelectedCheckpoint { .. })
        )
    }
    pub(crate) fn take_directory(&mut self) -> Option<Arc<RelationalCheckpointDirectory>> {
        if !self.has_directory() {
            return None;
        }
        let Some(EnvelopeBacking::SelectedCheckpoint { entry, directory }) = self.backing.take()
        else {
            unreachable!("checked displaced directory variant")
        };
        // The same captured row and new cell still retain this exact index owner.
        drop(entry);
        Some(directory.into_original())
    }
}
impl RelationalCommitEnvelopeResidency {
    pub(crate) fn selected_directory(&self) -> Option<Arc<RelationalCheckpointDirectory>> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match &state.backing {
            EnvelopeBacking::SelectedCheckpoint { directory, .. } => directory.snapshot(),
            _ => None,
        }
    }

    pub(crate) fn install_selected(
        &self,
        entry: super::super::RetainedCanonicalIndex,
        directory: Arc<RelationalCheckpointDirectory>,
    ) -> Result<
        RetiredCanonicalPlacement,
        crate::durability::medium::RelationalCheckpointAcknowledgementDenial,
    > {
        use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Ack;
        let row = directory
            .manifest
            .canonical_members
            .member(entry.position)
            .ok_or(Ack::Storage(Cause::MissingMember))?;
        let ordinal = usize::try_from(
            entry
                .position
                .0
                .checked_sub(1)
                .ok_or(Ack::Storage(Cause::IntegrityMismatch))?,
        )
        .map_err(|_| Ack::Storage(Cause::IntegrityMismatch))?;
        if directory.manifest.commits.get(ordinal) != Some(entry.as_ref()) || row.length < 66 {
            return Err(Ack::Storage(Cause::IntegrityMismatch));
        }
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Ack::Busy),
        };
        if let Some(position) = &state.position {
            if position.position() != entry.position {
                return Err(Ack::Storage(Cause::IntegrityMismatch));
            }
        }
        match &state.backing {
            EnvelopeBacking::Journal {
                entry: actual,
                source,
                ..
            } => {
                if !actual.same_owner(&entry) || !Arc::ptr_eq(source, &directory.source) {
                    return Err(Ack::Storage(Cause::IntegrityMismatch));
                }
            }
            EnvelopeBacking::SelectedCheckpoint {
                entry: actual,
                directory: old,
            } => {
                if !actual.same_owner(&entry) || !Arc::ptr_eq(&old.source, &directory.source) {
                    return Err(Ack::Storage(Cause::IntegrityMismatch));
                }
            }
            EnvelopeBacking::Memory { envelope, .. } => {
                if envelope.commit != entry.commit {
                    return Err(Ack::Storage(Cause::IntegrityMismatch));
                }
            }
        }
        let old = std::mem::replace(
            &mut state.backing,
            EnvelopeBacking::SelectedCheckpoint {
                entry,
                directory: crate::durability::data::NativeCheckpointDirectoryEdge::new(directory),
            },
        );
        state.canonical_payload_bytes = None;
        Ok(RetiredCanonicalPlacement { backing: Some(old) })
    }
    pub(crate) fn readmit_selected(
        entry: &RelationalCommitIndexEntry,
        directory: &RelationalCheckpointDirectory,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        let artifact = Self::selected_artifact(entry, directory, execution, limit)?;
        Self::readmit_owned_artifact(entry, artifact, execution)
    }
    pub(super) fn selected_artifact(
        entry: &RelationalCommitIndexEntry,
        directory: &RelationalCheckpointDirectory,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalCanonicalCommitArtifact, RelationalRecoveryStop> {
        let member = directory
            .manifest
            .canonical_members
            .member(entry.position)
            .ok_or(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(entry.position),
                cause: Cause::MissingMember,
            })?;
        let bytes = directory
            .canonical_read
            .read_member(&directory.source, execution, member, limit)
            .map_err(|cause| {
                crate::durability::checkpoint_history::capture::owned_stop(
                    cause,
                    RelationalStorageSubject::Envelope(entry.position),
                )
            })?;
        // Only this consumer read a canonical body; generic selected reads
        // also carry derived-index members and cannot certify this event.
        directory.source.observe_canonical_member_read(member);
        if <[u8; 32]>::from(<sha2::Sha256 as sha2::Digest>::digest(bytes.bytes())) != member.sha256
            || bytes.len() as u64 != member.length
        {
            return Err(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(entry.position),
                cause: Cause::IntegrityMismatch,
            }
            .into());
        }
        let artifact =
            RelationalCanonicalCommitArtifact::from_owned_bytes(bytes).map_err(|_| {
                RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Envelope(entry.position),
                    cause: Cause::Damaged,
                }
            })?;
        Self::check_frame(entry, &artifact)?;
        Ok(artifact)
    }
}
