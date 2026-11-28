use std::sync::Arc;

use super::localization::LocalizedFailureParts;
use super::{
    BridgeFailureEvidenceAttachment, BridgeTemporalAsyncFailureClass,
    BridgeTemporalAsyncFailureLocalizationRejection, BridgeTemporalAsyncFailureSubcode,
};
use crate::policy::{BridgePolicyRejection, BridgePolicyRejectionKind};
use crate::subscription::{
    BridgeDeniedMixedCause, BridgeHistoricalTemporalReplayRejection,
    BridgeHistoricalTemporalReplayRejectionKind, BridgeMixedCauseDeniedKind,
    BridgeSubscriptionPreviewLifecycleDiscardRejection,
    BridgeSubscriptionPreviewLifecyclePromotionRejection,
    BridgeSubscriptionPreviewLifecyclePromotionRejectionKind,
    BridgeSubscriptionResumeBasisRejection, BridgeSubscriptionResumeBasisRejectionKind,
    BridgeSuppressedMixedCause, BridgeTemporalWakeRoutingRejection,
    BridgeTemporalWakeRoutingRejectionKind,
};
use crate::temporal::BridgeTemporalBasisDenial;

fn synthetic_attachment(
    family: &str,
    identity: impl Into<String>,
    detail: impl Into<String>,
) -> BridgeFailureEvidenceAttachment {
    let identity = identity.into();
    let detail = detail.into();
    BridgeFailureEvidenceAttachment::synthetic(family, identity, detail)
}

pub(super) fn localize_temporal_basis_denial(
    denial: BridgeTemporalBasisDenial,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let (subcode, detail) = match denial {
        BridgeTemporalBasisDenial::MissingWakeEvidence => (
            BridgeTemporalAsyncFailureSubcode::TemporalBasisMissing,
            "missing wake evidence".to_owned(),
        ),
        BridgeTemporalBasisDenial::WakeTickRegressed {
            signal_clock_tick,
            wake_tick,
        } => (
            BridgeTemporalAsyncFailureSubcode::TemporalBasisStale,
            format!("wake tick {wake_tick} regressed behind signal clock tick {signal_clock_tick}"),
        ),
        BridgeTemporalBasisDenial::BranchMismatch {
            truth_branch_identity,
            signal_branch_identity,
        } => (
            BridgeTemporalAsyncFailureSubcode::TemporalBasisCrossBranch,
            format!(
                "truth branch `{}` mismatched signal branch `{}`",
                truth_branch_identity.canonical_label(),
                signal_branch_identity.canonical_label()
            ),
        ),
        BridgeTemporalBasisDenial::TruthBasisDenied(denial) => (
            BridgeTemporalAsyncFailureSubcode::TemporalBasisMissing,
            format!("{denial:?}"),
        ),
        BridgeTemporalBasisDenial::SignalBasisDenied(denial) => (
            BridgeTemporalAsyncFailureSubcode::TemporalBasisIncompatible,
            format!("{denial:?}"),
        ),
    };
    Ok((
        BridgeTemporalAsyncFailureClass::TemporalBasisFailure,
        subcode,
        vec![synthetic_attachment(
            "temporal.basis_denial",
            subcode.as_str(),
            &detail,
        )],
        Arc::from(detail),
    ))
}

pub(super) fn localize_historical_temporal_replay_rejection(
    rejection: BridgeHistoricalTemporalReplayRejection,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match rejection.rejection_kind() {
        BridgeHistoricalTemporalReplayRejectionKind::MissingPreviousValueEvidence => {
            BridgeTemporalAsyncFailureSubcode::TemporalReadinessPreviousValueMissing
        }
        BridgeHistoricalTemporalReplayRejectionKind::PreviousValueEvidenceBranchMismatch
        | BridgeHistoricalTemporalReplayRejectionKind::PreviousValueEvidenceSnapshotMismatch => {
            BridgeTemporalAsyncFailureSubcode::TemporalBasisIncompatible
        }
        BridgeHistoricalTemporalReplayRejectionKind::TemporalTruthBasisNotHistorical
        | BridgeHistoricalTemporalReplayRejectionKind::TemporalAdmissionFamilyNotHistoricalReplay
        | BridgeHistoricalTemporalReplayRejectionKind::TemporalBasisIdentityMismatch
        | BridgeHistoricalTemporalReplayRejectionKind::HistoricalTruthSnapshotIdentityMismatch
        | BridgeHistoricalTemporalReplayRejectionKind::HistoricalTruthBranchIdentityMismatch => {
            BridgeTemporalAsyncFailureSubcode::TemporalReadinessNotReady
        }
    };
    Ok((
        BridgeTemporalAsyncFailureClass::TemporalReadinessFailure,
        subcode,
        vec![BridgeFailureEvidenceAttachment::reference(
            "subscription.historical_temporal_replay_rejection",
            rejection.temporal_admission_identity(),
            rejection.digest(),
        )
        .with_detail(rejection.rejection_kind().as_str())],
        Arc::from(rejection.rejection_kind().as_str()),
    ))
}

