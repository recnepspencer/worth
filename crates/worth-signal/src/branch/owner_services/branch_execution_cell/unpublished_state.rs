//! Only a unique, unpublished cell can lend its state without a runtime lock.
use super::SignalBranchExecutionCell;
impl<S> SignalBranchExecutionCell<S> {
    pub(in crate::branch::owner_services) fn unpublished_state_mut(&mut self) -> &mut S {
        self.state
            .get_mut()
            .expect("an unpublished cell has never executed caller code")
    }
}
