//! Borrow and advance the original erased slot and original handler allocation.
use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    WorthQueryOperationHomeRetirementDenial as Denial,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial as Failure;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
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

/// The last completed actual entry, retained with its original request grant.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct HandlerRetirementCursor {
    completed: Option<RetiredHandlerIdentity>,
}
struct RetiredHandlerIdentity {
    identity: String,
    _funding: Option<ExecutionMemoryReservation>,
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
        let after = cursor
            .completed
            .as_ref()
            .map(|entry| entry.identity.as_str());
        let next = match after {
            Some(after) => self
                .entries
                .range_mut::<str, _>((Excluded(after), Unbounded))
                .next(),
            None => self.entries.iter_mut().next(),
        };
        let Some((identity, entry)) = next else {
            return Ok(Progress::Complete);
        };
        let binding = entry.identity;
        admit_entry_work(request, 1 + identity.len() as u64)
            .map_err(|cause| Failure::Handler { binding, cause })?;
        // Fund the next cursor before calling a potentially advancing owner.
        let funding = request
            .reserve_retained_memory(identity.len() as u64)
            .map_err(|cause| Failure::Handler {
                binding,
                cause: Denial::Execution(cause),
            })?;
        let completed = RetiredHandlerIdentity {
            identity: identity.clone(),
            _funding: funding,
        };
        match (entry.advance_home_retirement)(&mut entry.value, request)
            .map_err(|cause| Failure::Handler { binding, cause })?
        {
            Progress::Advanced => Ok(Progress::Advanced),
            Progress::Complete => {
                cursor.completed = Some(completed);
                Ok(Progress::Advanced)
            }
        }
    }
}
