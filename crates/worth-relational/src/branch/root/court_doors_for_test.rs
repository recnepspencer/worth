//! Replace one pinned storage image without changing its installed contract or identity.
use super::*;
use crate::identity::data::EntityId;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
impl RelationalBranchRoot {
    pub(crate) fn corrupt_field_image_for_test(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        locator: &AspectFieldLocator,
        replacement: Option<AspectValue>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Option<Arc<Self>> {
        let region = self.regions.get(entity.partition_id)?;
        let mut partition = region.resident_partition().ok()?.clone();
        if !partition.corrupt_entity_field_for_test(entity, locator, replacement) {
            return None;
        }
        let replacement = RelationalRootRegion::new(
            lane,
            execution,
            region.creation_root_id,
            region.id,
            partition,
            symbols,
        )
        .ok()?;
        let mut regions = self.regions.materialize();
        regions.insert(entity.partition_id, Arc::new(replacement));
        let mut image = self.clone();
        image.regions = RelationalPersistentRegionSet::initial(
            lane,
            self.id,
            regions,
            &RelationalBranchRootIdentityIssuer::default(),
        )
        .ok()?;
        Some(Arc::new(image))
    }
}
