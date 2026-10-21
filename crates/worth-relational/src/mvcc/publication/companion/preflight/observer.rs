//! A prepared companion cutover's scheduling outcome.
//!
//! This is a hint for callers that queued work before publication. The
//! selected native image remains the authority for currentness.

use std::fmt;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

const PREPARED: u8 = 0;
const INSTALLED: u8 = 1;
const ABORTED: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompanionPublicationCompletion {
    Prepared,
    Installed,
    Aborted,
}

struct CompletionCore {
    state: AtomicU8,
}

/// Read-only, exact-attempt outcome. Holding the observer also holds the
/// caller's prepaid capacity ticket for its Arc allocation.
#[derive(Clone)]
pub struct CompanionPublicationCompletionObserver {
    core: Arc<CompletionCore>,
}

impl fmt::Debug for CompanionPublicationCompletionObserver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompanionPublicationCompletionObserver")
            .field("state", &self.state())
            .finish()
    }
}

impl CompanionPublicationCompletionObserver {
    pub fn state(&self) -> CompanionPublicationCompletion {
        match self.core.state.load(Ordering::Acquire) {
            PREPARED => CompanionPublicationCompletion::Prepared,
            INSTALLED => CompanionPublicationCompletion::Installed,
            ABORTED => CompanionPublicationCompletion::Aborted,
            _ => unreachable!("native completion state has three values"),
        }
    }

    pub(super) fn new() -> Self {
        Self {
            core: Arc::new(CompletionCore {
                state: AtomicU8::new(PREPARED),
            }),
        }
    }

    pub(super) fn installed(&self) {
        self.core.state.store(INSTALLED, Ordering::Release);
    }

    pub(super) fn abort_uninstalled(&self) {
        let _ = self.core.state.compare_exchange(
            PREPARED,
            ABORTED,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}
