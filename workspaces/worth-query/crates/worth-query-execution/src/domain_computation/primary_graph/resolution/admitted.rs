//! Fresh installed principal decode from one bounded selected native image.

use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
use worth_query_installation::facade::ApplicationIdentityScalarValueBinding;
use worth_query_installation::facade::WorthQueryInstalledPrincipalBinding;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::runtime::VisibilityProjectionView;

use super::super::authenticated_principal::WorthQueryResolvedPrincipalEvidence;
use super::super::freshness::WorthQueryPrincipalFreshnessEvidence;

use super::super::WorthQueryPrincipalResolutionDenialKind;
use super::admitted_lookup::AdmittedPrincipalLookupStop;
use super::admitted_observations::AdmittedPrincipalObservationStop;
use super::WorthQueryPrincipalSnapshotResolution;

#[derive(Debug)]
pub(super) enum AdmittedPrincipalResolutionStop {
    Index(worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind),
    Semantic(WorthQueryPrincipalResolutionDenialKind),
}

impl From<AdmittedPrincipalLookupStop> for AdmittedPrincipalResolutionStop {
    fn from(stop: AdmittedPrincipalLookupStop) -> Self {
        match stop {
            AdmittedPrincipalLookupStop::Index(kind) => Self::Index(kind),
            AdmittedPrincipalLookupStop::Semantic(kind) => Self::Semantic(kind),
        }
    }
}

impl From<AdmittedPrincipalObservationStop> for AdmittedPrincipalResolutionStop {
    fn from(stop: AdmittedPrincipalObservationStop) -> Self {
        match stop {
            AdmittedPrincipalObservationStop::Semantic(kind) => Self::Semantic(kind),
        }
    }
}

pub(super) fn resolve_at_snapshot<
    Schema,
    Binding,
    Mapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    view: &VisibilityProjectionView,
    resolution: &WorthQueryPrincipalSnapshotResolution<'_>,
    installed_binding: &WorthQueryInstalledPrincipalBinding<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
) -> Result<WorthQueryResolvedPrincipalEvidence<PrincipalIdentity>, AdmittedPrincipalResolutionStop>
where
    PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
    PrincipalIdentity: 'static,
{
    use AdmittedPrincipalResolutionStop as Stop;
    let checked = super::admitted_lookup::resolve_unique_mapping_candidate(
        runtime, execution, view, resolution,
    )?;
    let mapping = super::admitted_observations::observe_mapping(execution, &checked)?;
    if !mapping.enabled {
        return Err(Stop::Semantic(
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::DisabledPrincipal),
        ));
    }
    let target = super::admitted_observations::observe_target(
        execution,
        view,
        mapping.entity_id,
        resolution.layout,
    )?;

    let principal_identity = installed_binding
        .decode_principal_identity(&target.principal_identity)
        .map_err(|_| {
            Stop::Semantic(WorthQueryPrincipalResolutionDenialKind::Refused(
                PrincipalRefusal::StalePrincipalProof,
            ))
        })?;

    let principal_entity_id = target.target;
    let target_relation_id = target.relation_id;
    let mapping_entity_id = mapping.entity_id;
    let freshness = WorthQueryPrincipalFreshnessEvidence::new(mapping, target);
    Ok(WorthQueryResolvedPrincipalEvidence {
        principal_entity_id,
        principal_identity,
        runtime_authority: resolution.runtime_authority,
        binding_identity: resolution.binding_identity.clone(),
        binding: resolution.binding.to_owned(),
        mapping_entity_id,
        target_relation_id,
        freshness,
        examined_candidate_count: checked.examined_count(),
    })
}
