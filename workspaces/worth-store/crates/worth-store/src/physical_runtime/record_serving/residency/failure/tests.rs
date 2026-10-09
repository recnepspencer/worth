use super::*;

#[test]
fn lower_conflicts_keep_distinct_store_owned_meaning() {
    for denial in [
        PhysicalResidencyDenial::EmptyCandidateBatch,
        PhysicalResidencyDenial::CandidateCardinalityMismatch {
            declared: 2,
            provided: 1,
        },
        PhysicalResidencyDenial::DuplicateCandidateIdentity,
        PhysicalResidencyDenial::CandidateCoverageConflict,
        PhysicalResidencyDenial::CandidateSequenceConflict,
    ] {
        assert_eq!(
            PhysicalRecordResidencyFailure::from(denial).kind(),
            PhysicalRecordResidencyFailureKind::CandidateContractConflict
        );
    }
    assert_eq!(
        PhysicalRecordResidencyFailure::from(PhysicalResidencyDenial::BoundedLoadLimitConflict {
            active_limit: 16,
            requested_limit: 64,
        })
        .kind(),
        PhysicalRecordResidencyFailureKind::FrameLoadConflict
    );
    for denial in [
        PhysicalResidencyDenial::CompleteArtifactRequiresOffsetZero,
        PhysicalResidencyDenial::ArtifactIdentityOccupied,
        PhysicalResidencyDenial::FrameIdentityOccupied,
    ] {
        assert_eq!(
            PhysicalRecordResidencyFailure::from(denial).kind(),
            PhysicalRecordResidencyFailureKind::FrameIdentityConflict
        );
    }
}

#[test]
fn exact_store_reasons_separate_declaration_and_identity_actions() {
    let cases = [
        (
            PhysicalResidencyDenial::CompleteArtifactRequiresOffsetZero,
            PhysicalRecordResidencyFailureReason::CompleteArtifactRequiresOffsetZero,
        ),
        (
            PhysicalResidencyDenial::ArtifactIdentityOccupied,
            PhysicalRecordResidencyFailureReason::ArtifactIdentityOccupied,
        ),
        (
            PhysicalResidencyDenial::FrameIdentityOccupied,
            PhysicalRecordResidencyFailureReason::FrameIdentityOccupied,
        ),
        (
            PhysicalResidencyDenial::IdentityAlreadyCurrent,
            PhysicalRecordResidencyFailureReason::IdentityAlreadyCurrent,
        ),
    ];
    for (denial, expected) in cases {
        assert_eq!(
            PhysicalRecordResidencyFailure::from(denial).reason(),
            expected
        );
    }
}

#[test]
fn exact_store_reasons_separate_frame_and_writeback_actions() {
    for (denial, expected) in [
        (
            PhysicalResidencyDenial::FrameLengthMismatch,
            PhysicalRecordResidencyFailureReason::FrameLengthMismatch,
        ),
        (
            PhysicalResidencyDenial::FramePinned,
            PhysicalRecordResidencyFailureReason::FramePinned,
        ),
        (
            PhysicalResidencyDenial::FrameDirty,
            PhysicalRecordResidencyFailureReason::FrameDirty,
        ),
        (
            PhysicalResidencyDenial::WriteBackFrameAlreadyClaimed,
            PhysicalRecordResidencyFailureReason::WritebackFrameAlreadyClaimed,
        ),
        (
            PhysicalResidencyDenial::WriteBackReceiptMismatch,
            PhysicalRecordResidencyFailureReason::WritebackReceiptMismatch,
        ),
    ] {
        assert_eq!(
            PhysicalRecordResidencyFailure::from(denial).reason(),
            expected
        );
    }
}

