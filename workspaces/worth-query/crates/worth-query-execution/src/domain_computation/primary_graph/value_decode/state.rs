//! The decode owner's first refusal and cumulative native memory custody.
use super::{session::ValueDecodeSession, WorthQueryValueDecodeDenial as Stop};
use std::cell::{Cell, RefCell};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, ExecutionWorkCeiling};
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog;

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

    /// The receiver retains grants and the first failure across individual
    /// observations. Its owner supplies the original borrowed request each time.
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
                ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                    lease,
                    None,
                    |context| {
                        if self.memory.borrow().is_none() {
                            *self.memory.borrow_mut() = Some(
                                ExecutionMemoryReservation::reserve_in_scope(lease, 0)
                                    .map_err(Stop::Reservation)?,
                            );
                        }
                        let context = RefCell::new(context);
                        let checkpoint = |work| context.borrow_mut().checkpoint(work);
                        let session = ValueDecodeSession {
                            state: self,
                            checkpoint: &checkpoint,
                        };
                        let output = body(&session);
                        if let Some(cause) = self.denial() {
                            drop(output);
                            return Err(cause);
                        }
                        Ok(output)
                    },
                )
            })
            .map_err(Stop::Scope)
            .and_then(|result| result.map_err(Stop::Scope))
            .and_then(|(result, _)| result);
        if let Err(cause) = &result {
            self.refuse(*cause);
        }
        match self.denial() {
            Some(cause) => Err(cause),
            None => result,
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn with_value_decode<R>(
    execution: &ExecutionRequest<'_, '_>,
    bindings: &WorthQueryInstalledApplicationValueBindingCatalog,
    body: impl FnOnce(&ValueDecodeSession<'_>) -> R,
) -> Result<R, Stop> {
    ValueDecodeState::new(bindings.clone()).run(execution, body)
}
