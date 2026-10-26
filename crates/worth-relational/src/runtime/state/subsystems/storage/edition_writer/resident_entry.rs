//! A resident retention visit borrows without copying until it requests an edit.
use super::{PartitionState, ResidentMirrorTarget};

pub(super) trait ResidentTargetEditor {
    fn read_target(&self, target: &ResidentMirrorTarget) -> &PartitionState;
    fn edit_target(&mut self, target: &ResidentMirrorTarget) -> &mut PartitionState;
}

pub(crate) struct ResidentPartitionEntry<'entry> {
    pub(super) target: &'entry ResidentMirrorTarget,
    pub(super) writer: &'entry mut dyn ResidentTargetEditor,
}

impl ResidentPartitionEntry<'_> {
    pub(crate) fn read(&self) -> &PartitionState {
        self.writer.read_target(self.target)
    }

    pub(crate) fn edit(&mut self) -> &mut PartitionState {
        self.writer.edit_target(self.target)
    }
}
