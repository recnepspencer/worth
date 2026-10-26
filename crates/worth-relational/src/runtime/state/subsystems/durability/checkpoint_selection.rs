//! The runtime's last acknowledged native directory. Payloads remain in Store.
use crate::durability::data::{RelationalCheckpointDirectory, RelationalCheckpointManifest};
use crate::durability::medium::RelationalDurableColdSource;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub(crate) struct RelationalCheckpointManifestSelection {
    selected: Arc<Mutex<Option<Arc<RelationalCheckpointDirectory>>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct RelationalCheckpointManifestPublication {
    selection: RelationalCheckpointManifestSelection,
    directory: Arc<RelationalCheckpointDirectory>,
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
    ) -> RelationalCheckpointManifestPublication {
        RelationalCheckpointManifestPublication {
            selection: self.clone(),
            directory: Arc::new(RelationalCheckpointDirectory { manifest, source }),
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
    pub(crate) fn acknowledge(&self) {
        let mut selected = self
            .selection
            .selected
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if selected.as_ref().is_some_and(|prior| {
            prior.manifest.covered_position > self.directory.manifest.covered_position
                || prior.manifest.coverage.up_to_branch_transition
                    > self.directory.manifest.coverage.up_to_branch_transition
        }) {
            return;
        }
        *selected = Some(self.directory.clone());
    }
}
