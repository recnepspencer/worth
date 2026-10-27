//! Actual Native/provider progression writes custody before each fallible handoff.
mod advance;
pub(super) mod close;
mod completion;
use super::{
    installation::InstalledResourceBinding, occurrence::ResourceLifecycle, stop::ResourceCause,
    storage::ResourceRowLease,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
pub(super) use advance::{advance, revalidate};
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_resource::{ApplicationResourceBinding, ApplicationResourceInput},
    application_schema::ApplicationSchema,
};

pub(super) fn start<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<Schema, Binding>,
    row: &mut ResourceRowLease,
    input: &ApplicationResourceInput<Schema, Binding>,
    recovery_authority: Option<
        &crate::domain_computation::application_aftermath::WorthQueryRecoveryInspectAuthority,
    >,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    let index = installed
        .storage
        .take_native()
        .map_err(ResourceCause::Storage)?;
    row.occurrence_mut().native_slot = Some(index);
    let slot = &installed.storage.nodes[index];
    execution.consult().map_err(ResourceCause::Request)?;
    let reading = installed.clock.observe().map_err(ResourceCause::Clock)?;
    execution.consult().map_err(ResourceCause::Request)?;
    let clock = runtime
        .product_runtime
        .advance_owned_async_source_clock(
            &runtime.bridge.conditional(),
            &slot.declaration,
            &row.occurrence().business_product,
            reading.nanoseconds,
        )
        .map_err(ResourceCause::Native)?;
    row.occurrence_mut().lifecycle = ResourceLifecycle::Reserved { clock: Some(clock) };
    execution.consult().map_err(ResourceCause::Request)?;
    let owner = runtime
        .product_runtime
        .admit_managed_owned_async_request(
            &runtime.bridge.conditional(),
            &slot.declaration,
            &row.occurrence().business_product,
        )
        .map_err(ResourceCause::Native)?;
    let refused = owner.refusal();
    let clock = std::mem::replace(
        &mut row.occurrence_mut().lifecycle,
        ResourceLifecycle::Live {
            owner,
            completion: None,
        },
    );
    if let ResourceLifecycle::Reserved { clock } = clock {
        row.occurrence_mut().last_clock = clock;
    }
    if let Some(cause) = refused {
        return Err(ResourceCause::Managed(cause));
    }
    if let Some(handle) = row.occurrence().recovery.as_ref() {
        super::recovery::require_unresolved(
            runtime,
            execution,
            handle,
            recovery_authority.ok_or(ResourceCause::RecoveryHandoffRequired)?,
            &row.occurrence().access,
            &row.occurrence().business_product,
        )?;
    }
    start_provider(execution, installed, row, input)
}

fn start_provider<Schema, Binding>(
    execution: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<Schema, Binding>,
    row: &mut ResourceRowLease,
    input: &ApplicationResourceInput<Schema, Binding>,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    let occurrence = row.occurrence_mut();
    let ResourceLifecycle::Live { owner, .. } = &occurrence.lifecycle else {
        return Err(ResourceCause::Unavailable);
    };
    start_provider_parts(
        execution,
        installed,
        owner,
        occurrence.native_slot.ok_or(ResourceCause::Unavailable)?,
        &mut occurrence.attempt,
        &mut occurrence.provider_closed,
        input,
    )
}

fn start_provider_parts<Schema, Binding>(
    execution: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<Schema, Binding>,
    owner: &worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRequestAdmission,
    index: usize,
    retained_attempt: &mut Option<super::provider::WorthQueryResourceProviderAttempt>,
    provider_closed: &mut bool,
    input: &ApplicationResourceInput<Schema, Binding>,
) -> Result<(), ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    let slot = &installed.storage.nodes[index];
    let attempt = super::provider::WorthQueryResourceProviderAttempt::issue(execution, owner)
        .map_err(ResourceCause::Attempt)?;
    let writer = slot
        .ingress
        .open(&attempt)
        .map_err(ResourceCause::Ingress)?;
    // Before the provider is contacted, the installed row already owns the
    // exact attempt and its cleanup path, including a refused provider start.
    *retained_attempt = Some(attempt.clone());
    *provider_closed = false;
    execution.consult().map_err(ResourceCause::Request)?;
    installed
        .provider
        .start(&attempt, input, writer)
        .map_err(ResourceCause::Provider)?;
    execution.consult().map_err(ResourceCause::Request)
}
