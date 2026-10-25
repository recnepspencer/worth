//! Borrowed request authority with scoped execution and fixed-size backing metadata.

use super::{ExecutionResourceLease, SerialRequest};
use crate::{ExecutionWorkCeiling, WorkCeilingDenial};
use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

/// Sealed serial or leased backing borrowed from the host call.
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
    pub fn has_accounting(self) -> bool {
        match self.form {
            RequestBacking::Leased(lease) => lease.has_accounting(),
            RequestBacking::Serial(request) => request.memory().is_some(),
        }
    }

    /// Run using the original worker and control authority, without a payload
    /// or cost contract. Deliberately accounted ancestors remain authoritative.
    pub fn run_native<R>(
        self,
        operation: impl for<'scope> FnOnce(Option<&'scope ExecutionResourceLease<'authority>>) -> R,
    ) -> Result<(R, ExecutionPhysicalReport), WorkCeilingDenial> {
        self.run_native_with_checkpoint(|_| match self.form {
            RequestBacking::Leased(lease) => {
                let active = lease.current_descendant();
                operation(Some(active.as_ref().unwrap_or(lease)))
            }
            RequestBacking::Serial(_) => operation(None),
        })
    }

    /// Lend the same request's control checkpoint. A swallowed first stop is
    /// still refused at handoff after the callback completes.
    pub fn run_native_with_checkpoint<R>(
        self,
        operation: impl FnOnce(&mut crate::MapKernelContext<'_, '_>) -> R,
    ) -> Result<(R, ExecutionPhysicalReport), WorkCeilingDenial> {
        self.run_native_checkpoint_report(None, operation)
            .map(|(value, report)| (value, report.physical()))
    }

    /// Run a native diagnostic on the existing isolated test lane, sharing
    /// physical workers and original controls without changing worker-only mode.
    #[doc(hidden)]
    pub fn run_native_detached_for_test<R>(
        self,
        operation: impl FnOnce(Option<&ExecutionResourceLease<'_>>) -> R,
    ) -> Result<(R, ExecutionPhysicalReport), WorkCeilingDenial> {
        match self.form {
            RequestBacking::Leased(lease) => {
                let resources = lease.diagnostic_resources_for_test();
                let diagnostic = resources.lease_for_test();
                let _detached = crate::backend::detach_meter_for_test();
                let _worker = resources.detach_worker_context();
                ExecutionRequest::leased(&diagnostic).run_native(operation)
            }
            RequestBacking::Serial(_) => {
                let _detached = crate::backend::detach_meter_for_test();
                self.run_native(operation)
            }
        }
    }

    /// A native pattern owns its typed failure. A successful pattern must also
    /// pass the original request's final control boundary before handoff.
    pub(crate) fn run_native_report<R, E>(
        self,
        ceiling: ExecutionWorkCeiling,
        operation: impl for<'scope> FnOnce(
            Option<&'scope ExecutionResourceLease<'authority>>,
        ) -> Result<R, E>,
    ) -> Result<(Result<R, E>, ExecutionReport), WorkCeilingDenial> {
        use crate::backend::{KernelFailure, ScopeStop};
        let outcome = self.run_native_scope(ceiling.work_units(), |context| {
            let value = match self.form {
                RequestBacking::Leased(lease) => {
                    let active = lease.current_descendant();
                    operation(Some(active.as_ref().unwrap_or(lease)))
                }
                RequestBacking::Serial(_) => operation(None),
            }
            .map_err(KernelFailure::Domain)?;
            context.poll()?;
            Ok(value)
        });
        match outcome.result {
            Ok(value) => Ok((Ok(value), outcome.report)),
            Err(ScopeStop::Failure(KernelFailure::Domain(cause))) => {
                Ok((Err(cause), outcome.report))
            }
            Err(ScopeStop::Admission(cause)) => Err(WorkCeilingDenial::Admission(cause)),
            Err(ScopeStop::Failure(KernelFailure::Stop(stop))) => {
                Err(WorkCeilingDenial::Stopped(stop))
            }
            Err(_) => Err(WorkCeilingDenial::Panicked),
        }
    }

    fn run_native_checkpoint_report<R>(
        self,
        ceiling: Option<u64>,
        operation: impl FnOnce(&mut crate::MapKernelContext<'_, '_>) -> R,
    ) -> Result<(R, ExecutionReport), WorkCeilingDenial> {
        let outcome = self.run_native_scope::<R, std::convert::Infallible>(ceiling, |context| {
            let value = operation(context);
            context.poll()?;
            Ok(value)
        });
        use crate::backend::{KernelFailure, ScopeStop};
        match outcome.result {
            Ok(value) => Ok((value, outcome.report)),
            Err(ScopeStop::Admission(cause)) => Err(WorkCeilingDenial::Admission(cause)),
            Err(ScopeStop::Failure(KernelFailure::Stop(stop))) => {
                Err(WorkCeilingDenial::Stopped(stop))
            }
            Err(_) => Err(WorkCeilingDenial::Panicked),
        }
    }

    fn run_native_scope<R, E>(
        self,
        ceiling: Option<u64>,
        operation: impl FnOnce(
            &mut crate::MapKernelContext<'_, '_>,
        ) -> Result<R, crate::backend::KernelFailure<E>>,
    ) -> crate::backend::ScopeOutcome<R, E> {
        let active = match self.form {
            RequestBacking::Leased(lease) => lease.current_descendant(),
            RequestBacking::Serial(_) => None,
        };
        let (lease, serial) = match self.form {
            RequestBacking::Leased(lease) => (Some(active.as_ref().unwrap_or(lease)), None),
            RequestBacking::Serial(serial) => (None, Some(serial)),
        };
        crate::backend::run_scope_native_within(lease, true, ceiling, serial, operation)
    }

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

    /// Inspects interruption without charging work or opening an execution scope.
    pub fn check_live(self) -> Result<(), crate::MapKernelStop> {
        consult_observation::reached();
        let (cancelled, expired) = match self.form {
            RequestBacking::Leased(lease) => {
                let status = lease.status();
                (status.is_cancelled(), status.deadline_elapsed())
            }
            RequestBacking::Serial(request) => (
                request.cancellation.is_cancelled(),
                request
                    .deadline
                    .is_some_and(|deadline| std::time::Instant::now() >= deadline),
            ),
        };
        if cancelled {
            consult_observation::stopped(None);
            return Err(crate::MapKernelStop::Cancelled);
        }
        if expired {
            consult_observation::stopped(None);
            return Err(crate::MapKernelStop::DeadlineElapsed);
        }
        Ok(())
    }

    pub fn is_leased(self) -> bool {
        matches!(self.form, RequestBacking::Leased(_))
    }

    /// Fixed-size backing metadata; commits derive their allocation authority
    /// from this request rather than accepting an independent policy.
    /// The policy borrows the backing, not its active scope. Use it only for
    /// the originating host call: do not retain it after that scope closes,
    /// carry it into another host call, or mint a request from its lease.
    pub fn allocation_policy(self) -> super::ExecutionAllocationPolicy<'request, 'authority> {
        match self.form {
            RequestBacking::Leased(lease) => super::ExecutionAllocationPolicy::Execution(lease),
            RequestBacking::Serial(_) => super::ExecutionAllocationPolicy::SystemAllocation,
        }
    }

    /// The request's own policy ceiling. This does not observe active ancestors
    /// or subtract live holds; use `ExecutionMemoryReservation::available_in_scope`
    /// inside `in_scope` when planning admission against the complete lineage.
    pub fn memory_limit(self) -> Option<u64> {
        match self.form {
            RequestBacking::Leased(lease) => lease
                .policy()
                .accounting()
                .and_then(|budget| budget.charged_memory_bytes()),
            RequestBacking::Serial(request) => request.memory().and_then(|memory| memory.limit()),
        }
    }

    /// The request-level ceiling used to narrow a computation's declared work.
    /// Absence preserves the computation declaration without adding a request ceiling.
    /// Charged work is still metered and counter overflow remains a typed stop.
    pub fn work_ceiling(self) -> Option<u64> {
        match self.form {
            RequestBacking::Leased(lease) => lease
                .policy()
                .accounting()
                .and_then(|budget| budget.work_ceiling()),
            RequestBacking::Serial(request) => request.work_ceiling,
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
            RequestBacking::Leased(lease) => lease.policy().max_workers().get(),
            RequestBacking::Serial(_) => 1,
        }
    }

    /// Read cancellation, deadline and exhausted active work without charging
    /// work, allocating, opening a scope or containing a panic.
    pub fn consult(self) -> Result<(), WorkCeilingDenial> {
        consult_observation::reached();
        crate::backend::record_request_consult_for_test();
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
        stop.map_or(Ok(()), |stop| {
            consult_observation::stopped(None);
            Err(WorkCeilingDenial::Stopped(stop))
        })
    }

    /// Adapt within the original request scope. Leased carriage adds no scope.
    pub fn in_scope<R>(
        self,
        operation: impl for<'scope> FnOnce(Option<&'scope ExecutionResourceLease<'authority>>) -> R,
    ) -> Result<R, WorkCeilingDenial> {
        match self.form {
            RequestBacking::Leased(lease) => {
                let active = lease.current_descendant();
                Ok(operation(Some(active.as_ref().unwrap_or(lease))))
            }
            RequestBacking::Serial(_) if !self.has_accounting() => {
                self.run_native(operation).map(|(value, _)| value)
            }
            RequestBacking::Serial(request) => ExecutionWorkCeiling::new(self.work_ceiling())
                .run_serial(request, || operation(None))
                .map(|(value, _)| value),
        }
    }

    /// Runs an opaque-result stage with the original request and retained reservation.
    /// Known input/framework memory is admitted before invocation. Actual result or
    /// domain-error heap is admitted before framework retention, then remains on `memory`.
    /// This is post-materialization admission; callbacks needing pre-allocation
    /// admission must reserve their original backing before allocation themselves.
    pub fn run_measured_result<R: crate::ChargedBytes, E: crate::ChargedBytes>(
        self,
        ceiling: ExecutionWorkCeiling,
        retained_bytes: u64,
        memory: &mut super::ExecutionMemoryReservation,
        kernel: impl FnOnce(
            &mut crate::MapKernelContext<'_, '_>,
            &mut crate::ExecutionResultMemory<'_>,
        ) -> Result<R, crate::MapKernelFailure<E>>,
    ) -> crate::ScopeOutcome<R, E> {
        match self.form {
            RequestBacking::Leased(lease) => {
                let active = lease.current_descendant();
                crate::backend::run_scope_measured(
                    Some(active.as_ref().unwrap_or(lease)),
                    None,
                    ceiling.work_units(),
                    retained_bytes,
                    memory,
                    kernel,
                )
            }
            RequestBacking::Serial(serial) => crate::backend::run_scope_measured(
                None,
                Some(serial),
                ceiling.work_units(),
                retained_bytes,
                memory,
                kernel,
            ),
        }
    }

    /// Projects the optional lease only while its original execution scope is active.
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

#[path = "request/consult_observation.rs"]
pub(crate) mod consult_observation;
