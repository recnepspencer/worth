use std::sync::Arc;

use super::localization::LocalizedFailureParts;
use super::{
    BridgeFailureEvidenceAttachment, BridgeTemporalAsyncFailureClass,
    BridgeTemporalAsyncFailureLocalizationRejection,
    BridgeTemporalAsyncFailureLocalizationRejectionKind, BridgeTemporalAsyncFailureSubcode,
};
use crate::source::{
    BridgeAsyncClassifiedDeniedCompletion, BridgeAsyncCompletionRejection,
    BridgeAsyncCompletionRejectionKind, BridgeAsyncCompletionSupersessionClass,
    BridgeAsyncForwardCausalityRejection, BridgeAsyncForwardCausalityRejectionKind,
    BridgeAsyncRequestIdentityRejection, BridgeAsyncRequestIdentityRejectionKind,
    BridgeAsyncWritebackCommitReport, BridgeAsyncWritebackNoopClass,
    BridgeAsyncWritebackRejectedClass, BridgeAsyncWritebackRejection,
    BridgeAsyncWritebackRejectionKind,
};

fn synthetic_attachment(
    family: &str,
    identity: impl Into<String>,
    detail: impl Into<String>,
) -> BridgeFailureEvidenceAttachment {
    let identity = identity.into();
    let detail = detail.into();
    BridgeFailureEvidenceAttachment::synthetic(family, identity, detail)
}

pub(super) fn localize_async_request_identity(
    rejection: BridgeAsyncRequestIdentityRejection,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match rejection.kind() {
        BridgeAsyncRequestIdentityRejectionKind::FamilyKindMismatch => {
            BridgeTemporalAsyncFailureSubcode::AsyncIdentitySourceMismatch
        }
        BridgeAsyncRequestIdentityRejectionKind::LoweringIdentityMismatch => {
            BridgeTemporalAsyncFailureSubcode::AsyncIdentityBasisMismatch
        }
        BridgeAsyncRequestIdentityRejectionKind::SubscriptionInstanceRequired
        | BridgeAsyncRequestIdentityRejectionKind::SubscriptionInstanceUnexpected => {
            BridgeTemporalAsyncFailureSubcode::AsyncIdentitySubscriptionInstanceDrift
        }
        BridgeAsyncRequestIdentityRejectionKind::PreviewBasisSubscriptionInstanceMismatch => {
            BridgeTemporalAsyncFailureSubcode::AsyncIdentityPreviewMismatch
        }
        BridgeAsyncRequestIdentityRejectionKind::SignalRuntimeThreadAffinityViolation
        | BridgeAsyncRequestIdentityRejectionKind::SignalRequestAdmissionRejected
        | BridgeAsyncRequestIdentityRejectionKind::SignalAsyncRequestBlocked
        | BridgeAsyncRequestIdentityRejectionKind::InFlightRequestMissing => {
            BridgeTemporalAsyncFailureSubcode::AsyncIdentityGenerationDrift
        }
    };
    Ok((
        BridgeTemporalAsyncFailureClass::AsyncIdentityFailure,
        subcode,
        vec![BridgeFailureEvidenceAttachment::reference(
            "source.async_request_identity_rejection",
            format!("{:?}", rejection.kind()),
            rejection.digest(),
        )],
        Arc::from(rejection.detail().to_owned()),
    ))
}

pub(super) fn localize_async_completion_rejection(
    rejection: BridgeAsyncCompletionRejection,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match rejection.kind() {
        BridgeAsyncCompletionRejectionKind::EnvelopeHandleMismatch
        | BridgeAsyncCompletionRejectionKind::EnvelopeAttemptMismatch
        | BridgeAsyncCompletionRejectionKind::PayloadContractDigestMismatch => {
            BridgeTemporalAsyncFailureSubcode::CompletionAdmissionEnvelopeInvalid
        }
        BridgeAsyncCompletionRejectionKind::FamilyKindMismatch => {
            BridgeTemporalAsyncFailureSubcode::CompletionAdmissionTransportRejected
        }
        BridgeAsyncCompletionRejectionKind::SignalRuntimeThreadAffinityViolation
        | BridgeAsyncCompletionRejectionKind::SignalCompletionAdmissionUnavailable
        | BridgeAsyncCompletionRejectionKind::ForeignOwnerObservationAuthority
        | BridgeAsyncCompletionRejectionKind::SupersessionMismatch => {
            BridgeTemporalAsyncFailureSubcode::CompletionAdmissionLifecycleDenied
        }
    };
    Ok((
        BridgeTemporalAsyncFailureClass::CompletionAdmissionFailure,
        subcode,
        vec![BridgeFailureEvidenceAttachment::reference(
            "source.async_completion_rejection",
            format!("{:?}", rejection.kind()),
            rejection.digest(),
        )],
        Arc::from(rejection.detail().to_owned()),
    ))
}

