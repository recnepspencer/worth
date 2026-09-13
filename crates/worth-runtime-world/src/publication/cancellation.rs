use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use worth_signal::facade::branch::{SignalOwnerCancellationSource, SignalOwnerCancellationToken};

/// Runtime World cancellation authority for one publication or one branch
/// creation. It owns the shared World flag and one private Signal owner
/// source; `cancel()` changes both, so a caller can never supply a raw Signal
/// token in place of the Runtime World token.
#[derive(Debug, Default)]
pub struct RuntimeWorldCancellationSource {
    requested: Arc<AtomicBool>,
    signal: SignalOwnerCancellationSource,
}

/// Read-only cancellation token borrowed by a single owner transition.
#[derive(Debug, Clone)]
pub struct RuntimeWorldCancellationToken {
    requested: Arc<AtomicBool>,
    signal: SignalOwnerCancellationToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeWorldCancellationBoundary {
    BeforeReservation,
    BeforeFirstOwnerEffect,
    BetweenOwnerEffects,
    BeforeProductMovement,
}

impl RuntimeWorldCancellationSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token(&self) -> RuntimeWorldCancellationToken {
        RuntimeWorldCancellationToken {
            requested: Arc::clone(&self.requested),
            signal: self.signal.token(),
        }
    }

    /// Cancels Runtime World phases and any in-flight synchronous Signal owner
    /// call issued from this source.
    pub fn cancel(&self) {
        self.requested.store(true, Ordering::Release);
        self.signal.cancel();
    }
}

impl RuntimeWorldCancellationToken {
    pub fn is_cancelled(&self) -> bool {
        self.requested.load(Ordering::Acquire)
    }

    /// Crate-internal handoff for a Signal fork, advance, or retirement call.
    pub(crate) fn signal_token(&self) -> &SignalOwnerCancellationToken {
        &self.signal
    }

    pub(crate) fn check(&self, _boundary: RuntimeWorldCancellationBoundary) -> Result<(), ()> {
        if self.is_cancelled() {
            Err(())
        } else {
            Ok(())
        }
    }
}
