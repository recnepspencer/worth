//! Fresh installed principal decode from one bounded selected native image.

use worth_query_installation::facade::{
    ApplicationIdentityScalarValueBinding, WorthQueryInstalledPrincipalBinding,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::{RelationalRuntime, VisibilityProjectionView};

use super::super::authenticated_principal::WorthQueryResolvedPrincipalEvidence;
use super::super::freshness::WorthQueryPrincipalFreshnessEvidence;
use super::super::output_lineage::invalidation::InvalidationEditAdmission;
use super::super::WorthQueryPrincipalResolutionDenialKind;
use super::admitted_lookup::AdmittedPrincipalLookupStop;
use super::admitted_observations::AdmittedPrincipalObservationStop;
use super::WorthQueryPrincipalSnapshotResolution;

#[derive(Debug)]
pub(super) enum AdmittedPrincipalResolutionStop {
    Admission(CompanionPreflightStop),
    Index(worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind),
    Semantic(WorthQueryPrincipalResolutionDenialKind),
}

impl From<AdmittedPrincipalLookupStop> for AdmittedPrincipalResolutionStop {
    fn from(stop: AdmittedPrincipalLookupStop) -> Self {
        match stop {
            AdmittedPrincipalLookupStop::Admission(stop) => Self::Admission(stop),
            AdmittedPrincipalLookupStop::Index(kind) => Self::Index(kind),
            AdmittedPrincipalLookupStop::Semantic(kind) => Self::Semantic(kind),
        }
    }
}

impl From<AdmittedPrincipalObservationStop> for AdmittedPrincipalResolutionStop {
    fn from(stop: AdmittedPrincipalObservationStop) -> Self {
        match stop {
            AdmittedPrincipalObservationStop::Admission(stop) => Self::Admission(stop),
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
    runtime: &RelationalRuntime,
    view: &VisibilityProjectionView<'_>,
    resolution: &WorthQueryPrincipalSnapshotResolution<'_>,
    installed_binding: &WorthQueryInstalledPrincipalBinding<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
    admission: &mut InvalidationEditAdmission,
) -> Result<WorthQueryResolvedPrincipalEvidence<PrincipalIdentity>, AdmittedPrincipalResolutionStop>
where
    PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
    PrincipalIdentity: 'static,
{
    use AdmittedPrincipalResolutionStop as Stop;
    let checked = super::admitted_lookup::resolve_unique_mapping_candidate(
        runtime, view, resolution, admission,
    )?;
    let mapping = super::admitted_observations::observe_mapping(&checked, admission)?;
    if !mapping.enabled {
        return Err(Stop::Semantic(
            WorthQueryPrincipalResolutionDenialKind::DisabledPrincipal,
        ));
    }
    let target = super::admitted_observations::observe_target(
        view,
        mapping.entity_id,
        resolution.layout,
        admission,
    )?;
    admission.charge_external_work(1).map_err(Stop::Admission)?;
    let principal_identity = installed_binding
        .decode_principal_identity(&target.principal_identity)
        .map_err(|_| {
            Stop::Semantic(WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof)
        })?;
    let text_bytes = u64::try_from(resolution.binding.len())
        .map_err(|_| Stop::Admission(CompanionPreflightStop::WorkCounterOverflow))?;
    admission
        .admit_read_scratch(text_bytes)
        .and_then(|()| admission.charge_external_work(text_bytes.saturating_add(1)))
        .map_err(Stop::Admission)?;
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
