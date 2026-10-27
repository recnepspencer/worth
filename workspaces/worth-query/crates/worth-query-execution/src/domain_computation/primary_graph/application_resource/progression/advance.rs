use super::super::{occurrence::ResourceLifecycle, value_capture};
use super::*;

fn finish_previous_provider<Schema, Binding>(
    execution: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<Schema, Binding>,
    row: &mut ResourceRowLease,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    let ResourceLifecycle::Live { owner, .. } = &row.occurrence().lifecycle else {
        return Ok(());
    };
    if row
        .occurrence()
        .attempt
        .as_ref()
        .is_some_and(|attempt| !attempt.matches_native(owner))
    {
        // Native replacement may already have succeeded when provider cleanup
        // or the original request handoff refuses. Keep the old issued attempt
        // until that exact provider has actually completed its cleanup.
        close::close_provider(execution, installed, row)?;
        row.occurrence_mut().attempt = None;
    }
    Ok(())
}

fn ensure_provider<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<Schema, Binding>,
    row: &mut ResourceRowLease,
    effect_authority: Option<
        &crate::domain_computation::application_aftermath::WorthQueryRecoveryInspectAuthority,
    >,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    finish_previous_provider(execution, installed, row)?;
    let ResourceLifecycle::Live { owner, completion } = &row.occurrence().lifecycle else {
        return Ok(());
    };
    if completion.is_some() {
        return Ok(());
    }
    if row
        .occurrence()
        .attempt
        .as_ref()
        .is_some_and(|attempt| attempt.matches_native(owner))
    {
        return if row.occurrence().provider_closed {
            Err(ResourceCause::Storage(
                super::super::storage::ResourceStorageDenial::CleanupPending,
            ))
        } else {
            Ok(())
        };
    }
    if !row.occurrence().provider_closed {
        return Err(ResourceCause::Storage(
            super::super::storage::ResourceStorageDenial::CleanupPending,
        ));
    }
    if let Some(handle) = row.occurrence().recovery.as_ref() {
        super::super::recovery::require_unresolved(
            runtime,
            execution,
            handle,
            effect_authority.ok_or(ResourceCause::RecoveryHandoffRequired)?,
            &row.occurrence().access,
            &row.occurrence().business_product,
        )?;
    }
    let occurrence = row.occurrence_mut();
    let ResourceLifecycle::Live { owner, .. } = &occurrence.lifecycle else {
        unreachable!()
    };
    // Borrow disjoint row fields through the callback. The original input and
    // all provider custody remain in the row even if the provider unwinds.
    value_capture::with_scalar::<Binding::InputBinding, _>(
        execution,
        &occurrence.input.value,
        |input| {
            start_provider_parts(
                execution,
                installed,
                owner,
                occurrence.native_slot.ok_or(ResourceCause::Unavailable)?,
                &mut occurrence.attempt,
                &mut occurrence.provider_closed,
                input,
            )
        },
    )
    .map_err(ResourceCause::Value)?
}

