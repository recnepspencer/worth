use std::sync::{Arc, Mutex};

/// The installed provider owns this aggregate. A ticket follows the final
/// shared evidence holder, including copies made by receipt observation.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct CompletedEvidenceCapacity {
    inner: Arc<Mutex<CapacityState>>,
}

struct CapacityState {
    maximum: usize,
    retained: usize,
}

pub(super) struct CompletedEvidenceTicket {
    owner: CompletedEvidenceCapacity,
    bytes: usize,
}

impl CompletedEvidenceCapacity {
    pub(super) fn new(maximum: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CapacityState {
                maximum,
                retained: 0,
            })),
        }
    }

    /// Whether `bytes` fit the installed capacity at all.
    pub(super) fn admits(&self, bytes: usize) -> bool {
        bytes
            <= self
                .inner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .maximum
    }

    pub(super) fn reserve(&self, bytes: usize) -> Option<CompletedEvidenceTicket> {
        let mut state = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let next = state.retained.checked_add(bytes)?;
        if next > state.maximum {
            return None;
        }
        state.retained = next;
        Some(CompletedEvidenceTicket {
            owner: self.clone(),
            bytes,
        })
    }

    #[cfg(any(test, feature = "test-query-execution-observer"))]
    pub(in crate::domain_computation::primary_graph) fn retained_bytes(&self) -> usize {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retained
    }
}

impl Drop for CompletedEvidenceTicket {
    fn drop(&mut self) {
        let mut state = self
            .owner
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.retained = state
            .retained
            .checked_sub(self.bytes)
            .expect("a completed evidence ticket refunds its own reservation");
    }
}
