//! Concrete artifact clone work. No retained-byte reservation is granted here.
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::error::SignalError;
use crate::data::output::PartitionSubscription;
use crate::data::reuse::{
    PersistentCorrespondenceEvidence, ReuseBoundaryContext, ReuseStrategyBoundaryContext,
};
use crate::logic::evaluation::{EvaluationEffect, EvaluationWork};

pub(super) fn string(
    value: Option<&str>,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(value.map_or(Some(1), |s| s.len().checked_add(1)))
}

fn comparator(
    value: &VersionComparatorPolicy,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    match value {
        VersionComparatorPolicy::Custom { key } => string(Some(key), work),
        VersionComparatorPolicy::Exact
        | VersionComparatorPolicy::Tolerance { .. }
        | VersionComparatorPolicy::OutputIdentity
        | VersionComparatorPolicy::Installed { .. } => Ok(()),
    }
}

fn scopes(
    values: &[PartitionSubscription],
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(
        values
            .len()
            .checked_mul(std::mem::size_of::<PartitionSubscription>() + 1)
            .filter(|bytes| *bytes <= isize::MAX as usize),
    )?;
    for value in values {
        string(Some(&value.partition.0), work)?;
        string(value.detail.as_deref(), work)?;
    }
    Ok(())
}

pub(super) fn warm(
    effect: &EvaluationEffect,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(Some(std::mem::size_of::<
        crate::data::trace::RuntimeArtifactState,
    >()))?;
    string(
        effect
            .operational
            .reuse_basis
            .artifact_family_basis
            .as_ref()
            .map(|v| v.as_str()),
        work,
    )?;
    let authority = &effect.operational.reuse_boundary_authority;
    comparator(&authority.tolerance_regime, work)?;
    string(authority.artifact_family.as_ref().map(|v| v.as_str()), work)
}

pub(super) fn cold(
    effect: &EvaluationEffect,
    retain_detail: bool,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    work.reserve(Some(std::mem::size_of::<
        crate::data::trace::ColdArtifactIntent,
    >()))?;
    for label in effect
        .labels()
        .iter()
        .take(crate::data::trace::COLD_ARTIFACT_INTENT_LABEL_LIMIT)
    {
        string(Some(label), work)?;
    }
    if let Some(keyed) = effect.keyed_context() {
        string(keyed.family.as_ref().map(|v| v.as_str()), work)?;
        string(keyed.key.as_ref().map(|v| v.as_str()), work)?;
    }
    if let Some(certification) = effect.reuse_certification() {
        work.reserve(
            certification
                .proofs
                .len()
                .checked_mul(std::mem::size_of::<crate::data::reuse::ReuseBoundaryProof>() + 1)
                .filter(|bytes| *bytes <= isize::MAX as usize),
        )?;
    }
    if retain_detail {
        if let Some(detail) = effect.reuse_boundary_detail() {
            context(detail, work)?;
        }
    }
    Ok(())
}

fn context(
    detail: &ReuseBoundaryContext,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    let ReuseBoundaryContext {
        topology_regime: _,
        tolerance_regime,
        semantic_region,
        authority_policy: _,
        artifact_family,
        structural_dependency_basis: _,
        partition_region_basis,
        strategy_detail,
    } = detail;
    work.reserve(Some(std::mem::size_of::<ReuseBoundaryContext>()))?;
    comparator(tolerance_regime, work)?;
    string(artifact_family.as_ref().map(|v| v.as_str()), work)?;
    scopes(&semantic_region.partition_scope, work)?;
    scopes(partition_region_basis.as_slice(), work)?;
    match strategy_detail {
        ReuseStrategyBoundaryContext::None => Ok(()),
        ReuseStrategyBoundaryContext::PartialArtifactSplice {
            composition_regions,
        } => scopes(composition_regions.as_slice(), work),
        ReuseStrategyBoundaryContext::CrossIdentity {
            persistent_correspondence,
        } => {
            let (PersistentCorrespondenceEvidence::HostSuppliedKey(value)
            | PersistentCorrespondenceEvidence::ContractDeclaredBasis(value)
            | PersistentCorrespondenceEvidence::LineageBackedMapping(value)
            | PersistentCorrespondenceEvidence::RegionIdentityBasis(value)) =
                persistent_correspondence;
            string(Some(value), work)
        }
    }
}
