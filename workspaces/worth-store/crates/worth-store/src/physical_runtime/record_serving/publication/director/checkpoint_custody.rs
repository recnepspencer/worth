use super::RecordPublicationDirector;

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn commit_selected_checkpoint_custody(
        &self,
        publication: &crate::physical_runtime::durability::NamespaceDurableCheckpointPublication,
    ) -> Result<(), crate::physical_runtime::durability::CheckpointCustodyDenial> {
        self.root_owner
            .commit_selected_checkpoint_custody(publication)
    }
}
