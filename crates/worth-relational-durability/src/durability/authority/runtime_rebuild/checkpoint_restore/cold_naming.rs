//! Immutable per-checkpoint cold naming source for the lineage subsystem.
//!
//! Each `SelectedNamingSource` is bound permanently to the exact
//! `RelationalCheckpointDirectory` whose branch roots match the routes pointing
//! at it (held weakly so close and ACK displacement never leak directory
//! custody). Acknowledging a newer checkpoint never mutates what existing
//! routes read: displaced directories stay strongly alive in `retired` until
//! `consolidate_acknowledged_naming` atomically updates both source and branch
//! routes together under the lineage write lock.
use std::sync::{Arc, Weak};

use worth_relational_lineage::internal_port::lineage::tiered::{
    ColdNamingSource, PinnedNamingStore,
};
use worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant;

use crate::durability::data::RelationalCheckpointDirectory;

#[derive(Debug)]
pub struct SelectedNamingSource {
    directory: Weak<RelationalCheckpointDirectory>,
    working_set: Option<Arc<WorkingSetAccountant>>,
}

impl SelectedNamingSource {
    pub fn new(
        directory: &Arc<RelationalCheckpointDirectory>,
        working_set: Option<Arc<WorkingSetAccountant>>,
    ) -> Self {
        Self {
            directory: Arc::downgrade(directory),
            working_set,
        }
    }
}

impl ColdNamingSource for SelectedNamingSource {
    fn store(&self) -> Option<PinnedNamingStore> {
        let directory = self.directory.upgrade()?;
        let store = directory.naming_state().clone();
        if let Some(accountant) = &self.working_set {
            store.bind_working_set(accountant.clone());
        }
        Some(PinnedNamingStore::new(store, directory))
    }
}
