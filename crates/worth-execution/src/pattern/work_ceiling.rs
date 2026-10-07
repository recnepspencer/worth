use worth_foundational::ExecutionReport;

use crate::{
    authority::{ExecutionResourceLease, LeaseDenial, SerialRequest},
    backend::{run_scope_within, KernelFailure, KernelStop, ScopeStop},
};

/// The work a computation declared for itself.
///
/// A lease bounds a request; a computation inside it may declare less. The
/// patterns run inside [`ExecutionWorkCeiling::run`] inherit what remains of
/// the narrower bound, so their canonical exhaustion boundary is decided by
/// the declaration even when no lease is in reach and the serial backend runs.
/// The closure receives no kernel context: patterns stay the only way to
/// charge work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionWorkCeiling(u64);

/// Why a computation under a declared ceiling returned no value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkCeilingDenial {
    /// The lease refused the scope before the computation started.
    Admission(LeaseDenial),
    /// Cancellation or a deadline stopped the scope before the computation
    /// started.
    Stopped(KernelStop),
    /// The computation panicked outside every pattern.
    Panicked,
}

impl ExecutionWorkCeiling {
    pub const fn new(work_units: u64) -> Self {
        Self(work_units)
    }

    pub const fn work_units(self) -> u64 {
        self.0
    }

    /// Runs `computation` on the calling thread under `lease`. Its value is
    /// returned whenever it ran, beside the work every pattern inside it
    /// charged: a pattern that stopped has already reported its own typed
    /// stop. Every pattern inside runs under the same lease.
    pub fn run<R>(
        self,
        lease: &ExecutionResourceLease<'_>,
        computation: impl FnOnce() -> R,
    ) -> Result<(R, ExecutionReport), WorkCeilingDenial> {
        self.run_within(Some(lease), None, computation)
    }

    /// Runs `computation` with no lease, on the calling thread. Every
    /// lease-free pattern inside observes the request's cancellation and
    /// deadline. The scope and those patterns hold their bytes on the required
    /// memory budget, framework bytes at one worker, and refuse at its boundary.
    pub fn run_serial<R>(
        self,
        request: &SerialRequest,
        computation: impl FnOnce() -> R,
    ) -> Result<(R, ExecutionReport), WorkCeilingDenial> {
        self.run_within(None, Some(request), computation)
    }

    fn run_within<R>(
        self,
        lease: Option<&ExecutionResourceLease<'_>>,
        serial: Option<&SerialRequest>,
        computation: impl FnOnce() -> R,
    ) -> Result<(R, ExecutionReport), WorkCeilingDenial> {
        let mut value = None;
        let outcome = run_scope_within::<(), (), _>(lease, 0, 0, true, self.0, serial, |_| {
            value = Some(computation());
            Ok(())
        });
        match (value, outcome.result) {
            (Some(value), _) => Ok((value, outcome.report)),
            (None, Err(ScopeStop::Admission(denial))) => Err(WorkCeilingDenial::Admission(denial)),
            (None, Err(ScopeStop::Failure(KernelFailure::Stop(stop)))) => {
                Err(WorkCeilingDenial::Stopped(stop))
            }
            (None, _) => Err(WorkCeilingDenial::Panicked),
        }
    }
}
