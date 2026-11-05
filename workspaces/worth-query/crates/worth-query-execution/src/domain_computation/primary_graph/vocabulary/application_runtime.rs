//! `application_runtime`'s vocabulary, moved whole from `primary_graph::application_runtime`: its root file's items here, one child per other source file.

/// Receipt from closing this runtime's live delivery.
///
/// Later commits no longer enter live delivery. `owner_terminal()` is true once
/// no live consumers remain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationLiveDeliveryCloseReceipt {
    remaining_live_consumers: usize,
}

impl WorthQueryApplicationLiveDeliveryCloseReceipt {
    pub const fn remaining_live_consumers(self) -> usize {
        self.remaining_live_consumers
    }

    pub const fn owner_terminal(self) -> bool {
        self.remaining_live_consumers == 0
    }
}
pub(crate) mod certification_cost;
pub(crate) mod execution_memory;
pub(crate) mod receipt;
pub(crate) mod storage_retention;
