//! Queue, producer and Native release are separate real cleanup handoffs.
use super::super::{
    occurrence::ResourceCandidateQueue, storage::ResourceStorageDenial,
    WorthQueryResourceCleanupPosture,
};
use super::*;

pub(in crate::domain_computation::primary_graph::application_resource) fn drain_candidate<
    Schema,
    Binding,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    row: &mut ResourceRowLease,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    let occurrence = row.occurrence_mut();
    let Some(candidate) = occurrence.candidate.as_mut() else {
        return Ok(());
    };
    if let Some(admission) = candidate.admission.as_ref() {
        if admission.report().denied_completion().is_some() {
            let ResourceLifecycle::Live { owner, .. } = &occurrence.lifecycle else {
                return Err(ResourceCause::Unavailable);
            };
            runtime
                .bridge
                .conditional()
                .release_process_local_completion_denial(owner, admission)
                .map_err(ResourceCause::Managed)?;
        }
    }
    if matches!(candidate.queue, ResourceCandidateQueue::Admitted(_)) {
        let ResourceLifecycle::Live { owner, .. } = &occurrence.lifecycle else {
            return Err(ResourceCause::Unavailable);
        };
        let drained = runtime
            .bridge
            .conditional()
            .dequeue_managed_owned_async_queue(owner, 1)
            .map_err(ResourceCause::Managed)?;
        candidate.queue = ResourceCandidateQueue::Drained(drained);
    }
    // Drop releases only the buffered frame/scalar. Every successful Native
    // enqueue has already reached its one actual dequeue above.
    occurrence.candidate = None;
    Ok(())
}

pub(in crate::domain_computation::primary_graph::application_resource) fn close_provider<
    Schema,
    Binding,
>(
    execution: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<Schema, Binding>,
    row: &mut ResourceRowLease,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    let occurrence = row.occurrence_mut();
    if occurrence.provider_closed {
        return Ok(());
    }
    let index = occurrence.native_slot.ok_or(ResourceCause::Unavailable)?;
    let attempt = occurrence
        .attempt
        .as_ref()
        .ok_or(ResourceCause::Unavailable)?;
    installed.storage.nodes[index]
        .ingress
        .close(attempt)
        .map_err(ResourceCause::Ingress)?;
    execution.consult().map_err(ResourceCause::Request)?;
    let closed = installed.provider.close(attempt);
    // Store the real callback outcome before an original control refusal.
    occurrence.provider_closed = closed == WorthQueryResourceCleanupPosture::Closed;
    execution.consult().map_err(ResourceCause::Request)?;
    if !occurrence.provider_closed {
        return Err(ResourceCause::Storage(
            ResourceStorageDenial::CleanupPending,
        ));
    }
    Ok(())
}

pub(in crate::domain_computation::primary_graph::application_resource) fn finish_native<
    Schema,
    Binding,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    row: &mut ResourceRowLease,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    let occurrence = row.occurrence_mut();
    let ResourceLifecycle::Live { owner, completion } = &mut occurrence.lifecycle else {
        return Ok(());
    };
    if owner.wake_cleanup_pending() {
        runtime
            .bridge
            .conditional()
            .finish_managed_owned_async_wake_cleanup(owner)
            .map_err(ResourceCause::Managed)?;
    }
    if let Some(completion) = completion.as_mut() {
        if !completion.timeout_cleanup_complete() {
            runtime
                .bridge
                .conditional()
                .finish_owned_async_completion_timeout_cleanup(completion)
                .map_err(ResourceCause::Completion)?;
        }
    }
    if owner.has_pending_retry() {
        runtime
            .bridge
            .conditional()
            .abort_process_local_owned_async_retry(owner)
            .map_err(ResourceCause::Managed)?;
    }
    occurrence.retry = None;
    if completion.is_none() {
        runtime
            .bridge
            .conditional()
            .cancel_managed_owned_async_request(owner)
            .map_err(ResourceCause::Managed)?;
    }
    // No request callback or allocation runs between the consuming Native
    // release and installation of its genuine Released/refused-owner outcome.
    let prior = std::mem::replace(
        &mut occurrence.lifecycle,
        ResourceLifecycle::Reserved { clock: None },
    );
    let ResourceLifecycle::Live { owner, completion } = prior else {
        unreachable!()
    };
    match runtime
        .bridge
        .conditional()
        .release_process_local_managed_request(owner, completion)
    {
        Ok(released) => occurrence.lifecycle = ResourceLifecycle::Released(released),
        Err(refused) => {
            let (owner, completion, cause) = refused.into_parts();
            occurrence.lifecycle = ResourceLifecycle::Live { owner, completion };
            return Err(ResourceCause::Managed(cause));
        }
    }
    occurrence.timeout = None;
    Ok(())
}

pub(in crate::domain_computation::primary_graph::application_resource) fn close<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<Schema, Binding>,
    row: &mut ResourceRowLease,
    recovery_handoff: bool,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    if row.occurrence().recovery.is_some() && !recovery_handoff {
        return Err(ResourceCause::RecoveryHandoffRequired);
    }
    drain_candidate::<Schema, Binding>(runtime, row)?;
    close_provider(execution, installed, row)?;
    finish_native::<Schema, Binding>(runtime, row)?;
    row.finish_native_slot().map_err(ResourceCause::Storage)
}
