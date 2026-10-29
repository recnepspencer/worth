//! The erased node header is admitted separately from its already paid array.
use super::{
    WorthQueryManagedIndexNodes, WorthQueryManagedIndexProvider, WorthQueryManagedIndexStop,
};
use std::{any::Any, marker::PhantomData, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

pub(super) struct ProviderNodeBacking<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    pub(super) nodes: WorthQueryManagedIndexNodes<P::Node>,
    _memory: Option<ExecutionMemoryReservation>,
    marker: PhantomData<fn() -> (S, I, P)>,
}

pub(super) fn retain<S, I, P>(
    execution: &ExecutionRequest<'_, '_>,
    nodes: WorthQueryManagedIndexNodes<P::Node>,
) -> Result<Arc<dyn Any + Send + Sync>, WorthQueryManagedIndexStop>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    let bytes = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<ProviderNodeBacking<S, I, P>>())
        .ok()
        .map(|(layout, _)| layout.pad_to_align().size())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(WorthQueryManagedIndexStop::CapacityOverflow)?;
    let memory = execution
        .reserve_retained_memory(bytes)
        .map_err(WorthQueryManagedIndexStop::Request)?;
    Ok(Arc::new(ProviderNodeBacking::<S, I, P> {
        nodes,
        _memory: memory,
        marker: PhantomData,
    }))
}
