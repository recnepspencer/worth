//! The installed runtime coordinates its index catalog and optional original cold image.
use crate::durability::derived_index_artifacts::{
    cold_checkpoint::DeferredIndexImage, FrozenIndexCheckpoint,
};
use crate::runtime::state::subsystems::history::canonical_publication_routes::RelationalCanonicalPublicationRoutes;
use crate::runtime::state::subsystems::indexing::{
    IndexDefinitionReadBinding, IndexingState, IndexingSubsystem,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use crate::storage::substrate::owned_state::RuntimeOwnedState;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

mod restore;
mod retirement;

#[derive(Clone, Debug)]
struct InstalledIndexImage {
    image: Arc<DeferredIndexImage>,
    history_routes: Arc<RelationalCanonicalPublicationRoutes>,
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeIndexingSubsystem {
    catalog: IndexingSubsystem,
    cold: RuntimeOwnedState<Option<InstalledIndexImage>>,
}

impl std::ops::Deref for RuntimeIndexingSubsystem {
    type Target = IndexingSubsystem;

    fn deref(&self) -> &Self::Target {
        &self.catalog
    }
}

impl RuntimeIndexingSubsystem {
    pub(crate) fn fork(&self) -> Self {
        let cold = self.cold.read();
        Self {
            catalog: self.catalog.fork_catalog(),
            cold: RuntimeOwnedState::new(cold.clone()),
        }
    }

    pub(crate) fn catalog_binding(&self) -> IndexDefinitionReadBinding {
        self.catalog.definition_read_binding()
    }

    pub(crate) fn definition_read_binding(&self) -> RuntimeIndexDefinitionReadBinding {
        RuntimeIndexDefinitionReadBinding {
            catalog: self.catalog_binding(),
            cold: self.cold.share(),
        }
    }

    pub(crate) fn checkpoint_inputs_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        FrozenIndexCheckpoint,
        crate::durability::data::RelationalNativeCheckpointCaptureDenial,
    > {
        let cold = self.cold.read();
        let indexes = self.catalog.checkpoint_inputs_for_request(request)?;
        Ok(FrozenIndexCheckpoint::new(
            indexes,
            cold.as_ref().map(|original| original.image.clone()),
        ))
    }

    pub(crate) fn snapshot_with_original(
        &self,
    ) -> (IndexingState, Option<Arc<DeferredIndexImage>>) {
        let cold = self.cold.read();
        (
            self.catalog.snapshot(),
            cold.as_ref().map(|original| original.image.clone()),
        )
    }
}

#[derive(Debug)]
pub(crate) struct RuntimeIndexDefinitionReadBinding {
    catalog: IndexDefinitionReadBinding,
    cold: RuntimeOwnedState<Option<InstalledIndexImage>>,
}

impl Clone for RuntimeIndexDefinitionReadBinding {
    fn clone(&self) -> Self {
        Self {
            catalog: self.catalog.clone(),
            cold: self.cold.share(),
        }
    }
}

impl std::ops::Deref for RuntimeIndexDefinitionReadBinding {
    type Target = IndexDefinitionReadBinding;

    fn deref(&self) -> &Self::Target {
        &self.catalog
    }
}

impl RuntimeIndexDefinitionReadBinding {
    pub(crate) fn demand_checkpoint_image(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        let Some(image) = ({
            let cold = self.cold.read();
            cold.as_ref()
                .filter(|_| self.catalog.has_definitions())
                .map(|original| original.image.clone())
        }) else {
            return Ok(());
        };
        let admitted = image.readmit(execution)?;
        // Take the existing generation gate first. The cold-owner lock binds
        // the exact image to this merge and to any concurrent checkpoint capture.
        let merge = self
            .catalog
            .prepare_checkpoint_merge(execution)
            .map_err(Failure::Request)?;
        let mut cold = self.cold.write();
        if cold
            .as_ref()
            .is_some_and(|pending| Arc::ptr_eq(&pending.image, &image))
        {
            merge.install(&admitted);
            *cold = None;
        }
        Ok(())
    }
}
