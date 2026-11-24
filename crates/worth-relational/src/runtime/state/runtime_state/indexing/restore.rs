//! Initial installation and authenticated image admission share one catalog cut.
use super::{InstalledIndexImage, RuntimeIndexingSubsystem};
use crate::durability::checkpoints::region_directory::ReadmittedSchemas;
use crate::durability::data::RelationalCheckpointDirectory;
use crate::durability::derived_index_artifacts::cold_checkpoint::{
    storage_subject, PreparedIndexImage,
};
use crate::indexes::authority::CheckpointIndexProjections;
use crate::runtime::state::subsystems::indexing::IndexingState;
use crate::storage::data::{RelationalStorageUnavailable, RelationalStorageUnavailableCause};
use crate::storage::substrate::maintenance::ReconstructionLane;
use std::sync::Arc;

impl RuntimeIndexingSubsystem {
    pub(crate) fn install_cold_checkpoint(
        &self,
        lane: &ReconstructionLane<'_>,
        directory: Arc<RelationalCheckpointDirectory>,
        runtime: &super::super::RelationalRuntime,
        schemas: ReadmittedSchemas,
    ) -> Result<(), RelationalStorageUnavailable> {
        let subject = storage_subject(&directory);
        let prepared = PreparedIndexImage::prepare(directory)?;
        let history = runtime
            .history
            .detached_publication_snapshot()
            .map_err(|_| RelationalStorageUnavailable {
                subject,
                cause: RelationalStorageUnavailableCause::Damaged,
            })?;
        let projections =
            CheckpointIndexProjections::from_context(runtime.checkpoint_projection_context(lane));
        let (history, history_routes) = history.into_parts();
        let (state, image) = prepared.admit(history, projections, schemas);
        self.replace(
            state,
            Some(InstalledIndexImage {
                image,
                history_routes,
            }),
        );
        Ok(())
    }

    pub(crate) fn install(&self, state: IndexingState) {
        self.replace(state, None);
    }

    fn replace(&self, state: IndexingState, image: Option<InstalledIndexImage>) {
        let install = self.catalog.prepare_checkpoint_replacement();
        let mut cold = self.cold.write();
        install.install(state);
        let previous = std::mem::replace(&mut *cold, image);
        drop(cold);
        drop(install);
        drop(previous);
    }
}
