use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentityBinding;
use worth_query_installation::facade::{
    ApplicationIdentityScalarValueBinding, ApplicationScalarValueBinding, ApplicationSchema,
    ApplicationSchemaBindingIdentity, WorthQueryInstalledPrincipalBinding,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionRuntime, WorthQueryRuntimeAuthorityIdentity,
};

use super::authenticated_principal::WorthQueryResolvedPrincipalEvidence;
use super::freshness::{validate_freshness_at_snapshot, WorthQueryPrincipalFreshnessEvidence};
use super::observations::{observe_mapping, resolve_principal_target};
use super::resolution_denial::{
    entity_lookup_resolution_denial, principal_binding_resolution_denial, resolution_denial,
};
use super::root::WorthQueryPrimaryGraph;
use super::schema_layout::WorthQueryPrimaryPrincipalBindingLayout;
use super::{
    WorthQueryAuthenticatedPrincipal, WorthQueryPrincipalResolutionDenial,
    WorthQueryPrincipalResolutionDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrincipalResolutionMode {
    Ordinary,
    Certification,
}

struct WorthQueryPrincipalSnapshotResolution<'a> {
    binding: &'a str,
    layout: &'a WorthQueryPrimaryPrincipalBindingLayout,
    expected_identity: &'a AspectValue,
    mode: WorthQueryPrincipalResolutionMode,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
}

#[path = "resolution/selected_product.rs"]
mod selected_product;

fn principal_graph_binding<'a>(
    runtime: &'a WorthQueryExecutionRuntime,
    binding: &str,
) -> Result<
    (
        &'a WorthQueryPrimaryGraph,
        WorthQueryPrimaryPrincipalBindingLayout,
    ),
    WorthQueryPrincipalResolutionDenial,
> {
    let graph = runtime.primary_graph().ok_or_else(|| {
        resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::PrimaryGraphNotInstalled,
            binding,
        )
    })?;
    let layout = graph
        .layout
        .principal_binding(binding)
        .cloned()
        .ok_or_else(|| {
            resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::BindingNotInstalled,
                binding,
            )
        })?;
    Ok((graph, layout))
}

fn resolve_at_snapshot<
    Schema,
    Binding,
    Mapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
>(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    resolution: &WorthQueryPrincipalSnapshotResolution<'_>,
    installed_binding: &WorthQueryInstalledPrincipalBinding<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
) -> Result<
    WorthQueryResolvedPrincipalEvidence<PrincipalIdentity>,
    WorthQueryPrincipalResolutionDenial,
>
where
    PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
    PrincipalIdentity: 'static,
{
    let (mapping_id, examined_candidate_count) =
        resolve_unique_mapping_candidate(runtime, snapshot, resolution)?;
    let mapping = observe_mapping(
        runtime,
        snapshot,
        mapping_id,
        resolution.layout,
        resolution.binding,
    )?;
    validate_resolved_mapping(&mapping, resolution.expected_identity, resolution.binding)?;
    let target = resolve_principal_target(
        runtime,
        snapshot,
        mapping_id,
        resolution.layout,
        resolution.binding,
    )?;
    let freshness = WorthQueryPrincipalFreshnessEvidence::new(mapping.clone(), target.clone());
    let principal_identity = installed_binding
        .decode_principal_identity(&target.principal_identity)
        .map_err(|_| {
            resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof,
                resolution.binding,
            )
        })?;
    Ok(WorthQueryResolvedPrincipalEvidence {
        principal_entity_id: target.target,
        principal_identity,
        runtime_authority: resolution.runtime_authority,
        binding_identity: resolution.binding_identity.clone(),
        binding: resolution.binding.to_string(),
        mapping_entity_id: mapping.entity_id,
        target_relation_id: target.relation_id,
        freshness,
        examined_candidate_count,
    })
}

fn resolve_unique_mapping_candidate(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    resolution: &WorthQueryPrincipalSnapshotResolution<'_>,
) -> Result<(EntityId, usize), WorthQueryPrincipalResolutionDenial> {
    let request = BoundedEntityFieldLookupRequest::new(
        snapshot.clone(),
        resolution.layout.index_id,
        resolution.layout.mapping_kind,
        resolution.layout.identity_locator.clone(),
        resolution.expected_identity.clone(),
        2,
    )
    .map_err(|denial| entity_lookup_resolution_denial(denial.kind(), resolution.binding))?;
    let parity_mode = match resolution.mode {
        WorthQueryPrincipalResolutionMode::Ordinary => BoundedIndexParityMode::Production,
        WorthQueryPrincipalResolutionMode::Certification => BoundedIndexParityMode::Certification,
    };
    let lookup = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, parity_mode)
        .map_err(|denial| entity_lookup_resolution_denial(denial.kind(), resolution.binding))?;
    if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
        return Err(resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::AmbiguousPrincipal,
            resolution.binding,
        ));
    }
    let mapping_id = *lookup.candidate_entity_ids().first().ok_or_else(|| {
        resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::UnknownPrincipal,
            resolution.binding,
        )
    })?;
    Ok((mapping_id, lookup.examined_entry_count()))
}

fn validate_resolved_mapping(
    mapping: &super::observations::WorthQueryPrincipalMappingObservation,
    expected_identity: &AspectValue,
    binding: &str,
) -> Result<(), WorthQueryPrincipalResolutionDenial> {
    if &mapping.identity != expected_identity {
        return Err(resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::CorruptIdentityIndex,
            binding,
        ));
    }
    if !mapping.enabled {
        return Err(resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::DisabledPrincipal,
            binding,
        ));
    }
    Ok(())
}

fn validate_current_schema_binding<Schema>(
    runtime: &WorthQueryExecutionRuntime,
    graph_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    proof_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    binding: &str,
) -> Result<(), WorthQueryPrincipalResolutionDenial>
where
    Schema: ApplicationSchema,
{
    let declaration = Schema::declaration().map_err(|_| {
        resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::StaleInstalledSchema,
            binding,
        )
    })?;
    let current = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .map_err(|_| {
            resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::StaleInstalledSchema,
                binding,
            )
        })?;
    if current.binding_identity() == *graph_identity
        && current.binding_identity() == *proof_identity
    {
        Ok(())
    } else {
        Err(resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::StaleInstalledSchema,
            binding,
        ))
    }
}

fn admit_resolution_request(
    scope: &WorthQueryRequestScope,
    binding: &str,
    authentication_expired: bool,
) -> Result<(), WorthQueryPrincipalResolutionDenial> {
    match scope.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => {
            return Err(resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::Cancelled,
                binding,
            ));
        }
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
            return Err(resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::DeadlineExceeded,
                binding,
            ));
        }
        None => {}
    }
    if authentication_expired {
        return Err(resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::ExpiredAuthentication,
            binding,
        ));
    }
    Ok(())
}
