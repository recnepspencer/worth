//! Original decode control and optional explicitly accounted memory custody.
use super::{session::ValueDecodeSession, WorthQueryValueDecodeDenial as Stop};
use std::cell::{Cell, RefCell};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, ExecutionWorkCeiling};
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog;

#[cfg(test)]
mod tests;

pub(in crate::domain_computation::primary_graph) struct ValueDecodeState {
    pub(super) bindings: WorthQueryInstalledApplicationValueBindingCatalog,
    pub(super) memory: RefCell<Option<ExecutionMemoryReservation>>,
    first: Cell<Option<Stop>>,
}

impl ValueDecodeState {
    pub(in crate::domain_computation::primary_graph) fn new(
        bindings: WorthQueryInstalledApplicationValueBindingCatalog,
    ) -> Self {
        Self {
            bindings,
            memory: RefCell::new(None),
            first: Cell::new(None),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn denial(&self) -> Option<Stop> {
        self.first.get()
    }

    pub(super) fn refuse(&self, cause: Stop) {
        self.first.set(self.first.get().or(Some(cause)));
    }

    /// Poll the original request at the final handoff, without releasing the
    /// cumulative parse reservation or replacing an earlier refusal.
    pub(in crate::domain_computation::primary_graph) fn check(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Stop> {
        if let Some(cause) = self.denial() {
            return Err(cause);
        }
        execution.consult().map_err(|cause| {
            let cause = Stop::Scope(cause);
            self.refuse(cause);
            cause
        })
    }

    /// Retains the first failure and any explicitly accounted grants across reads.
    /// Its owner supplies the original borrowed request each time.
    pub(in crate::domain_computation::primary_graph) fn run<R>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        body: impl FnOnce(&ValueDecodeSession<'_>) -> R,
    ) -> Result<R, Stop> {
        if let Some(cause) = self.denial() {
            return Err(cause);
        }
        let result = execution
            .in_scope(|lease| {
                let decode = |context: &mut worth_execution::MapKernelContext<'_, '_>| {
                    let accounted = context.has_accounting();
                    if accounted && self.memory.borrow().is_none() {
                        *self.memory.borrow_mut() = Some(
                            ExecutionMemoryReservation::reserve_in_scope(lease, 0)
                                .map_err(Stop::Reservation)?,
                        );
                    }
                    let context = RefCell::new(context);
                    let checkpoint = |work| context.borrow_mut().checkpoint(work);
                    let poll = || context.borrow_mut().poll();
                    let session = ValueDecodeSession {
                        state: self,
                        checkpoint: accounted.then_some(
                            &checkpoint
                                as &dyn Fn(u64) -> Result<(), worth_execution::MapKernelStop>,
                        ),
                        poll: &poll,
                    };
                    let output = body(&session);
                    if let Some(cause) = self.denial() {
                        drop(output);
                        return Err(cause);
                    }
                    Ok(output)
                };
                if execution.has_accounting() {
                    ExecutionWorkCeiling::new(execution.work_ceiling())
                        .run_with_checkpoint(lease, None, decode)
                        .map(|(result, _)| result)
                } else {
                    execution
                        .run_native_with_checkpoint(decode)
                        .map(|(result, _)| result)
                }
            })
            .map_err(Stop::Scope)
            .and_then(|result| result.map_err(Stop::Scope))
            .and_then(|result| result);
        if let Err(cause) = &result {
            self.refuse(*cause);
        }
        match self.denial() {
            Some(cause) => Err(cause),
            None => result,
        }
    }
}
