use std::sync::Arc;

use super::{
    remove_name_for_branch, SignalBranchExecutionCell, SignalBranchRegistry,
    SignalBranchRegistryEntry,
};

pub(in crate::branch::owner_services) struct SignalBranchRegistryCloseBatch<S> {
    _cells: Vec<Arc<SignalBranchExecutionCell<S>>>,
    cleaned_entries: usize,
}

impl<S> SignalBranchRegistryCloseBatch<S> {
    pub(in crate::branch::owner_services) fn cleaned_entries(&self) -> usize {
        self.cleaned_entries
    }

    pub(in crate::branch::owner_services) fn is_empty(&self) -> bool {
        self.cleaned_entries == 0
    }
}

impl<S> SignalBranchRegistry<S> {
    pub(in crate::branch::owner_services) fn take_close_batch(
        &self,
        maximum_batch_size: usize,
    ) -> SignalBranchRegistryCloseBatch<S> {
        let mut state = self.lock_state();
        let mut cells = Vec::with_capacity(maximum_batch_size.min(state.entries.len()));
        let mut cleaned_entries = 0;
        while cleaned_entries < maximum_batch_size {
            let Some((branch_id, entry)) = state.entries.pop_first() else {
                break;
            };
            cleaned_entries += 1;
            remove_name_for_branch(&mut state, branch_id);
            match entry {
                SignalBranchRegistryEntry::Reserved => {
                    state.reservation_count = state
                        .reservation_count
                        .checked_sub(1)
                        .expect("owner close releases every branch reservation once");
                }
                SignalBranchRegistryEntry::Live(cell)
                | SignalBranchRegistryEntry::Retiring(cell) => {
                    state.live_count = state
                        .live_count
                        .checked_sub(1)
                        .expect("owner close releases every live branch once");
                    cells.push(cell);
                }
            }
        }
        SignalBranchRegistryCloseBatch {
            _cells: cells,
            cleaned_entries,
        }
    }
}
