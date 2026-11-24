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
        WorthQueryConditionalExecutionCause::NoEffect(
            WorthQueryApplicationNoEffectCause::CapacityExhausted,
        ),
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

#[test]
fn bridge_managed_temporal_retention_capacity_classifies_as_backpressured() {
    use worth_query_execution::facade::primary_graph::{
        WorthQueryConditionalClockObservationDenialKind,
        WorthQueryConditionalClockObservationFailureKind,
        WorthQueryConditionalRuntimeInstallationDenialKind,
    };
    use worth_runtime_bridge::facade::BridgeManagedTemporalDenialKind;

    for kind in [
        BridgeManagedTemporalDenialKind::RetentionCapacityExhausted,
        BridgeManagedTemporalDenialKind::IntentCapacityExhausted,
    ] {
        let clock_denial = WorthQueryConditionalClockObservationDenialKind::ProductAdmission(
            WorthQueryConditionalRuntimeInstallationDenialKind::ManagedTemporal(kind),
        );
        assert_eq!(
            denials::classify_clock_port_denial_kind(&clock_denial),
            Some(WorthQueryTemporalCatchUpStatus::Backpressured)
        );

        let recon_failure = WorthQueryConditionalClockObservationFailureKind::ReconstructionDenied(
            WorthQueryConditionalRuntimeInstallationDenialKind::ManagedTemporal(kind),
        );
        assert_eq!(
            denials::classify_observation_failure_kind(&recon_failure),
            Some(WorthQueryTemporalCatchUpStatus::Backpressured)
        );
    }

    assert_eq!(
        denials::classify_observation_failure_kind(
            &WorthQueryConditionalClockObservationFailureKind::RetentionCapacityExhausted,
        ),
        Some(WorthQueryTemporalCatchUpStatus::Backpressured)
    );
}

#[test]
fn accumulator_tracks_progress_for_post_progress_step_refusal() {
    use super::super::WorthQueryTemporalCatchUpDenial;

    let mut acc = CatchUpAccumulator::<()>::new();
    assert!(!acc.has_progress_or_batches());
    acc.steps_executed = 1;
    assert!(acc.has_progress_or_batches());
    let report = acc.finish(
        WorthQueryTemporalCatchUpStatus::StepRefused(Box::new(
            WorthQueryTemporalCatchUpDenial::ForeignClockHandle,
        )),
        true,
    );
    assert!(matches!(
        report.status(),
        WorthQueryTemporalCatchUpStatus::StepRefused(denial)
            if matches!(denial.as_ref(), WorthQueryTemporalCatchUpDenial::ForeignClockHandle)
    ));
    assert!(report.remaining());
    assert_eq!(report.steps_executed(), 1);
}