pub(super) fn localize_temporal_wake_routing_rejection(
    rejection: BridgeTemporalWakeRoutingRejection,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match rejection.rejection_kind() {
        BridgeTemporalWakeRoutingRejectionKind::DuplicateWakeSubmission => {
            BridgeTemporalAsyncFailureSubcode::OrderingDuplicateCause
        }
        BridgeTemporalWakeRoutingRejectionKind::StaleWakeSubmission => {
            BridgeTemporalAsyncFailureSubcode::OrderingReplayDrift
        }
        BridgeTemporalWakeRoutingRejectionKind::RoutingLaneMismatch
        | BridgeTemporalWakeRoutingRejectionKind::TruthPatchBranchIdentityMismatch
        | BridgeTemporalWakeRoutingRejectionKind::TruthPatchSnapshotIdentityMismatch => {
            BridgeTemporalAsyncFailureSubcode::TemporalReadinessWakeMissing
        }
    };
    let failure_class = match subcode {
        BridgeTemporalAsyncFailureSubcode::OrderingDuplicateCause
        | BridgeTemporalAsyncFailureSubcode::OrderingReplayDrift => {
            BridgeTemporalAsyncFailureClass::OrderingFailure
        }
        _ => BridgeTemporalAsyncFailureClass::TemporalReadinessFailure,
    };
    Ok((
        failure_class,
        subcode,
        vec![BridgeFailureEvidenceAttachment::reference(
            "subscription.temporal_wake_routing_rejection",
            format!(
                "{}:{}",
                rejection.wake_id().get(),
                rejection.wake_tick().get()
            ),
            rejection.digest(),
        )
        .with_detail(rejection.rejection_kind().as_str())],
        Arc::from(rejection.rejection_kind().as_str()),
    ))
}

pub(super) fn localize_resume_basis(
    rejection: BridgeSubscriptionResumeBasisRejection,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match rejection.rejection_kind() {
        BridgeSubscriptionResumeBasisRejectionKind::DeliveryBasisMissing
        | BridgeSubscriptionResumeBasisRejectionKind::TemporalBasisMissing
        | BridgeSubscriptionResumeBasisRejectionKind::InflightAsyncBasisMissing => {
            BridgeTemporalAsyncFailureSubcode::ResumeBasisMissing
        }
        BridgeSubscriptionResumeBasisRejectionKind::InflightAsyncGenerationMissing => {
            BridgeTemporalAsyncFailureSubcode::ResumeBasisStale
        }
        BridgeSubscriptionResumeBasisRejectionKind::RetentionTruncated => {
            BridgeTemporalAsyncFailureSubcode::ResumeBasisTruncated
        }
        BridgeSubscriptionResumeBasisRejectionKind::ActiveSubscriptionMismatch
        | BridgeSubscriptionResumeBasisRejectionKind::AdmittedSubscriptionMismatch
        | BridgeSubscriptionResumeBasisRejectionKind::BasisMismatch
        | BridgeSubscriptionResumeBasisRejectionKind::CostProfileMismatch
        | BridgeSubscriptionResumeBasisRejectionKind::ConsumerContractMismatch
        | BridgeSubscriptionResumeBasisRejectionKind::DeliveryBasisMismatch
        | BridgeSubscriptionResumeBasisRejectionKind::CrossBranchResumeRejected => {
            BridgeTemporalAsyncFailureSubcode::ResumeBasisIncompatible
        }
    };
    Ok((
        BridgeTemporalAsyncFailureClass::ResumeBasisFailure,
        subcode,
        vec![BridgeFailureEvidenceAttachment::reference(
            "subscription.resume_basis_rejection",
            rejection.rejection_kind().as_str(),
            rejection.digest(),
        )],
        Arc::from(rejection.rejection_kind().as_str()),
    ))
}

