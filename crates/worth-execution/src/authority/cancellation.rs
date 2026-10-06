use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// A request-owned cancellation signal: the one handle that can cancel. Its
/// owner hands leases and serial runs only [`CancellationToken`]s, so nothing
/// that runs under the request can cancel it, and whatever the owner does on
/// cancel (waking its waiters) happens on every cancel.
#[derive(Debug, Clone, Default)]
pub struct CancellationSource {
    cancelled: Arc<AtomicBool>,
}

/// An observe-only view of a [`CancellationSource`]. Leases and serial runs
/// carry it; child leases also observe their parent. A token with no source
/// is never cancelled, and no token can cancel:
///
/// ```compile_fail
/// let source = worth_execution::CancellationSource::new();
/// source.token().cancel();
/// ```
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token(&self) -> CancellationToken {
        CancellationToken {
            cancelled: Arc::clone(&self.cancelled),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

impl CancellationToken {
    /// A token no source can cancel.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}
