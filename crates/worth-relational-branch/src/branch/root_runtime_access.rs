use super::root::RelationalBranchRootState;

impl RelationalBranchRootState {
    pub fn version_id(&self) -> crate::identity::data::VersionId {
        self.root
            .axes()
            .map(|axes| crate::identity::data::VersionId(axes.storage_version))
            .unwrap_or(crate::identity::data::VersionId(0))
    }

    pub fn entity_slot_count(&self) -> usize {
        self.root.entity_slot_count()
    }

    pub fn relation_slot_count(&self) -> usize {
        self.root.relation_slot_count()
    }
}
