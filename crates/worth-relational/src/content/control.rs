use super::RelationalContentDenial as Denial;
use crate::execution::request_stage::checkpoint;
use worth_execution::{ExecutionRequest, MapKernelContext};

/// Borrowed original controls; codecs cannot replace this account or its first stop.
pub struct RelationalContentControl<'borrow, 'scope, 'lease> {
    context: &'borrow mut MapKernelContext<'scope, 'lease>,
    first_stop: Option<Denial>,
}

impl<'borrow, 'scope, 'lease> RelationalContentControl<'borrow, 'scope, 'lease> {
    fn new(context: &'borrow mut MapKernelContext<'scope, 'lease>) -> Self {
        Self {
            context,
            first_stop: None,
        }
    }

    pub fn checkpoint(&mut self, units: u64) -> Result<(), Denial> {
        if let Some(cause) = &self.first_stop {
            return Err(*cause);
        }
        match checkpoint(self.context, units) {
            Ok(()) => Ok(()),
            Err(cause) => {
                let cause = Denial::Execution(cause.into());
                self.first_stop = Some(cause);
                Err(cause)
            }
        }
    }
}

pub(super) fn run<T>(
    execution: &ExecutionRequest<'_, '_>,
    operation: impl FnOnce(&mut RelationalContentControl<'_, '_, '_>) -> Result<T, Denial>,
) -> Result<T, Denial> {
    let mut result = None;
    let mut operation = Some(operation);
    let handoff = crate::execution::request_stage::optional_stage(
        execution,
        |context, _memory| {
            let mut control = RelationalContentControl::new(context);
            let operation = operation
                .take()
                .expect("the original content stage runs once");
            result = Some(control.checkpoint(0).and_then(|()| operation(&mut control)));
            // Preserve the original classified stop even if a codec swallowed it.
            if let Err(cause) = control.checkpoint(0) {
                result = Some(Err(cause));
            }
        },
        |_| 0,
    );
    match handoff {
        Ok(_) => result.expect("a completed content scope invoked its callback"),
        Err(cause) => match result {
            Some(Err(cause)) => Err(cause),
            _ => Err(Denial::Execution(cause)),
        },
    }
}
