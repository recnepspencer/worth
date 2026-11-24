//! Transfer placement in the same cell; selected members never fall back to journals.
use super::*;
use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Ack;
use crate::storage::data::directory_source_edge::{DirectorySourceEdge, RetirableDirectory};

/// A selected cell owns the same directory allocation it originally admitted.
pub(crate) trait SelectedCanonicalDirectory: RetirableDirectory + std::any::Any {
    fn source(&self) -> &Arc<dyn RelationalDurableColdSource>;
    fn require_entry(&self, entry: &RelationalCommitIndexEntry) -> Result<(), Ack>;
    fn readmit_canonical_member(
        &self,
        entry: &RelationalCommitIndexEntry,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop>;
    fn preflight_retirement(&self, source: &dyn RelationalDurableColdSource) -> Result<(), Ack>;
}
pub(crate) type SelectedCheckpointEdge = DirectorySourceEdge<dyn SelectedCanonicalDirectory>;

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
    pub(crate) fn retire_journal_source(
        &mut self,
        source: Option<&dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<(), crate::durability::medium::RelationalCheckpointAcknowledgementDenial> {
        match &mut self.backing {
            Some(EnvelopeBacking::Journal {
                source: original, ..
            }) => original
                .retire(source.ok_or(Ack::CaptureNotBound)?)
                .map_err(Ack::Retirement),
            Some(EnvelopeBacking::Memory { .. } | EnvelopeBacking::SelectedCheckpoint { .. })
            | None => Ok(()),
        }
    }

    pub(crate) fn has_directory(&self) -> bool {
        matches!(
            self.backing,
            Some(EnvelopeBacking::SelectedCheckpoint { .. })
        )
    }
    pub(crate) fn take_directory(&mut self) -> Option<Arc<dyn SelectedCanonicalDirectory>> {
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
    pub(crate) fn selected_directory(&self) -> Option<Arc<dyn SelectedCanonicalDirectory>> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match &state.backing {
            EnvelopeBacking::SelectedCheckpoint { directory, .. } => directory.snapshot(),
            _ => None,
        }
    }

    pub(crate) fn install_selected(
        &self,
        entry: super::super::RetainedCanonicalIndex,
        directory: Arc<dyn SelectedCanonicalDirectory>,
    ) -> Result<
        RetiredCanonicalPlacement,
        crate::durability::medium::RelationalCheckpointAcknowledgementDenial,
    > {
        directory.require_entry(entry.as_ref())?;
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Ack::Busy),
        };
        if let Some(position) = &state.position
            && position.position() != entry.position
        {
            return Err(Ack::Storage(Cause::IntegrityMismatch));
        }
        match &state.backing {
            EnvelopeBacking::Journal {
                entry: actual,
                source,
                ..
            } => {
                if !actual.same_owner(&entry) || !Arc::ptr_eq(source, directory.source()) {
                    return Err(Ack::Storage(Cause::IntegrityMismatch));
                }
            }
            EnvelopeBacking::SelectedCheckpoint {
                entry: actual,
                directory: old,
            } => {
                if !actual.same_owner(&entry) || !Arc::ptr_eq(old.source(), directory.source()) {
                    return Err(Ack::Storage(Cause::IntegrityMismatch));
                }
            }
            EnvelopeBacking::Memory { canonical, .. } => {
                if canonical.envelope().commit != entry.commit {
                    return Err(Ack::Storage(Cause::IntegrityMismatch));
                }
            }
        }
        let old = std::mem::replace(
            &mut state.backing,
            EnvelopeBacking::SelectedCheckpoint {
                entry,
                directory: SelectedCheckpointEdge::new(directory),
            },
        );
        state.canonical_payload_bytes = None;
        Ok(RetiredCanonicalPlacement { backing: Some(old) })
    }
    pub(crate) fn readmit_selected(
        entry: &RelationalCommitIndexEntry,
        directory: &dyn SelectedCanonicalDirectory,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        directory.readmit_canonical_member(entry, execution, limit)
    }
}
