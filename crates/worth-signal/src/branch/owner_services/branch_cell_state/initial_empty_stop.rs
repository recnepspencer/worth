//! Initial sealed state comparison without producing a new observation.
use super::SignalBranchCellState;
use crate::branch::SignalBranchObservation;
impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn matches_initial_empty_stop(
        &self,
        witness: &SignalBranchObservation,
    ) -> bool {
        let Some(target) = witness.target().as_basis() else {
            return false;
        };
        self.head_generation == 0
            && witness.generation().get() == 0
            && self.handle.head_snapshot_id.is_none()
            && self.restore_snapshot_id.is_none()
            && target.snapshot_id().is_none()
            && target.restore_snapshot_id().is_none()
            && self.definition_basis == target.definition_basis()
            && self.state.has_initial_empty_sealed_origin()
    }
}
