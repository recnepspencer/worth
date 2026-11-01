//! The runtime's last acknowledged native directory. Payloads remain in Store.
use crate::durability::data::{
    PreparedCheckpointDirectory, RelationalCheckpointDirectory, RelationalCheckpointManifest,
};
use crate::durability::medium::RelationalDurableColdSource;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub(crate) struct RelationalCheckpointManifestSelection {
    selected: Arc<Mutex<Option<Arc<RelationalCheckpointDirectory>>>>,
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
            .clone()
    }
    pub(crate) fn detached(&self) -> Self {
        Self {
            selected: Arc::new(Mutex::new(self.snapshot())),
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
    pub(crate) fn restore(&self, directory: Arc<RelationalCheckpointDirectory>) {
        *self
            .selected
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(directory);
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
    pub(crate) fn acknowledge(
        &self,
    ) -> Result<(), crate::storage::data::RelationalStorageUnavailableCause> {
        let directory = Arc::new(self.directory.clone().readmit()?);
        let mut selected = self
            .selection
            .selected
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if selected
            .as_ref()
            .is_some_and(|prior| self.regresses(&prior.manifest))
        {
            return Err(crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable);
        }
        *selected = Some(directory);
        Ok(())
    }
}
