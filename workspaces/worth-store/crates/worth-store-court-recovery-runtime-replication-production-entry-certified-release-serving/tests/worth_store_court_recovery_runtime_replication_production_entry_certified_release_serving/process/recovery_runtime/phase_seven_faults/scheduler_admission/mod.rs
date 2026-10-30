use super::*;

#[test]
fn cleanup_scheduler_rejection_after_removal_retains_escaped_effect_without_performed_authority() {
    let world = cleanup_world("cleanup-scheduler-settlement");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    reopened.certification_fail_cleanup_scheduler_settlement_at(
        worth_store::physical_runtime::PhysicalRecoveryCleanupCommandStage::Removal,
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        reopened.finish()
    else {
        panic!("cleanup scheduler debt cannot invalidate recovered publication")
    };
    let worth_store_recovery_runtime::RecoveryCleanupPosture::Deferred(evidence) =
        handoff.cleanup_posture()
    else {
        panic!("escaped removal with rejected scheduler settlement is deferred")
    };
    assert!(!candidate.exists());
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().removal_scheduler_submitted, 1);
    assert_eq!(evidence.counters().removal_scheduler_settled, 1);
    assert_eq!(evidence.counters().performed_effects, 0);
    let [worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence::IndeterminateEffect {
        evidence:
            worth_store::physical_runtime::PhysicalRecoveryCleanupRemovalIndeterminate::Scheduler {
                posture,
                signal,
                ..
            },
        ..
    }] = evidence.deferrals()
    else {
        panic!("one exact scheduler-indeterminate cleanup settlement")
    };
    assert_eq!(
        *posture,
        worth_store::physical_runtime::PhysicalWorkSchedulerPosture::RejectedAfterEffect
    );
    assert_eq!(
        *signal,
        worth_store::physical_runtime::PhysicalSignalSettlementOutcome::ReconciledFromPhysicalTruth
    );
}

#[test]
fn cleanup_background_deferral_cancels_before_the_physical_effect() {
    let world = cleanup_world("cleanup-background-deferral");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    reopened.certification_defer_cleanup_background();
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        reopened.finish()
    else {
        panic!("scheduler deferral remains cleanup debt")
    };
    let worth_store_recovery_runtime::RecoveryCleanupPosture::Deferred(evidence) =
        handoff.cleanup_posture()
    else {
        panic!("background pacing deferral retains the candidate")
    };
    assert!(candidate.exists());
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().removal_scheduler_submitted, 1);
    assert_eq!(evidence.counters().removal_scheduler_deferred, 1);
    assert_eq!(evidence.counters().removal_scheduler_cancelled, 1);
    assert_eq!(evidence.counters().removal_scheduler_settled, 0);
    let [worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence::DeniedBeforeEffect {
        denial,
        ..
    }] = evidence.deferrals()
    else {
        panic!("one exact pre-effect cleanup deferral")
    };
    let worth_store::physical_runtime::PhysicalRecoveryCleanupRemovalDenialKind::Admission(
        admission,
    ) = denial.kind()
    else {
        panic!("cleanup deferral retains Store admission evidence")
    };
    assert!(matches!(
        admission.kind(),
        worth_store::physical_runtime::PhysicalRecoveryCleanupAdmissionDenialKind::BackgroundPacing(
            _
        )
    ));
}

#[test]
fn freshness_scheduler_and_signal_failures_stop_before_cleanup_effects() {
    for (label, signal) in [
        ("cleanup-freshness-scheduler", false),
        ("cleanup-freshness-signal", true),
    ] {
        let world = cleanup_world(label);
        let candidate = world.oldest_wal();
        let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
        if signal {
            reopened.certification_fail_cleanup_freshness_signal_settlement();
        } else {
            reopened.certification_fail_cleanup_scheduler_settlement_at(
                worth_store::physical_runtime::PhysicalRecoveryCleanupCommandStage::FreshnessRead,
            );
        }
        let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
            reopened.finish()
        else {
            panic!("freshness failure remains cleanup debt")
        };
        let worth_store_recovery_runtime::RecoveryCleanupPosture::Deferred(evidence) =
            handoff.cleanup_posture()
        else {
            panic!("freshness settlement failure defers cleanup")
        };
        assert!(candidate.exists());
        assert!(evidence.performed_removals().is_empty());
        assert_eq!(evidence.counters().actions_attempted, 0);
        assert_eq!(evidence.counters().freshness_reads_completed, 1);
        assert_eq!(evidence.counters().freshness_scheduler_submitted, 1);
        assert_eq!(evidence.counters().freshness_scheduler_settled, 1);
        let [worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence::Freshness {
            failure,
            ..
        }] = evidence.deferrals()
        else {
            panic!("one exact freshness failure")
        };
        let kind = failure.read().expect("completed freshness read").kind();
        assert!(if signal {
            matches!(
                kind,
                worth_store::physical_runtime::PhysicalRecoveryCleanupFreshnessReadDenialKind::SignalSettlement(
                    worth_store::physical_runtime::PhysicalSignalSettlementOutcome::DerivedStateUnavailable
                )
            )
        } else {
            matches!(
                kind,
                worth_store::physical_runtime::PhysicalRecoveryCleanupFreshnessReadDenialKind::SchedulerSettlement(
                    worth_store::physical_runtime::PhysicalWorkSchedulerPosture::RejectedAfterEffect
                )
            )
        });
    }
}
