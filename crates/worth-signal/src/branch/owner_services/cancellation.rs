use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Exact no-movement result of observing cancellation before linearization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SignalOwnerCancellationRequested;

/// Proof that one synchronous operation crossed its cancellation cutoff.
///
/// Cancellation requested after this permit is issued is descriptive only;
/// an owner-issued movement or performed outcome wins.
pub(crate) struct SignalOwnerMovementPermit<'a> {
    _token: &'a SignalOwnerCancellationToken,
}

/// Caller-owned source for synchronous Signal owner-operation cancellation.
#[derive(Debug, Clone, Default)]
pub struct SignalOwnerCancellationSource {
    requested: Arc<AtomicBool>,
}

/// Read-only cancellation capability borrowed by one synchronous port call.
#[derive(Debug, Clone)]
pub struct SignalOwnerCancellationToken {
    requested: Arc<AtomicBool>,
}

impl SignalOwnerCancellationSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token(&self) -> SignalOwnerCancellationToken {
        SignalOwnerCancellationToken {
            requested: Arc::clone(&self.requested),
        }
    }

    pub fn cancel(&self) {
        self.requested.store(true, Ordering::Release);
    }
}

impl SignalOwnerCancellationToken {
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_requested()
    }

    pub(crate) fn preflight_cell_wait(&self) -> Result<(), SignalOwnerCancellationRequested> {
        if self.cancellation_requested() {
            return Err(SignalOwnerCancellationRequested);
        }
        Ok(())
    }

    pub(crate) fn preflight_movement(
        &self,
    ) -> Result<SignalOwnerMovementPermit<'_>, SignalOwnerCancellationRequested> {
        if self.cancellation_requested() {
            return Err(SignalOwnerCancellationRequested);
        }
        Ok(SignalOwnerMovementPermit { _token: self })
    }

    fn cancellation_requested(&self) -> bool {
        self.requested.load(Ordering::Acquire)
    }
}
