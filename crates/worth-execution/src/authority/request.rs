//! Borrowed request carriage with closure-scoped optional-lease adaptation.

use super::{ExecutionResourceLease, SerialRequest};
use crate::{ExecutionWorkCeiling, WorkCeilingDenial};
use worth_foundational::{ExecutionPosture, ExecutionReport};

/// Either backing is sealed; lease absence is exposed only inside an active scope closure.
#[derive(Clone, Copy, Debug)]
pub struct ExecutionRequest<'request, 'authority> {
    form: RequestBacking<'request, 'authority>,
}

#[derive(Clone, Copy, Debug)]
enum RequestBacking<'request, 'authority> {
    Leased(&'request ExecutionResourceLease<'authority>),
    Serial(&'request SerialRequest),
}

impl<'request, 'authority> ExecutionRequest<'request, 'authority> {
    pub fn leased(lease: &'request ExecutionResourceLease<'authority>) -> Self {
        Self {
            form: RequestBacking::Leased(lease),
        }
    }

    pub fn serial(request: &'request SerialRequest) -> Self {
        Self {
            form: RequestBacking::Serial(request),
        }
    }

    pub fn is_leased(self) -> bool {
        matches!(self.form, RequestBacking::Leased(_))
    }

    pub fn memory_limit(self) -> u64 {
        match self.form {
            RequestBacking::Leased(lease) => lease.policy().budget().charged_memory_bytes(),
            RequestBacking::Serial(request) => request.memory.limit(),
        }
    }

    /// The request-level ceiling used to narrow a computation's declared work.
    /// Serial requests bound memory only, so `u64::MAX` leaves that declaration unchanged.
    /// Charged work is still metered and counter overflow remains a typed stop.
    pub fn work_ceiling(self) -> u64 {
        match self.form {
            RequestBacking::Leased(lease) => lease.policy().budget().work_ceiling(),
            RequestBacking::Serial(_) => u64::MAX,
        }
    }

    pub fn resolved_posture(self) -> ExecutionPosture {
        match self.form {
            RequestBacking::Leased(lease) => lease.resolved_posture(),
            RequestBacking::Serial(_) => ExecutionPosture::Serial,
        }
    }

    pub fn max_workers(self) -> usize {
        match self.form {
            RequestBacking::Leased(lease) => lease.policy().budget().max_workers().get(),
            RequestBacking::Serial(_) => 1,
        }
    }

    /// Read cancellation, deadline and exhausted active work without charging
    /// work, allocating, opening a scope or containing a panic.
    pub fn consult(self) -> Result<(), WorkCeilingDenial> {
        use crate::backend::KernelStop;
        let backing = match self.form {
            RequestBacking::Leased(lease) => lease.status().stop(),
            RequestBacking::Serial(request) => request.stop(),
        };
        let active = crate::backend::active_stop();
        let stop =
            if backing == Some(KernelStop::Cancelled) || active == Some(KernelStop::Cancelled) {
                Some(KernelStop::Cancelled)
            } else {
                backing.or(active)
            };
        stop.map_or(Ok(()), |stop| Err(WorkCeilingDenial::Stopped(stop)))
    }

    /// Adapt only within an active bounded scope. Leased carriage adds no scope.
    pub fn in_scope<R>(
        self,
        operation: impl for<'scope> FnOnce(Option<&'scope ExecutionResourceLease<'authority>>) -> R,
    ) -> Result<R, WorkCeilingDenial> {
        match self.form {
            RequestBacking::Leased(lease) => Ok(operation(Some(lease))),
            RequestBacking::Serial(request) => ExecutionWorkCeiling::new(self.work_ceiling())
                .run_serial(request, || operation(None))
                .map(|(value, _)| value),
        }
    }

    /// Projects the optional lease only while its bounded execution scope is active.
    pub fn run<R>(
        self,
        ceiling: ExecutionWorkCeiling,
        operation: impl for<'scope> FnOnce(Option<&'scope ExecutionResourceLease<'authority>>) -> R,
    ) -> Result<(R, ExecutionReport), WorkCeilingDenial> {
        match self.form {
            RequestBacking::Leased(lease) => ceiling.run(lease, || operation(Some(lease))),
            RequestBacking::Serial(request) => ceiling.run_serial(request, || operation(None)),
        }
    }
}
