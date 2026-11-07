#[cfg(test)]
use super::super::active_counters::ActiveSubscriptionCounters;
#[cfg(test)]
use super::super::active_digest::ActiveSubscriptionLaneDigest;
#[cfg(test)]
use super::super::continuation_error::{
    SubscriptionContinuationDenialKind, SubscriptionContinuationError,
};
#[cfg(test)]
use super::super::delivery_dimensions::ContinuationRemapWidth;
#[cfg(test)]
use super::super::evidence_identities::lifecycle_continuation_ordinary_checkpoint_identity;
#[cfg(test)]
use super::super::future_selection::QuerySubscriptionFutureSelection;
#[cfg(test)]
use super::class::SubscriptionContinuationClass;
#[cfg(test)]
use super::evidence::SubscriptionContinuationEvidence;
#[cfg(test)]
use crate::evidence_identity::WorthQueryEvidenceIdentity;

#[cfg(test)]
pub fn admit_subscription_continuation_evidence(
    active_lane_digest: ActiveSubscriptionLaneDigest,
    continuation_class: SubscriptionContinuationClass,
    source_identity: WorthQueryEvidenceIdentity,
    target_identity: WorthQueryEvidenceIdentity,
    basis_identity: WorthQueryEvidenceIdentity,
    authority_identity: WorthQueryEvidenceIdentity,
    remap_width: ContinuationRemapWidth,
) -> Result<SubscriptionContinuationEvidence, SubscriptionContinuationError> {
    let source_for_denial = active_lane_digest.evidence_identity().clone();
    let checkpoint_identity =
        lifecycle_continuation_ordinary_checkpoint_identity(active_lane_digest.evidence_identity());
    admit_subscription_continuation_evidence_with_active_identity(
        active_lane_digest,
        continuation_class,
        source_identity,
        target_identity,
        QuerySubscriptionFutureSelection::ordinary().map_err(|cause| {
            SubscriptionContinuationError::new(
                SubscriptionContinuationDenialKind::EvidenceIdentityDerivationDenied(cause),
                "canonical digest refused continuation evidence",
                source_for_denial.clone(),
                ActiveSubscriptionCounters::default(),
            )
        })?,
        basis_identity,
        checkpoint_identity,
        authority_identity,
        remap_width,
    )
}

#[cfg(test)]
pub fn admit_subscription_continuation_evidence_with_active_identity(
    active_lane_digest: ActiveSubscriptionLaneDigest,
    continuation_class: SubscriptionContinuationClass,
    source_identity: WorthQueryEvidenceIdentity,
    target_identity: WorthQueryEvidenceIdentity,
    future_selection: QuerySubscriptionFutureSelection,
    basis_identity: WorthQueryEvidenceIdentity,
    checkpoint_identity: WorthQueryEvidenceIdentity,
    authority_identity: WorthQueryEvidenceIdentity,
    remap_width: ContinuationRemapWidth,
) -> Result<SubscriptionContinuationEvidence, SubscriptionContinuationError> {
    let mut counters = ActiveSubscriptionCounters::default();
    if matches!(
        continuation_class,
        SubscriptionContinuationClass::UnsupportedContinuation
            | SubscriptionContinuationClass::PreviewPromotionRemap
    ) {
        counters.continuation_remap_denial_count = 1;
        return Err(SubscriptionContinuationError::new(
            SubscriptionContinuationDenialKind::UnsupportedContinuationClass,
            "unsupported or later-phase continuation class cannot produce active subscription evidence",
            active_lane_digest.evidence_identity().clone(),
            counters,
        ));
    }
    if remap_width.get() == 0 {
        counters.continuation_remap_denial_count = 1;
        return Err(SubscriptionContinuationError::new(
            SubscriptionContinuationDenialKind::ContinuationRemapBudgetExceeded,
            "continuation remap evidence requires an explicit nonzero remap width",
            active_lane_digest.evidence_identity().clone(),
            counters,
        ));
    }

    let source_for_denial = active_lane_digest.evidence_identity().clone();
    Ok(SubscriptionContinuationEvidence::new(
        active_lane_digest,
        continuation_class,
        source_identity,
        target_identity,
        future_selection,
        basis_identity,
        checkpoint_identity,
        authority_identity,
        remap_width,
    )
    .map_err(|cause| {
        SubscriptionContinuationError::new(
            SubscriptionContinuationDenialKind::EvidenceIdentityDerivationDenied(cause),
            "canonical digest refused continuation evidence",
            source_for_denial.clone(),
            ActiveSubscriptionCounters::default(),
        )
    })?)
}
