//! A resident retention visit borrows without copying until it requests an edit.
use super::{PartitionState, ResidentMirrorTarget};

pub(super) trait ResidentTargetEditor {
    fn read_target(
        &self,
        target: &ResidentMirrorTarget,
    ) -> Result<&PartitionState, crate::execution::denial_cause::RelationalExecutionDenialCause>;
    fn edit_target(&mut self, target: &ResidentMirrorTarget) -> &mut PartitionState;
}

pub struct ResidentPartitionEntry<'entry> {
    pub(super) target: &'entry ResidentMirrorTarget,
    pub(super) writer: &'entry mut dyn ResidentTargetEditor,
}

impl ResidentPartitionEntry<'_> {
    pub fn read(
        &self,
    ) -> Result<&PartitionState, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.writer.read_target(self.target)
    }

    pub fn edit(&mut self) -> &mut PartitionState {
        self.writer.edit_target(self.target)
    }
}
