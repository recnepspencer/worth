//! Borrow and advance the original erased slot and original handler allocation.
use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    WorthQueryOperationHomeRetirementDenial as Denial,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial as Failure;
use worth_execution::ExecutionRequest;
#[cfg(test)]
mod tests;

fn admit_entry_work(request: &ExecutionRequest<'_, '_>, work: u64) -> Result<(), Denial> {
    request
        .run_native_with_checkpoint(|context| context.checkpoint(work))
        .map_err(Denial::Execution)?
        .0
        .map_err(|stop| Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(stop)))
}

pub(super) type PreflightHandlerRetirement =
    fn(&mut Arc<dyn Any + Send + Sync>, &ExecutionRequest<'_, '_>) -> Result<(), Denial>;
pub(super) type AdvanceHandlerRetirement =
    fn(&mut Arc<dyn Any + Send + Sync>, &ExecutionRequest<'_, '_>) -> Result<Progress, Denial>;

/// The original static binding of the last completed actual entry.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct HandlerRetirementCursor {
    completed: Option<&'static str>,
}

fn original_handler<Schema, Binding>(
    value: &mut Arc<dyn Any + Send + Sync>,
) -> Result<&mut dyn OperationHandler<Schema, Binding>, Denial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    let erased = Arc::get_mut(value).ok_or(Denial::Busy)?;
    let slot = erased
        .downcast_mut::<TypedHandlerSlot<Schema, Binding>>()
        .expect("registration retains its original Schema and Binding thunk");
    Arc::get_mut(&mut slot.handler).ok_or(Denial::Busy)
}

pub(super) fn preflight_original_handler<Schema, Binding>(
    value: &mut Arc<dyn Any + Send + Sync>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    request.consult().map_err(Denial::Execution)?;
    original_handler::<Schema, Binding>(value)?.preflight_home_retirement(request)
}

pub(super) fn advance_original_handler<Schema, Binding>(
    value: &mut Arc<dyn Any + Send + Sync>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Progress, Denial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    request.consult().map_err(Denial::Execution)?;
    original_handler::<Schema, Binding>(value)?.advance_home_retirement(request)
}

impl<Schema: ApplicationSchema> InstalledMutationHandlerRegistry<Schema> {
    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        for entry in self.entries.values_mut() {
            let binding = entry.identity;
            admit_entry_work(request, 1).map_err(|cause| Failure::Handler { binding, cause })?;
            (entry.preflight_home_retirement)(&mut entry.value, request)
                .map_err(|cause| Failure::Handler { binding, cause })?;
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn advance_home_retirement(
        &mut self,
        cursor: &mut HandlerRetirementCursor,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Progress, Failure> {
        use std::ops::Bound::{Excluded, Unbounded};
        request.consult().map_err(Failure::Execution)?;
        let after = cursor.completed;
        let next = match after {
            Some(after) => self
                .entries
                .range_mut::<str, _>((Excluded(after), Unbounded))
                .next(),
            None => self.entries.iter_mut().next(),
        };
        let Some((_, entry)) = next else {
            return Ok(Progress::Complete);
        };
        let binding = entry.identity;
        // The registry already owns this static binding; advancing retirement
        // requires no fresh payload allocation after the close checkpoint.
        match (entry.advance_home_retirement)(&mut entry.value, request)
            .map_err(|cause| Failure::Handler { binding, cause })?
        {
            Progress::Advanced => Ok(Progress::Advanced),
            Progress::Complete => {
                cursor.completed = Some(binding);
                Ok(Progress::Advanced)
            }
        }
    }
}
