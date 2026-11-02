//! Selected directory DATA and the independently completed catalog frontier.
mod source_retirement;
use crate::durability::data::{
    NativeCheckpointDirectoryEdge, PreparedCheckpointDirectory, RelationalCheckpointDirectory,
    RelationalCheckpointManifest,
};
use crate::durability::medium::RelationalDurableColdSource;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub(crate) struct RelationalCheckpointManifestSelection {
    selected: Arc<Mutex<CheckpointSelectionState>>,
}
#[derive(Debug, Clone)]
struct CheckpointSelectionState {
    directory: Option<NativeCheckpointDirectoryEdge>,
    acknowledged_index_definition_frontier: u64,
}
impl Default for CheckpointSelectionState {
    fn default() -> Self {
        Self {
            directory: None,
            acknowledged_index_definition_frontier: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RelationalCheckpointManifestPublication {
    selection: RelationalCheckpointManifestSelection,
    directory: PreparedCheckpointDirectory,
}

impl RelationalCheckpointManifestSelection {
    pub(crate) fn snapshot(&self) -> Option<Arc<RelationalCheckpointDirectory>> {
        self.selected
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .directory
            .as_ref()
            .and_then(NativeCheckpointDirectoryEdge::snapshot)
    }
    pub(crate) fn acknowledged_index_definition_frontier(&self) -> u64 {
        self.selected
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .acknowledged_index_definition_frontier
    }
    pub(crate) fn detached(&self) -> Self {
        // Pointer and completed semantic frontier come from ONE owner observation.
        let state = self
            .selected
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        Self {
            selected: Arc::new(Mutex::new(state)),
        }
    }
    pub(crate) fn prepare(
        &self,
        manifest: RelationalCheckpointManifest,
        source: Arc<dyn RelationalDurableColdSource>,
        history_members: Arc<crate::durability::data::RelationalHistoryMemberDirectory>,
    ) -> RelationalCheckpointManifestPublication {
        RelationalCheckpointManifestPublication {
            selection: self.clone(),
            directory: PreparedCheckpointDirectory {
                manifest,
                source,
                history_members,
            },
        }
    }
    pub(crate) fn restore_data(&self, directory: Arc<RelationalCheckpointDirectory>) {
        let prior = {
            let mut state = self
                .selected
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            state
                .directory
                .replace(NativeCheckpointDirectoryEdge::new(directory))
        };
        drop(prior);
    }
    pub(crate) fn restore_selected(&self, directory: Arc<RelationalCheckpointDirectory>) {
        let prior = {
            let mut state = self
                .selected
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.acknowledged_index_definition_frontier =
                directory.manifest.index_definition_frontier;
            state
                .directory
                .replace(NativeCheckpointDirectoryEdge::new(directory))
        };
        drop(prior);
    }
}

impl RelationalCheckpointManifestPublication {
    pub(crate) fn validate_coverage(
        &self,
    ) -> Result<(), crate::durability::medium::RelationalCheckpointAcknowledgementDenial> {
        let selected = self
            .selection
            .selected
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if selected
            .directory
            .as_ref()
            .is_some_and(|prior| self.regresses(&prior.manifest))
        {
            return Err(
                crate::durability::medium::RelationalCheckpointAcknowledgementDenial::StaleCapture,
            );
        }
        Ok(())
    }
    fn regresses(&self, prior: &RelationalCheckpointManifest) -> bool {
        prior.covered_position > self.directory.manifest.covered_position
            || prior.coverage.up_to_branch_transition
                > self.directory.manifest.coverage.up_to_branch_transition
            || prior.index_definition_frontier > self.directory.manifest.index_definition_frontier
    }
    pub(crate) fn source_matches(&self, source: &Arc<dyn RelationalDurableColdSource>) -> bool {
        Arc::ptr_eq(&self.directory.source, source)
    }
    pub(crate) fn begin_selected_directory_admission(
        &self,
    ) -> crate::durability::data::CheckpointDirectoryAdmission {
        crate::durability::data::CheckpointDirectoryAdmission::new(self.directory.clone())
    }
    pub(crate) fn complete_acknowledgement(
        &self,
    ) -> Result<(), crate::durability::medium::RelationalCheckpointAcknowledgementDenial> {
        use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Denial;
        let mut selected = match self.selection.selected.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
        };
        // THIS operation's retired, original selection is the completed fact.
        // The data pointer may already belong to another incomplete capture.
        selected.acknowledged_index_definition_frontier = selected
            .acknowledged_index_definition_frontier
            .max(self.directory.manifest.index_definition_frontier);
        Ok(())
    }
    pub(crate) fn acknowledge_directory(
        &self,
        directory: Arc<RelationalCheckpointDirectory>,
    ) -> Result<
        Option<Arc<RelationalCheckpointDirectory>>,
        crate::durability::medium::RelationalCheckpointAcknowledgementDenial,
    > {
        use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Denial;
        let mut selected = match self.selection.selected.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::Busy),
        };
        if selected
            .directory
            .as_ref()
            .is_some_and(|prior| self.regresses(&prior.manifest))
        {
            return Err(Denial::StaleCapture);
        }
        Ok(selected
            .directory
            .replace(NativeCheckpointDirectoryEdge::new(directory))
            .map(NativeCheckpointDirectoryEdge::into_original))
    }
}
