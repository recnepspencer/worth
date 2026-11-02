use super::RelationalContentDenial as Denial;
use crate::execution::request_stage::checkpoint;
use worth_execution::{ExecutionRequest, MapKernelContext};

/// Borrowed original controls; codecs cannot replace this account or its first stop.
pub struct RelationalContentControl<'borrow, 'scope, 'lease> {
    context: &'borrow mut MapKernelContext<'scope, 'lease>,
    first_stop: Option<Denial>,
    source: Option<&'borrow crate::validation::data::CustomInvariantReadBudget<'borrow>>,
    source_stop: Option<crate::validation::data::StructuralWalkDenialKind>,
}

impl<'borrow, 'scope, 'lease> RelationalContentControl<'borrow, 'scope, 'lease> {
    fn new(context: &'borrow mut MapKernelContext<'scope, 'lease>) -> Self {
        Self {
            context,
            first_stop: None,
            source: None,
            source_stop: None,
        }
    }

    pub fn checkpoint(&mut self, units: u64) -> Result<(), Denial> {
        if let Some(cause) = &self.first_stop {
            return Err(*cause);
        }
        let result = if let Some(source) = self.source {
            use crate::validation::data::CustomInvariantNativeCheckpointDenial as Stop;
            match source.checkpoint_native(self.context, units) {
                Ok(()) => Ok(()),
                Err(Stop::Request(cause)) => Err(cause),
                Err(Stop::Source(cause)) => {
                    self.source_stop = Some(cause);
                    self.first_stop = Some(Denial::EncodingInconsistency);
                    return Err(Denial::EncodingInconsistency);
                }
            }
        } else {
            checkpoint(self.context, units)
        };
        match result {
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

/// Keep the exact Source refusal outside codec vocabulary; the sentinel never escapes.
pub(super) fn run_source<T>(
    budget: &crate::validation::data::CustomInvariantReadBudget<'_>,
    operation: impl FnOnce(&mut RelationalContentControl<'_, '_, '_>) -> Result<T, Denial>,
) -> Result<T, crate::validation::candidate_source::ManagedIndexSourcePreparationDenial> {
    use crate::derived_relation::DerivedRelationReadDenial;
    use crate::validation::candidate_source::ManagedIndexSourcePreparationDenial as Read;
    let mut result = None;
    let mut source_stop = None;
    let mut operation = Some(operation);
    let handoff = crate::execution::request_stage::optional_stage(
        &budget.execution(),
        |context, _memory| {
            let mut control = RelationalContentControl::new(context);
            control.source = Some(budget);
            let operation = operation.take().expect("one original Source content stage");
            result = Some(control.checkpoint(0).and_then(|()| operation(&mut control)));
            if let Err(cause) = control.checkpoint(0) {
                result = Some(Err(cause));
            }
            source_stop = control.source_stop;
        },
        |_| 0,
    );
    if let Some(cause) = source_stop {
        return Err(DerivedRelationReadDenial::Structural(cause).into());
    }
    let result = match handoff {
        Ok(_) => result.expect("completed original Source content stage"),
        Err(cause) => match result {
            Some(Err(cause)) => Err(cause),
            _ => Err(Denial::Execution(cause)),
        },
    };
    result.map_err(|cause| match cause {
        Denial::Execution(cause) => Read::Source(DerivedRelationReadDenial::Request(cause)),
        cause => Read::Content(cause),
    })
}
