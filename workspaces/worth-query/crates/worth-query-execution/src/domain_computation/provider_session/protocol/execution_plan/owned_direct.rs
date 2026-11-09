//! A coupled participant retains its real run until its provider session closes.
use super::*;
use std::sync::Mutex;

pub(in crate::domain_computation) enum CoupledProviderPlanAdmission<'run> {
    Admitted {
        plan: WorthQueryAdmittedProviderExecutionPlan<'run>,
        returned: ReturnedProviderDirectRun,
    },
    Refused {
        cause: WorthQueryProviderSessionFailure,
        returned: ReturnedProviderDirectRun,
    },
    AllocationDenied {
        run: WorthQueryRunningDirectRun,
        cause: worth_execution::ExecutionAllocationDenial,
    },
}
struct ReturnedRunState {
    run: Option<WorthQueryRunningDirectRun>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
pub(in crate::domain_computation) struct ReturnedProviderDirectRun {
    returned: Arc<Mutex<ReturnedRunState>>,
}
impl ReturnedProviderDirectRun {
    /// Available only after the protocol relinquishes custody. The commit
    /// owner then uses the ordinary terminal and cleanup owner for this run.
    pub(in crate::domain_computation) fn take(&self) -> Option<WorthQueryRunningDirectRun> {
        self.returned
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .run
            .take()
    }
}
pub(in crate::domain_computation::provider_session::protocol) struct OwnedDirectRun {
    run: Option<WorthQueryRunningDirectRun>,
    returned: Arc<Mutex<ReturnedRunState>>,
}
impl OwnedDirectRun {
    pub(super) fn new(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        run: WorthQueryRunningDirectRun,
    ) -> Result<
        (Box<Self>, ReturnedProviderDirectRun),
        (
            WorthQueryRunningDirectRun,
            worth_execution::ExecutionAllocationDenial,
        ),
    > {
        let bytes = (size_of::<Self>()
            + size_of::<Mutex<ReturnedRunState>>()
            + 2 * size_of::<usize>()) as u64;
        let memory = match execution.allocation_policy().reserve_memory(bytes) {
            Ok(memory) => memory,
            Err(cause) => return Err((run, cause)),
        };
        let returned = Arc::new(Mutex::new(ReturnedRunState {
            run: None,
            _memory: memory,
        }));
        Ok((
            Box::new(Self {
                run: Some(run),
                returned: returned.clone(),
            }),
            ReturnedProviderDirectRun { returned },
        ))
    }
    pub(super) fn run(&self) -> &WorthQueryRunningDirectRun {
        self.run
            .as_ref()
            .expect("provider protocol owns the live coupled run")
    }
}
impl Drop for OwnedDirectRun {
    fn drop(&mut self) {
        self.returned.lock().unwrap_or_else(|e| e.into_inner()).run = self.run.take();
    }
}
