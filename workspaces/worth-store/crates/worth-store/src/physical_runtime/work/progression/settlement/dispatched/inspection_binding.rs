use super::{DispatchedPhysicalWork, PhysicalWorkOperationFamily};

impl DispatchedPhysicalWork {
    pub(in crate::physical_runtime) fn bind_inspection_source(
        &mut self,
        source: worth_store_physical_backend::ArtifactTreeFile,
    ) {
        assert!(
            self.inspection_source.is_none() && self.intent().scope().inspection_target().is_some(),
            "one admitted inspection source"
        );
        self.inspection_source = Some(source);
    }

    pub(in crate::physical_runtime) fn matches_inspection(
        &self,
        physical: &worth_store_physical_backend::ObservedArtifactInspectionRead,
    ) -> bool {
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && self.inspection_source.as_ref() == Some(physical.artifact())
            && self.intent().scope().inspection_target() == Some(physical.range())
            && physical.completed_bytes() <= physical.range().length() as u64
            && self.intent().operation() == PhysicalWorkOperationFamily::ArtifactRangeRead
    }
}
