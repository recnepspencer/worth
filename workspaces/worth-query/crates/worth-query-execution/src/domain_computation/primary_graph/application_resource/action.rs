//! Fresh exact-selector admission precedes every protected row and provider read.
use super::{
    admission, installation::InstalledResourceBinding,
    selector::WorthQueryApplicationResourceReceipt, stop::ResourceCause, storage::ResourceRowLease,
};
use crate::domain_computation::{
    authorization::application_disclosure::WorthQueryPendingApplicationQueryGovernance,
    primary_graph::{WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode},
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_query::ApplicationQueryBinding, application_resource::ApplicationResourceBinding,
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryResourceAccessPurpose;
use worth_relational::facade::identity::EntityId;

pub(super) fn with_existing<Schema, Binding, T, R>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    installed: &Arc<InstalledResourceBinding<Schema, Binding>>,
    selector: &WorthQueryApplicationResourceReceipt<Schema, Binding>,
    purpose: WorthQueryResourceAccessPurpose,
    pending: impl FnOnce(
        EntityId,
        EntityId,
    )
        -> Result<Option<WorthQueryPendingApplicationQueryGovernance>, ResourceCause>,
    take: impl FnOnce(
        &Arc<super::storage::ResourceStorage>,
        super::storage::ResourceRowKey,
    ) -> Result<ResourceRowLease, super::storage::ResourceStorageDenial>,
    action: impl FnOnce(
        &mut ResourceRowLease,
        &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<T, ResourceCause>,
    finish: impl FnOnce(ResourceRowLease, T) -> Result<R, ResourceCause>,
) -> Result<R, ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    let selected = runtime
        .on_branch(selector.access.branch)
        .select_with_request(execution)
        .map_err(ResourceCause::Selection)?;
    let principal = selected
        .resolve_authenticated_principal_issued(
            execution,
            installed.installed.access_binding().principal_binding(),
            external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
            None,
        )
        .map_err(ResourceCause::Principal)?;
    let scope = selected
        .resolve_retained_query_scope_issued(
            execution,
            Binding::Access::scope_field(),
            &selector.access.scope,
            request,
        )
        .map_err(ResourceCause::Scope)?;
    let pending = pending(
        principal.principal().principal_entity_id(),
        scope.scope().entity_id(),
    )?;
    let admitted = admission::prepare(
        runtime,
        execution,
        &installed.installed,
        &installed.prepared[&purpose],
        purpose,
        &selected,
        &principal,
        &scope,
        request,
        Some(&selector.access.scope),
        &selector.access.parameters,
        pending,
    )
    .map_err(ResourceCause::Admission)?;
    // Runtime/slot existence and Busy become observable only after the real
    // action and all requested facets have been independently admitted.
    if !std::sync::Weak::ptr_eq(&selector.binding, &Arc::downgrade(installed)) {
        return Err(ResourceCause::Unavailable);
    }
    let mut row = take(&installed.storage, selector.key).map_err(ResourceCause::Storage)?;
    let result = action(&mut row, selected.product());
    let terminal = admitted.terminal(execution);
    drop(scope);
    drop(principal);
    let (_, product, basis) = selected.into_parts();
    drop(product);
    let terminal = match terminal {
        Ok(terminal) => terminal,
        Err(cause) => return Err(prefer_inner(result.err(), cause)),
    };
    match terminal.complete(execution, basis.release()) {
        Ok(completion) => row.occurrence_mut().last_action = Some(completion),
        Err(cause) => return Err(prefer_inner(result.err(), cause)),
    }
    match result {
        Ok(value) => finish(row, value),
        Err(cause) => Err(cause),
    }
}

pub(super) fn prefer_inner(
    inner: Option<ResourceCause>,
    outer: admission::ResourceAdmissionDenial,
) -> ResourceCause {
    if matches!(
        &outer,
        admission::ResourceAdmissionDenial::Request(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::NestedStopped
        ))
    ) && let Some(inner) = inner
    {
        return inner;
    }
    ResourceCause::Admission(outer)
}