#[test]
fn cleaning_authority_mismatches_remain_exact_and_non_retryable() {
    for (denial, reason) in [
        (
            PhysicalResidencyDenial::CandidateCleanAuthorityMismatch,
            PhysicalRecordResidencyFailureReason::CandidateCleanAuthorityMismatch,
        ),
        (
            PhysicalResidencyDenial::WritebackCleanAuthorityMismatch,
            PhysicalRecordResidencyFailureReason::WritebackCleanAuthorityMismatch,
        ),
    ] {
        let failure = PhysicalRecordResidencyFailure::from(denial);
        assert_eq!(
            failure.kind(),
            PhysicalRecordResidencyFailureKind::SettlementAuthorityMismatch
        );
        assert_eq!(failure.reason(), reason);
    }
}

#[test]
fn exact_store_reasons_retain_actionable_parameters() {
    let resize = PhysicalRecordResidencyFailure::from(
        PhysicalResidencyDenial::AllocationGrantResizeBelowActiveUse {
            requested: 40,
            active: 41,
        },
    );
    assert_eq!(
        resize.kind(),
        PhysicalRecordResidencyFailureKind::AllocationUseConflict
    );
    assert_eq!(
        resize.reason(),
        PhysicalRecordResidencyFailureReason::AllocationGrantResizeBelowActiveUse {
            requested: 40,
            active: 41,
        }
    );
    let allocation = PhysicalRecordResidencyFailure::from(
        PhysicalResidencyDenial::AllocatorExceededReservation {
            requested: 8,
            actual: 16,
        },
    );
    assert_eq!(
        allocation.kind(),
        PhysicalRecordResidencyFailureKind::AllocationUnavailable
    );
    assert_eq!(
        allocation.reason(),
        PhysicalRecordResidencyFailureReason::AllocatorExceededReservation {
            requested: 8,
            actual: 16,
        }
    );
    assert_eq!(
        PhysicalRecordResidencyFailure::from(PhysicalResidencyDenial::BoundedLoadLimitConflict {
            active_limit: 16,
            requested_limit: 64,
        })
        .reason(),
        PhysicalRecordResidencyFailureReason::BoundedLoadLimitConflict {
            active_limit: 16,
            requested_limit: 64,
        }
    );
    assert_eq!(
        PhysicalRecordResidencyFailure::from(
            PhysicalResidencyDenial::CandidateCardinalityMismatch {
                declared: 3,
                provided: 2,
            }
        )
        .reason(),
        PhysicalRecordResidencyFailureReason::CandidateCardinalityMismatch {
            declared: 3,
            provided: 2,
        }
    );
}

#[test]
fn dirty_generation_failures_keep_distinct_remediation_and_exact_budget() {
    let exhausted =
        PhysicalRecordResidencyFailure::from(PhysicalResidencyDenial::DirtyGenerationExhausted);
    assert_eq!(
        exhausted.kind(),
        PhysicalRecordResidencyFailureKind::DirtyGenerationExhausted
    );
    assert_eq!(
        exhausted.reason(),
        PhysicalRecordResidencyFailureReason::DirtyGenerationExhausted
    );

    let underfunded = PhysicalRecordResidencyFailure::from(
        PhysicalResidencyDenial::DirtyGenerationCaptureBudgetExceeded {
            required: 32,
            admitted: 16,
        },
    );
    assert_eq!(
        underfunded.kind(),
        PhysicalRecordResidencyFailureKind::AllocationUnavailable
    );
    assert_eq!(
        underfunded.reason(),
        PhysicalRecordResidencyFailureReason::DirtyGenerationCaptureBudgetExceeded {
            required: 32,
            admitted: 16,
        }
    );

    let foreign_session = PhysicalRecordResidencyFailure::from(
        PhysicalResidencyDenial::DirtyGenerationCaptureSessionMismatch,
    );
    assert_eq!(
        foreign_session.kind(),
        PhysicalRecordResidencyFailureKind::CaptureSessionAuthorityMismatch
    );
    assert_eq!(
        foreign_session.reason(),
        PhysicalRecordResidencyFailureReason::DirtyGenerationCaptureSessionMismatch
    );
}
