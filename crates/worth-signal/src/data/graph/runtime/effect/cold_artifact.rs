//! Retention-policy-selected cold artifact preparation.
use crate::data::error::SignalError;
use crate::data::output::{CanonicalChangedRegions, ChangedRegion};
use crate::data::trace::{ColdArtifactIntent, COLD_ARTIFACT_INTENT_LABEL_LIMIT};
use crate::diagnostics::policy::ArtifactRetentionPolicy;
use crate::logic::evaluation::{EvaluationEffect, EvaluationWork};
use smallvec::SmallVec;

pub(super) fn build_cold_artifact_intent(
    effect: &EvaluationEffect,
    retention: &crate::diagnostics::policy::RetentionBudget,
    scopes: &crate::data::proof::PartitionScopeSet,
    work: &mut EvaluationWork<'_>,
) -> Result<Option<ColdArtifactIntent>, SignalError> {
    if matches!(
        retention.explanation_retention,
        ArtifactRetentionPolicy::Omit
    ) && matches!(
        retention.provenance_retention,
        ArtifactRetentionPolicy::Omit
    ) {
        return Ok(None);
    }
    let retain_reuse_boundary_detail = matches!(
        effect.operational.reuse_basis.strategy,
        Some(crate::data::reuse::ReuseStrategy::CrossIdentityPersistentMatch)
            | Some(crate::data::reuse::ReuseStrategy::PartialArtifactSplicing)
    );
    super::artifact_payload_work::cold(effect, retain_reuse_boundary_detail, work)?;
    let labels = if matches!(
        retention.explanation_retention,
        ArtifactRetentionPolicy::Retain
    ) || matches!(
        retention.provenance_retention,
        ArtifactRetentionPolicy::Retain
    ) {
        effect
            .labels()
            .iter()
            .take(COLD_ARTIFACT_INTENT_LABEL_LIMIT)
            .cloned()
            .collect()
    } else {
        SmallVec::new()
    };
    let intent = ColdArtifactIntent {
        changed_regions: copy_canonical_regions(scopes, work)?,
        labels,
        keyed_family: effect.keyed_context().and_then(|keyed| {
            keyed
                .family
                .as_ref()
                .map(|family| family.as_str().to_owned())
        }),
        keyed_key: effect
            .keyed_context()
            .and_then(|keyed| keyed.key.as_ref().map(|key| key.as_str().to_owned())),
        reuse_certification: effect.reuse_certification().cloned(),
        reuse_boundary_context: retain_reuse_boundary_detail
            .then(|| effect.reuse_boundary_detail().cloned())
            .flatten(),
    };
    Ok((!intent.is_empty()).then_some(intent))
}

fn copy_canonical_regions(
    scopes: &crate::data::proof::PartitionScopeSet,
    work: &mut EvaluationWork<'_>,
) -> Result<CanonicalChangedRegions, SignalError> {
    work.reserve(
        scopes
            .len()
            .checked_mul(std::mem::size_of::<ChangedRegion>() + 1)
            .filter(|bytes| *bytes <= isize::MAX as usize),
    )?;
    let mut regions = Vec::with_capacity(scopes.len());
    for scope in scopes.as_slice() {
        // One owned copy and the checked canonical-order comparison below.
        work.reserve(
            scope
                .partition
                .0
                .len()
                .checked_add(scope.detail.as_ref().map_or(0, String::len))
                .and_then(|bytes| bytes.checked_mul(3))
                .and_then(|n| n.checked_add(8)),
        )?;
        regions.push(ChangedRegion {
            partition: scope.partition.clone(),
            detail: scope.detail.clone(),
        });
    }
    CanonicalChangedRegions::from_canonical_regions(regions)
        .ok_or_else(|| SignalError::internal("artifact scopes do not yield canonical regions"))
}
