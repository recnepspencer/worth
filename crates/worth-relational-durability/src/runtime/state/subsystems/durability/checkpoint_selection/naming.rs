//! Naming-specific detached custody pinning, acknowledgement consolidation,
//! and retention-floor monotonicity checks for checkpoint selection.

use std::any::Any;
use std::sync::Arc;

use super::{RelationalCheckpointManifestPublication, RelationalCheckpointManifestSelection};
use crate::durability::authority::runtime_rebuild::checkpoint_restore::cold_naming::SelectedNamingSource;
use crate::durability::data::{
    NativeCheckpointDirectoryEdge, RelationalCheckpointDirectory, RelationalCheckpointManifest,
};
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::cold_naming::LineageNamingAckGuard;

impl RelationalCheckpointManifestSelection {
    /// Pins the exact `RelationalCheckpointDirectory` backing a detached
    /// runtime fork's lineage routes whenever a concurrent ACK on the parent
    /// runtime has already advanced `state.directory` past it.
    pub fn pin_detached_naming_directory(&self, custody: Arc<dyn Any + Send + Sync>) {
        let Ok(directory) = custody.downcast::<RelationalCheckpointDirectory>() else {
            return;
        };
        let mut state = self.lock_state();
        let already_selected = state
            .directory
            .as_ref()
            .and_then(NativeCheckpointDirectoryEdge::snapshot)
            .is_some_and(|selected| Arc::ptr_eq(&selected, &directory));
        if !already_selected {
            state.naming_directory = Some(NativeCheckpointDirectoryEdge::new(directory));
        }
    }
}

impl RelationalCheckpointManifestPublication {
    pub(super) fn consolidate_cold_naming_locked(
        &self,
        selected: &mut super::CheckpointSelectionState,
        ack_guard: Option<LineageNamingAckGuard<'_>>,
        directory: &Arc<RelationalCheckpointDirectory>,
    ) -> Option<NativeCheckpointDirectoryEdge> {
        if let Some(guard) = ack_guard {
            let source = Arc::new(SelectedNamingSource::new(
                directory,
                selected.working_set.clone(),
            ));
            guard.consolidate_acknowledged_naming(
                source,
                &directory.manifest.naming,
                &self.naming_generations,
            );
        }
        selected.naming_directory.take()
    }

    pub(super) fn regresses_naming_retention_floor(
        &self,
        prior: &RelationalCheckpointManifest,
    ) -> bool {
        prior.naming.branches.iter().any(|prior_root| {
            let Some(prior_floor) = prior_root.index.and_then(|h| h.retention_floor) else {
                return false;
            };
            let next_floor = self
                .directory
                .manifest
                .naming
                .root(&prior_root.branch)
                .and_then(|r| r.index)
                .and_then(|h| h.retention_floor);
            next_floor.is_none_or(|nf| nf < prior_floor)
        })
    }
}
