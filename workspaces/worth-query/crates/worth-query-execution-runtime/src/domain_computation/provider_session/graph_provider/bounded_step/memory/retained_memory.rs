use std::sync::Arc;

use super::arena::WorthQueryGraphProviderMemoryState;

/// Move-only memory admitted against one managed provider execution.
///
/// The framework constructs this value only after reserving its actual vector
/// capacity against the installed retained-memory ceiling. Dropping the value
/// releases that reservation.
pub struct WorthQueryGraphProviderRetainedMemory {
    allocation: Arc<WorthQueryGraphProviderRetainedAllocation>,
}

struct WorthQueryGraphProviderRetainedAllocation {
    state: Arc<WorthQueryGraphProviderMemoryState>,
    bytes: Vec<u8>,
    charged_bytes: u64,
}

impl WorthQueryGraphProviderRetainedMemory {
    pub(super) fn new(
        state: Arc<WorthQueryGraphProviderMemoryState>,
        bytes: Vec<u8>,
        charged_bytes: u64,
    ) -> Self {
        Self {
            allocation: Arc::new(WorthQueryGraphProviderRetainedAllocation {
                state,
                bytes,
                charged_bytes,
            }),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.allocation.bytes
    }

    pub fn bytes_mut(&mut self) -> Option<&mut [u8]> {
        Arc::get_mut(&mut self.allocation).map(|allocation| allocation.bytes.as_mut_slice())
    }

    pub fn len(&self) -> usize {
        self.allocation.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.allocation.bytes.is_empty()
    }

    pub fn arena_identity(&self) -> u64 {
        self.allocation.state.identity()
    }

    pub fn provisional_restore_alias(&self) -> Self {
        Self {
            allocation: Arc::clone(&self.allocation),
        }
    }
}

impl std::fmt::Debug for WorthQueryGraphProviderRetainedMemory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryGraphProviderRetainedMemory")
            .field("byte_count", &self.allocation.bytes.len())
            .finish_non_exhaustive()
    }
}

impl Drop for WorthQueryGraphProviderRetainedAllocation {
    fn drop(&mut self) {
        self.state.release(self.charged_bytes);
    }
}
