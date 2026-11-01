//! Effectful callback custody through the original request scope's settlement.
use super::{ExecutionRequest, RequestBacking};
use crate::backend::{KernelFailure, ScopeStop};
use crate::{ExecutionResourceLease, ExecutionWorkCeiling, WorkCeilingDenial};
use worth_foundational::{ExecutionPhysicalReport, ExecutionReport};

impl<'authority> ExecutionRequest<'_, 'authority> {
    /// Complete effectful work on the original request, preserving its returned owner.
    ///
    /// Unlike `run_native` and pre-effect-only `prepare_native`, a returned
    /// callback value survives final or nested execution stops. The callback
    /// must carry its own business result and any partial-effect custody;
    /// returning a value does not prove that canonical work committed.
    /// Admission or interruption before callback entry returns a typed denial.
    /// A callback panic before returning retains ordinary panic containment.
    ///
    /// The same active descendant, remaining work and latched parent stops
    /// remain authoritative. This opens no fresh allowance and adds no
    /// accounting contract to a worker-only request. The report is the actual
    /// scope's physical observation. Returned-value custody adds constant
    /// space; scope admission and controls retain their existing lineage costs.
    pub fn run_native_completion<R>(
        self,
        operation: impl for<'scope> FnOnce(Option<&'scope ExecutionResourceLease<'authority>>) -> R,
    ) -> Result<(R, ExecutionPhysicalReport), WorkCeilingDenial> {
        // Scope admission keeps lineage precedence. Preserve the original
        // meter's control fact before entering its descendant scope.
        let original_refusal = self.completion_refusal();
        let mut value = None;
        let outcome = self.run_native_scope::<(), std::convert::Infallible>(None, |context| {
            if original_refusal.is_some() {
                // The invoking meter already owns this stop. Observe admission
                // without settling that same failure again as a child outcome.
                return Ok(());
            }
            context.poll()?;
            value = Some(match self.form {
                RequestBacking::Leased(lease) => {
                    let active = lease.current_descendant();
                    operation(Some(active.as_ref().unwrap_or(lease)))
                }
                RequestBacking::Serial(_) => operation(None),
            });
            context.poll()?;
            Ok(())
        });
        match (value, outcome.result) {
            (Some(value), _) => Ok((value, outcome.report.physical())),
            (None, Ok(())) => Err(original_refusal.expect("uninvoked callback retains refusal")),
            (None, Err(ScopeStop::Admission(cause))) => Err(WorkCeilingDenial::Admission(cause)),
            (None, Err(ScopeStop::Failure(KernelFailure::Stop(stop)))) => {
                Err(WorkCeilingDenial::Stopped(stop))
            }
            (None, _) => Err(WorkCeilingDenial::Panicked),
        }
    }

    /// Complete effectful work under the original request and declared ceiling.
    ///
    /// The existing accounted scope retains the exact returned business value
    /// through later control stops and returns its actual aggregate report.
    /// Admission or an original invoking stop prevents callback entry. An
    /// admitted observation of an existing stop never settles that cause again.
    /// The active descendant supplies admission, memory and worker policy before
    /// the scope opens; this creates neither another meter nor fresh allowance.
    /// A returned value is not evidence that canonical business work committed.
    pub fn run_completion<R>(
        self,
        ceiling: ExecutionWorkCeiling,
        operation: impl for<'scope> FnOnce(Option<&'scope ExecutionResourceLease<'authority>>) -> R,
    ) -> Result<(R, ExecutionReport), WorkCeilingDenial> {
        let original_refusal = self.completion_refusal();
        let guarded = |lease| {
            if original_refusal.is_some() {
                None
            } else {
                Some(operation(lease))
            }
        };
        let outcome = match self.form {
            RequestBacking::Leased(lease) => {
                let active = lease.current_descendant();
                let carried = active.as_ref().unwrap_or(lease);
                ceiling.run(carried, || guarded(Some(carried)))
            }
            RequestBacking::Serial(serial) => ceiling.run_serial(serial, || guarded(None)),
        };
        outcome.and_then(|(value, report)| match value {
            Some(value) => Ok((value, report)),
            None => Err(original_refusal.expect("uninvoked callback retains refusal")),
        })
    }

    fn completion_refusal(self) -> Option<WorkCeilingDenial> {
        self.consult().err().or_else(|| {
            crate::backend::active_nested_stop_owner()
                .map(|_| WorkCeilingDenial::Stopped(crate::backend::KernelStop::NestedStopped))
        })
    }
}