pub(in crate::domain_computation::primary_graph::application_resource) fn advance<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<Schema, Binding>,
    row: &mut ResourceRowLease,
    selected: &crate::basis::WorthQueryProductBranchLease,
    effect_authority: Option<
        &crate::domain_computation::application_aftermath::WorthQueryRecoveryInspectAuthority,
    >,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    if let Some(handle) = row.occurrence().recovery.as_ref() {
        super::super::recovery::require_unresolved(
            runtime,
            execution,
            handle,
            effect_authority.ok_or(ResourceCause::RecoveryHandoffRequired)?,
            &row.occurrence().access,
            &row.occurrence().business_product,
        )?;
    }
    if row.occurrence().result.is_some() {
        close::drain_candidate::<Schema, Binding>(runtime, row)?;
        close::close_provider(execution, installed, row)?;
        close::finish_native::<Schema, Binding>(runtime, row)?;
        return row.finish_native_slot().map_err(ResourceCause::Storage);
    }
    let index = row
        .occurrence()
        .native_slot
        .ok_or(ResourceCause::Unavailable)?;
    let reading = installed.clock.observe().map_err(ResourceCause::Clock)?;
    execution.consult().map_err(ResourceCause::Request)?;
    let clock = runtime
        .product_runtime
        .advance_owned_async_source_clock(
            &runtime.bridge.conditional(),
            &installed.storage.nodes[index].declaration,
            selected,
            reading.nanoseconds,
        )
        .map_err(ResourceCause::Native)?;
    row.occurrence_mut().last_clock = Some(clock);
    execution.consult().map_err(ResourceCause::Request)?;
    let due = match &row.occurrence().lifecycle {
        ResourceLifecycle::Live { owner, .. } => owner
            .request()
            .and_then(|request| {
                request
                    .request()
                    .in_flight_identity()
                    .in_flight()
                    .timeout_due_tick()
            })
            .is_some_and(|due| reading.nanoseconds >= due.get()),
        _ => return Err(ResourceCause::Unavailable),
    };
    if !due && row.occurrence().timeout.is_none() && row.occurrence().retry.is_none() {
        ensure_provider(runtime, execution, installed, row, effect_authority)?;
        match completion::receive(runtime, execution, installed, row, effect_authority)? {
            completion::Received::Committed => {
                close::drain_candidate::<Schema, Binding>(runtime, row)?;
                close::close_provider(execution, installed, row)?;
                close::finish_native::<Schema, Binding>(runtime, row)?;
                row.finish_native_slot().map_err(ResourceCause::Storage)?;
            }
            completion::Received::None | completion::Received::EffectsIndeterminate => {}
        }
        return Ok(());
    }
    // An owned external effect remains with9.17.7's recovery owner. It cannot
    // be redispatched by the ordinary request-response retry family.
    let occurrence = row.occurrence_mut();
    let ResourceLifecycle::Live { owner, .. } = &mut occurrence.lifecycle else {
        return Err(ResourceCause::Unavailable);
    };
    if owner.wake_cleanup_pending() {
        runtime
            .bridge
            .conditional()
            .finish_managed_owned_async_wake_cleanup(owner)
            .map_err(ResourceCause::Managed)?;
    }
    if occurrence.timeout.is_none() && occurrence.retry.is_none() {
        let timeout = runtime
            .bridge
            .conditional()
            .advance_managed_owned_async_request_to_timeout(owner, reading.nanoseconds)
            .map_err(ResourceCause::Managed)?;
        occurrence.timeout = Some(timeout);
    }
    if occurrence.recovery.is_some() {
        return Err(ResourceCause::RecoveryHandoffRequired);
    }
    close::drain_candidate::<Schema, Binding>(runtime, row)?;
    close::close_provider(execution, installed, row)?;
    let occurrence = row.occurrence_mut();
    let ResourceLifecycle::Live { owner, .. } = &mut occurrence.lifecycle else {
        unreachable!()
    };
    if occurrence.retry.is_none() {
        let timeout = occurrence
            .timeout
            .as_ref()
            .ok_or(ResourceCause::Unavailable)?;
        let scheduled = runtime
            .bridge
            .conditional()
            .schedule_managed_owned_async_retry(owner, timeout)
            .map_err(ResourceCause::Managed)?;
        occurrence.retry = Some(scheduled);
    }
    let before = owner.request_handle();
    let result = runtime
        .bridge
        .conditional()
        .advance_managed_owned_async_retry(
            owner,
            occurrence.retry.as_ref().expect("actual issued schedule"),
            reading.nanoseconds,
        );
    // A post-admission Bridge refusal can already own the successor. Keep that
    // true new occurrence, never an apparently live old provider attempt.
    if owner.request_handle() != before {
        occurrence.retry = None;
        occurrence.timeout = None;
        occurrence.attempt = None;
    }
    result.map_err(ResourceCause::Managed)?;
    ensure_provider(runtime, execution, installed, row, None)
}

pub(in crate::domain_computation::primary_graph::application_resource) fn revalidate<
    Schema,
    Binding,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<Schema, Binding>,
    row: &mut ResourceRowLease,
    selected: &crate::basis::WorthQueryProductBranchLease,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    if row.occurrence().recovery.is_some() {
        return Err(ResourceCause::RecoveryHandoffRequired);
    }
    close::drain_candidate::<Schema, Binding>(runtime, row)?;
    finish_previous_provider(execution, installed, row)?;
    let occurrence = row.occurrence_mut();
    let ResourceLifecycle::Live { owner, .. } = &mut occurrence.lifecycle else {
        return Err(ResourceCause::Unavailable);
    };
    let before = owner.request_handle();
    let result = runtime
        .product_runtime
        .revalidate_custodied_owned_async_request(&runtime.bridge.conditional(), owner, selected);
    if owner.request_handle() != before {
        occurrence.retry = None;
        occurrence.timeout = None;
    }
    result.map_err(ResourceCause::Revalidation)?;
    ensure_provider(runtime, execution, installed, row, None)
}
