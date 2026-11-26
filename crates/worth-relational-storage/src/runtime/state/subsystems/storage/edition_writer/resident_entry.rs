//! A resident retention visit borrows without copying until it requests an edit.
use super::PartitionState;
use crate::identity::data::PartitionId;

pub(super) trait ResidentTargetEditor {
    fn read_target(
        &self,
        id: PartitionId,
    ) -> Result<&PartitionState, crate::execution::denial_cause::RelationalExecutionDenialCause>;
    fn edit_target(&mut self, id: PartitionId) -> &mut PartitionState;
}

pub struct ResidentPartitionEntry<'entry> {
    pub(super) id: PartitionId,
    pub(super) writer: &'entry mut dyn ResidentTargetEditor,
}

impl ResidentPartitionEntry<'_> {
    pub fn read(
        &self,
    ) -> Result<&PartitionState, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.writer.read_target(self.id)
    }

    pub fn edit(&mut self) -> &mut PartitionState {
        self.writer.edit_target(self.id)
    }
}