pub(super) fn localize_preview_discard(
    rejection: BridgeSubscriptionPreviewLifecycleDiscardRejection,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    Ok((
        BridgeTemporalAsyncFailureClass::PreviewBoundaryFailure,
        BridgeTemporalAsyncFailureSubcode::PreviewBoundaryDiscardResidue,
        vec![BridgeFailureEvidenceAttachment::reference(
            "subscription.preview_discard_rejection",
            rejection.rejection_kind().as_str(),
            rejection.digest(),
        )],
        Arc::from(rejection.rejection_kind().as_str()),
    ))
}

pub(super) fn localize_preview_promotion(
    rejection: BridgeSubscriptionPreviewLifecyclePromotionRejection,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match rejection.rejection_kind() {
        BridgeSubscriptionPreviewLifecyclePromotionRejectionKind::PreviewCrossedCompletion => {
            BridgeTemporalAsyncFailureSubcode::PreviewBoundaryPreviewCrossedCompletion
        }
        _ => BridgeTemporalAsyncFailureSubcode::PreviewBoundaryPromotionMismatch,
    };
    Ok((
        BridgeTemporalAsyncFailureClass::PreviewBoundaryFailure,
        subcode,
        vec![BridgeFailureEvidenceAttachment::reference(
            "subscription.preview_promotion_rejection",
            rejection.rejection_kind().as_str(),
            rejection.digest(),
        )],
        Arc::from(rejection.rejection_kind().as_str()),
    ))
}

pub(super) fn localize_mixed_cause_denial(
    denied: BridgeDeniedMixedCause,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match denied.denied_kind() {
        BridgeMixedCauseDeniedKind::AuthoritativePreviewCauseRejected => {
            BridgeTemporalAsyncFailureSubcode::OrderingSuppressedCause
        }
        BridgeMixedCauseDeniedKind::AsyncStaleCauseRejected
        | BridgeMixedCauseDeniedKind::AsyncLineageNonDeliverable => {
            BridgeTemporalAsyncFailureSubcode::OrderingReplayDrift
        }
    };
    Ok((
        BridgeTemporalAsyncFailureClass::OrderingFailure,
        subcode,
        vec![BridgeFailureEvidenceAttachment::reference(
            "subscription.mixed_cause_denial",
            denied.source_identity(),
            denied.digest(),
        )
        .with_detail(format!("{:?}", denied.denied_kind()))],
        Arc::from(format!("{:?}", denied.denied_kind())),
    ))
}

pub(super) fn localize_mixed_cause_suppression(
    suppressed: BridgeSuppressedMixedCause,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    Ok((
        BridgeTemporalAsyncFailureClass::OrderingFailure,
        BridgeTemporalAsyncFailureSubcode::OrderingSuppressedCause,
        vec![synthetic_attachment(
            "subscription.mixed_cause_suppression",
            suppressed.digest(),
            format!("{:?}", suppressed.suppressed_kind()),
        )],
        Arc::from(format!("{:?}", suppressed.suppressed_kind())),
    ))
}

pub(super) fn localize_policy_rejection(
    rejection: BridgePolicyRejection,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match rejection.kind() {
        BridgePolicyRejectionKind::PolicyLoweringMismatch => {
            BridgeTemporalAsyncFailureSubcode::PolicyRemaskSchemaContextDrift
        }
        BridgePolicyRejectionKind::PreviewPolicyBoundaryViolation => {
            BridgeTemporalAsyncFailureSubcode::PolicyRemaskTenantDrift
        }
        BridgePolicyRejectionKind::PolicySourceAmbiguity
        | BridgePolicyRejectionKind::UnsupportedExecutionMode
        | BridgePolicyRejectionKind::ReplayPolicyConflict
        | BridgePolicyRejectionKind::DiagnosticsPolicyConflict
        | BridgePolicyRejectionKind::ArtifactRetentionConflict
        | BridgePolicyRejectionKind::TruthViewPolicyConflict => {
            BridgeTemporalAsyncFailureSubcode::PolicyRemaskPolicyDrift
        }
    };
    Ok((
        BridgeTemporalAsyncFailureClass::PolicyRemaskFailure,
        subcode,
        vec![BridgeFailureEvidenceAttachment::reference(
            "policy.rejection",
            rejection.declaration_identity().as_str(),
            rejection.digest(),
        )
        .with_detail(rejection.detail())],
        Arc::from(rejection.detail().to_owned()),
    ))
}
