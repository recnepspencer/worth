use super::*;

#[test]
fn capacity_deferrals_classify_as_backpressure() {
    for cause in [
        WorthQueryConditionalExecutionCause::CandidateCapacityExhausted {
            maximum_candidates: 1,
        },
        WorthQueryConditionalExecutionCause::PublishedSnapshotCapacityExhausted {
            maximum_handles: 1,
        },
        WorthQueryConditionalExecutionCause::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots: 1,
        },
        WorthQueryConditionalExecutionCause::RetentionCapacityExhausted,
        WorthQueryConditionalExecutionCause::PatchPositionReservationContended,
    ] {
        assert!(is_backpressure_cause(Some(&cause)), "{cause:?}");
    }
}

#[test]
fn non_auth_terminal_causes_never_classify_as_authorization_refused() {
    for cause in [
        WorthQueryConditionalExecutionCause::TerminalFailure,
        WorthQueryConditionalExecutionCause::ApplicationCommitDenied(
            WorthQueryApplicationCommitDenialKind::UnobservedWrite,
        ),
        WorthQueryConditionalExecutionCause::ApplicationCommitDenied(
            WorthQueryApplicationCommitDenialKind::DomainValueRejected,
        ),
    ] {
        let (mut c, mut b, mut a, mut o, mut bp, mut bl) =
            (false, false, false, false, false, false);
        classify_failure_cause(
            Some(&cause),
            &mut c,
            &mut b,
            &mut a,
            &mut o,
            &mut bp,
            &mut bl,
        );
        assert!(!a, "{cause:?} must not map to AuthorizationRefused");
        assert!(o, "{cause:?} must map to OperationRefused");
    }
    let (mut c, mut b, mut a, mut o, mut bp, mut bl) = (false, false, false, false, false, false);
    classify_failure_cause(
        Some(&WorthQueryConditionalExecutionCause::AuthorizationRefused),
        &mut c,
        &mut b,
        &mut a,
        &mut o,
        &mut bp,
        &mut bl,
    );
    assert!(a);
    assert!(!o);
}