pub(super) fn localize_classified_completion(
    classified: BridgeAsyncClassifiedDeniedCompletion,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match classified.supersession_class() {
        BridgeAsyncCompletionSupersessionClass::TruthBasisSuperseded => {
            BridgeTemporalAsyncFailureSubcode::SupersessionTruthBasis
        }
        BridgeAsyncCompletionSupersessionClass::BranchDrifted => {
            BridgeTemporalAsyncFailureSubcode::SupersessionBranch
        }
        BridgeAsyncCompletionSupersessionClass::PreviewBasisDrifted
        | BridgeAsyncCompletionSupersessionClass::PreviewDiscarded => {
            BridgeTemporalAsyncFailureSubcode::SupersessionPreview
        }
        BridgeAsyncCompletionSupersessionClass::SubscriptionInstanceSuperseded => {
            BridgeTemporalAsyncFailureSubcode::SupersessionSubscriptionInstance
        }
        BridgeAsyncCompletionSupersessionClass::SignalGenerationSuperseded => {
            BridgeTemporalAsyncFailureSubcode::SupersessionGeneration
        }
    };
    Ok((
        BridgeTemporalAsyncFailureClass::SupersessionFailure,
        subcode,
        vec![
            BridgeFailureEvidenceAttachment::reference(
                "source.async_completion_supersession_receipt",
                classified.receipt().supersession_identity(),
                classified.receipt().digest(),
            ),
            BridgeFailureEvidenceAttachment::reference(
                "source.async_completion_supersession_evidence",
                classified.evidence().supersession_identity().as_str(),
                classified.evidence().digest(),
            ),
        ],
        Arc::from(format!("{:?}", classified.supersession_class())),
    ))
}

pub(super) fn localize_async_forward_causality_rejection(
    rejection: BridgeAsyncForwardCausalityRejection,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match rejection.kind() {
        BridgeAsyncForwardCausalityRejectionKind::TimeoutEvidenceMissing => {
            BridgeTemporalAsyncFailureSubcode::RetryRevalidationTimeout
        }
        BridgeAsyncForwardCausalityRejectionKind::CancellationEvidenceMissing => {
            BridgeTemporalAsyncFailureSubcode::RetryRevalidationCancelled
        }
        BridgeAsyncForwardCausalityRejectionKind::BasisDriftRequiredForRevalidation
        | BridgeAsyncForwardCausalityRejectionKind::RevalidationAdmissionMissing
        | BridgeAsyncForwardCausalityRejectionKind::StaleSignalGenerationRejected => {
            BridgeTemporalAsyncFailureSubcode::RetryRevalidationRevalidationRejected
        }
        _ => BridgeTemporalAsyncFailureSubcode::RetryRevalidationRetryRejected,
    };
    Ok((
        BridgeTemporalAsyncFailureClass::RetryRevalidationFailure,
        subcode,
        vec![synthetic_attachment(
            "source.async_forward_causality_rejection",
            format!("{:?}", rejection.kind()),
            rejection.detail(),
        )],
        Arc::from(rejection.detail().to_owned()),
    ))
}

pub(super) fn localize_async_writeback(
    rejection: BridgeAsyncWritebackRejection,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let subcode = match rejection.kind() {
        BridgeAsyncWritebackRejectionKind::MapperFailed => {
            BridgeTemporalAsyncFailureSubcode::WritebackBoundaryMapperFailed
        }
        _ => BridgeTemporalAsyncFailureSubcode::WritebackBoundaryAuthorityRejected,
    };
    Ok((
        BridgeTemporalAsyncFailureClass::WritebackBoundaryFailure,
        subcode,
        vec![BridgeFailureEvidenceAttachment::reference(
            "source.async_writeback_rejection",
            format!("{:?}", rejection.kind()),
            rejection.digest(),
        )],
        Arc::from(rejection.detail().to_owned()),
    ))
}

pub(super) fn localize_async_writeback_commit_report(
    report: BridgeAsyncWritebackCommitReport,
) -> Result<LocalizedFailureParts, BridgeTemporalAsyncFailureLocalizationRejection> {
    let (subcode, attachment) = match report {
        BridgeAsyncWritebackCommitReport::Noop(noop) => {
            let subcode = match noop.noop_class() {
                BridgeAsyncWritebackNoopClass::DuplicateCompletion
                | BridgeAsyncWritebackNoopClass::CanonicalNoop => {
                    BridgeTemporalAsyncFailureSubcode::WritebackBoundaryIdempotentNoop
                }
            };
            (
                subcode,
                BridgeFailureEvidenceAttachment::reference(
                    "source.async_writeback_noop",
                    noop.receipt_identity().as_str(),
                    noop.noop_identity().as_str(),
                )
                .with_detail(format!("{:?}", noop.noop_class())),
            )
        }
        BridgeAsyncWritebackCommitReport::Rejected(rejected) => {
            let subcode = match rejected.rejected_class() {
                BridgeAsyncWritebackRejectedClass::LoopPreventionRejected => {
                    BridgeTemporalAsyncFailureSubcode::WritebackBoundaryLoopPrevented
                }
                BridgeAsyncWritebackRejectedClass::AuthorityRejected => {
                    BridgeTemporalAsyncFailureSubcode::WritebackBoundaryAuthorityRejected
                }
            };
            (
                subcode,
                BridgeFailureEvidenceAttachment::reference(
                    "source.async_writeback_rejected_outcome",
                    rejected.receipt_identity().as_str(),
                    rejected.rejected_identity().as_str(),
                )
                .with_detail(rejected.detail()),
            )
        }
        BridgeAsyncWritebackCommitReport::Committed(committed) => {
            return Err(BridgeTemporalAsyncFailureLocalizationRejection::new(
                BridgeTemporalAsyncFailureLocalizationRejectionKind::UnsupportedFailureArtifact,
                format!(
                    "committed async writeback `{}` is not a failure artifact",
                    committed.committed_identity().as_str()
                ),
            ));
        }
    };
    Ok((
        BridgeTemporalAsyncFailureClass::WritebackBoundaryFailure,
        subcode,
        vec![attachment],
        Arc::from(subcode.as_str()),
    ))
}
