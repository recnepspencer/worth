//! Resource monitoring never replaces9.17.7's live effect/recovery owners.
use super::{
    occurrence::OriginalResourceAccess,
    stop::{ResourceCause, ResourceRecoveryAffinity as Affinity},
};
use crate::domain_computation::{
    application_aftermath::{WorthQueryRecoveryHandle, WorthQueryRecoveryInspectAuthority},
    authorization::WorthQueryOperationScopeEntityBinding,
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) fn require_authority<Schema: ApplicationSchema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    handle: &WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryInspectAuthority,
    access: &OriginalResourceAccess,
    product: &crate::basis::WorthQueryProductBranchLease,
) -> Result<(), ResourceCause> {
    authority
        .ensure_for(handle)
        .map_err(ResourceCause::Recovery)?;
    if handle.runtime_authority() != runtime.runtime.authority_identity() {
        return Err(ResourceCause::RecoveryAffinity(Affinity::Runtime));
    }
    if handle.binding().principal_scope().binding_identity()
        != &runtime.installed_schema.binding_identity()
    {
        return Err(ResourceCause::RecoveryAffinity(Affinity::Schema));
    }
    if handle.binding().branch() != product.relational_basis().identity().branch_id() {
        return Err(ResourceCause::RecoveryAffinity(Affinity::Branch));
    }
    if handle.binding().principal_scope().scope()
        != WorthQueryOperationScopeEntityBinding::from_entity(access.scope.subject())
    {
        return Err(ResourceCause::RecoveryAffinity(Affinity::Scope));
    }
    Ok(())
}

pub(super) fn require_unresolved<Schema: ApplicationSchema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    handle: &WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryInspectAuthority,
    access: &OriginalResourceAccess,
    product: &crate::basis::WorthQueryProductBranchLease,
) -> Result<(), ResourceCause> {
    require_authority(runtime, handle, authority, access, product)?;
    let record = handle
        .binding()
        .dispatch_outbox()
        .ok_or(ResourceCause::RecoveryAffinity(Affinity::OutboxMissing))?;
    // This is the actual current terminal/outcome owner, before consulting a
    // historical outbox row that may already have been lawfully released.
    runtime
        .refuse_completed_external_effect(record)
        .map_err(ResourceCause::EffectTerminal)?;
    // The retained exact-commit read proves original record/correlation/commit
    // provenance only. It does not establish current effect status.
    let original = runtime
        .primary_provider
        .committed_dispatch_outbox_for_binding(execution, handle.binding())
        .map_err(ResourceCause::EffectOutbox)?;
    drop(original);
    Ok(())
}
