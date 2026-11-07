//! Consuming initial-owner stop; allocation destruction remains unbudgeted.
use crate::logic::transaction::SignalRuntime;
use worth_execution::{ExecutionRequest, MapKernelStop};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalInitialEmptyStopDenial {
    Busy,
    Poisoned,
    Unsupported,
    OwnerUnavailable,
    Stopped(MapKernelStop),
}

/// Untouched owning runtime after preparation refusal.
#[must_use]
pub struct SignalInitialEmptyStopRefused<D, I, E, Ctx, T = ()>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime: SignalRuntime<D, I, E, Ctx, T>,
    reason: SignalInitialEmptyStopDenial,
}
impl<D, I, E, Ctx, T> SignalInitialEmptyStopRefused<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(
        runtime: SignalRuntime<D, I, E, Ctx, T>,
        reason: SignalInitialEmptyStopDenial,
    ) -> Self {
        Self { runtime, reason }
    }
    pub fn reason(&self) -> SignalInitialEmptyStopDenial {
        self.reason
    }
    pub fn into_runtime(self) -> SignalRuntime<D, I, E, Ctx, T> {
        self.runtime
    }
}

/// Original root in reversible CLOSING|0. No operation admission or cleanup
/// coordinator is retained. Generic Drop is UNBUDGETED owner abandonment.
#[must_use]
pub struct PreparedInitialEmptySignalStop<D, I, E, Ctx, T = ()>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime: SignalRuntime<D, I, E, Ctx, T>,
}
impl<D, I, E, Ctx, T> PreparedInitialEmptySignalStop<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(runtime: SignalRuntime<D, I, E, Ctx, T>) -> Self {
        Self { runtime }
    }
    pub fn resume(
        self,
    ) -> Result<SignalRuntime<D, I, E, Ctx, T>, (Self, SignalInitialEmptyStopDenial)> {
        match self.runtime.try_resume_initial_empty_owner() {
            Ok(()) => Ok(self.runtime),
            Err(reason) => Err((self, reason)),
        }
    }
    /// Busy or stop retains this SAME prepared owner. No service is resumed.
    pub fn commit(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        StoppedInitialEmptySignalAllocation<D, I, E, Ctx, T>,
        (Self, SignalInitialEmptyStopDenial),
    > {
        match self.runtime.try_commit_initial_empty_owner_stop(request) {
            Ok(()) => Ok(StoppedInitialEmptySignalAllocation {
                _runtime: self.runtime,
            }),
            Err(reason) => Err((self, reason)),
        }
    }
}

/// Stopped original allocation, NOT Signal cleanup or Signal Closed.
/// Retains the actual root and all original user payloads until UNBUDGETED Drop.
#[must_use]
pub struct StoppedInitialEmptySignalAllocation<D, I, E, Ctx, T = ()>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    _runtime: SignalRuntime<D, I, E, Ctx, T>,
}
impl<D, I, E, Ctx, T> StoppedInitialEmptySignalAllocation<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn reclaim(self) {
        drop(self);
    }
}

macro_rules! opaque_debug {
    ($name:ident) => {
        impl<D, I, E, Ctx, T> std::fmt::Debug for $name<D, I, E, Ctx, T>
        where
            D: Copy + Ord + std::fmt::Debug + 'static,
            I: Copy + Ord,
            T: Copy + Ord,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(stringify!($name))
            }
        }
    };
}
opaque_debug!(PreparedInitialEmptySignalStop);
opaque_debug!(StoppedInitialEmptySignalAllocation);
opaque_debug!(SignalInitialEmptyStopRefused);
#[cfg(test)]
#[path = "initial_empty_stop_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
