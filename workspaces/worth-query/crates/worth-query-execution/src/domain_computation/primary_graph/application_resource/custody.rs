//! Retain original work before effects; only live owners record release.
pub(super) mod handoff;
use super::super::{query_obligations::*, WorthQueryPrimaryGraphApplicationRuntime};
use super::{
    installation::InstalledResourceBinding, occurrence::ResourceLifecycle, stop::ResourceCause,
    storage::ResourceRowLease,
};
pub(super) use handoff::HomeResourceCustody;
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_resource::ApplicationResourceBinding, application_schema::ApplicationSchema,
};

pub(super) fn retain<S: ApplicationSchema, B: ApplicationResourceBinding<S>>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    request: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<S, B>,
    row: &mut ResourceRowLease,
) -> Result<(), ResourceCause> {
    if row.occurrence().disposal
        != super::super::query_obligations::history::ResourceDisposal::Retained
    {
        return Ok(());
    }
    let Some(support) = runtime.installed_program_support() else {
        return Ok(());
    };
    let history = support.lifecycle();
    history
        .settle_resource_publication(request)
        .map_err(ResourceCause::Obligation)?;
    if row
        .occurrence()
        .obligation
        .is_some_and(|identity| history.resource_retained(identity))
    {
        row.occurrence()
            .access
            .obligation
            .get_or_init(|| row.occurrence().obligation.expect("acknowledged resource"));
        return Ok(());
    }
    let occurrence = row.occurrence();
    let access = &occurrence.access;
    let observation = occurrence.business_product.observation();
    let binding = installed.installed.binding_identity();
    let original = QueryResourceOriginal {
        binding: B::IDENTITY.to_owned(),
        package: *binding.package_identity().bytes(),
        schema: *binding.schema_identity().bytes(),
        branch: observation.branch_identity().name().as_str().to_owned(),
        commit: observation.selected_commit().ordinal(),
        principal: access.birth_principal,
        scope_subject: access.scope.subject(),
        scope_aspect: access
            .scope
            .locator()
            .aspect()
            .aspect_key()
            .as_str()
            .to_owned(),
        scope_fields: access
            .scope
            .locator()
            .field_path()
            .fields()
            .iter()
            .map(|field| field.as_str().to_owned())
            .collect(),
        scope_value: access.scope.value().clone(),
        query: *access.parameters.query().as_bytes(),
        parameters: access
            .parameters
            .parameters()
            .bindings()
            .iter()
            .map(|(key, value)| (key.to_string(), value.clone()))
            .collect(),
        input: occurrence.input.value.clone(),
        declaration: installed.storage.nodes[occurrence.native_slot.unwrap_or(0)]
            .source
            .clone(),
    };
    if let Some((identity, pending)) = history
        .retain_resource(request, original)
        .map_err(ResourceCause::Obligation)?
    {
        row.occurrence_mut().obligation = Some(identity);
        if let Some(cause) = pending {
            return Err(ResourceCause::Obligation(cause));
        }
        row.occurrence().access.obligation.get_or_init(|| identity);
    }
    Ok(())
}
pub(super) fn record_request<S: ApplicationSchema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    request: &ExecutionRequest<'_, '_>,
    row: &mut ResourceRowLease,
) -> Result<(), ResourceCause> {
    let Some(identity) = row.occurrence().obligation else {
        return Ok(());
    };
    let ResourceLifecycle::Live { owner, .. } = &row.occurrence().lifecycle else {
        return Ok(());
    };
    let handle = owner.request_handle();
    let handoff = row
        .occurrence_mut()
        .handoff
        .get_or_insert_with(QueryResourceHandoff::default);
    handoff.original_request.get_or_insert(handle);
    handoff.request = Some(handle);
    runtime
        .installed_program_support()
        .ok_or(ResourceCause::Unavailable)?
        .lifecycle()
        .record_resource_handoff(request, identity, handoff.clone())
        .map_err(ResourceCause::Obligation)
}
/// Ordinary explicit Close discharges only after its actual terminal/release.
pub(super) fn discharge<S: ApplicationSchema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    request: &ExecutionRequest<'_, '_>,
    row: &mut ResourceRowLease,
) -> Result<(), ResourceCause> {
    if let Some(identity) = row.occurrence().obligation {
        let history = runtime
            .installed_program_support()
            .ok_or(ResourceCause::Unavailable)?
            .lifecycle();
        if row.occurrence().disposal
            == super::super::query_obligations::history::ResourceDisposal::Retained
        {
            history
                .settle_resource_publication(request)
                .map_err(ResourceCause::Obligation)?;
            if !history
                .resource_obligation(identity)
                .is_some_and(|record| record.handoff.release.is_some())
            {
                // Explicit Close disposes payloads. A refused Home snapshot
                // must not make that disposal depend on its large ingress.
                if let Some(handoff) = row.occurrence_mut().handoff.as_mut() {
                    handoff.ingress.clear();
                    handoff.result = None;
                }
                handoff::record_release(runtime, request, row)?;
            }
        }
        history
            .finish_live_resource_disposal(request, identity, &mut row.occurrence_mut().disposal)
            .map_err(ResourceCause::Obligation)?;
    }
    Ok(())
}
