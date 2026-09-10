use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

#[cfg(test)]
mod cancellation_tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRequestInterruption {
    Cancelled,
    DeadlineExceeded,
}

#[derive(Default)]
struct CancellationState {
    cancelled: AtomicBool,
    waiters: Mutex<Vec<(std::sync::Weak<()>, Waker)>>,
}

#[derive(Clone, Default)]
pub struct WorthQueryCancellationSource {
    state: Arc<CancellationState>,
}

impl WorthQueryCancellationSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token(&self) -> WorthQueryCancellationToken {
        WorthQueryCancellationToken {
            state: Arc::clone(&self.state),
        }
    }

    pub fn cancel(&self) {
        if self.state.cancelled.swap(true, Ordering::AcqRel) {
            return;
        }
        let waiters = {
            let mut waiters = self
                .state
                .waiters
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            std::mem::take(&mut *waiters)
        };
        for (_, waiter) in waiters {
            waiter.wake();
        }
    }
}

#[derive(Clone)]
pub struct WorthQueryCancellationToken {
    state: Arc<CancellationState>,
}

impl WorthQueryCancellationToken {
    pub fn is_cancelled(&self) -> bool {
        self.state.cancelled.load(Ordering::Acquire)
    }

    pub fn cancelled(&self) -> WorthQueryCancellationFuture<'_> {
        WorthQueryCancellationFuture {
            token: self,
            registration: Arc::new(()),
        }
    }
}

pub struct WorthQueryCancellationFuture<'a> {
    token: &'a WorthQueryCancellationToken,
    registration: Arc<()>,
}

impl Future for WorthQueryCancellationFuture<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            return Poll::Ready(());
        }
        let mut waiters = self
            .token
            .state
            .waiters
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.token.is_cancelled() {
            return Poll::Ready(());
        }
        let identity = Arc::downgrade(&self.registration);
        if let Some((_, waiter)) = waiters.iter_mut().find(|(key, _)| key.ptr_eq(&identity)) {
            waiter.clone_from(context.waker());
        } else {
            waiters.push((identity, context.waker().clone()));
        }
        Poll::Pending
    }
}

impl Drop for WorthQueryCancellationFuture<'_> {
    fn drop(&mut self) {
        let identity = Arc::downgrade(&self.registration);
        self.token
            .state
            .waiters
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|(key, _)| !key.ptr_eq(&identity));
    }
}

#[derive(Clone)]
pub struct WorthQueryRequestScope {
    deadline: Instant,
    cancellation: WorthQueryCancellationToken,
}

impl WorthQueryRequestScope {
    pub fn new(deadline: Instant, cancellation: WorthQueryCancellationToken) -> Self {
        Self {
            deadline,
            cancellation,
        }
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }

    pub fn cancellation(&self) -> &WorthQueryCancellationToken {
        &self.cancellation
    }

    pub fn remaining(&self) -> Option<Duration> {
        self.deadline.checked_duration_since(Instant::now())
    }

    pub fn interruption(&self) -> Option<WorthQueryRequestInterruption> {
        if self.cancellation.is_cancelled() {
            Some(WorthQueryRequestInterruption::Cancelled)
        } else if Instant::now() >= self.deadline {
            Some(WorthQueryRequestInterruption::DeadlineExceeded)
        } else {
            None
        }
    }
}

impl std::fmt::Debug for WorthQueryRequestScope {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryRequestScope")
            .field("deadline", &self.deadline)
            .field("cancelled", &self.cancellation.is_cancelled())
            .finish()
    }
}
