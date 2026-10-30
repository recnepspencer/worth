#![cfg_attr(test, allow(clippy::disallowed_methods))]
#![cfg(feature = "certification-test-authority")]

#[allow(dead_code)]
mod phase_three_support;
#[path = "phase_seven_faults/revalidation.rs"]
mod revalidation;
#[path = "phase_seven_faults/world.rs"]
mod world;

use world::{
    assert_child_succeeded, cleanup_fault, cleanup_world, empty_fault_schedule,
    paused_cleanup_schedule, recover_with_schedule, reopen_with_schedule, required_child_root,
    required_crash_marker, run_child, spawn_crashing_child,
};

#[test]
fn killed_cleanup_after_the_deletion_effect_reopens_in_a_fresh_process() {
    let world = cleanup_world("cleanup-crash-restart");
    let candidate = world.oldest_wal();
    let marker = world.root.parent().unwrap().join("cleanup-effect-reached");
    let mut first =
        spawn_crashing_child("phase_seven_killed_cleanup_process", &world.root, &marker);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while !marker.exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "cleanup child did not reach the post-effect crash seam"
        );
        assert!(
            first.try_wait().unwrap().is_none(),
            "cleanup child exited before kill"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    first.kill().unwrap();
    let killed = first.wait_with_output().unwrap();
    assert!(
        !killed.status.success(),
        "parent must terminate the cleanup child"
    );
    assert!(
        !candidate.exists(),
        "the deletion effect must escape before the forced process death"
    );
    let second = run_child("phase_seven_reopen_after_cleanup_process", &world.root);
    assert_child_succeeded("post-cleanup recovery", &second);
}

#[test]
#[ignore = "launched and forcibly terminated by the Phase 7 cleanup crash-matrix parent"]
fn phase_seven_killed_cleanup_process() {
    let root = required_child_root();
    let marker = required_crash_marker();
    let (schedule, gate) = paused_cleanup_schedule();
    let worker = std::thread::spawn(move || recover_with_schedule(&root, schedule));
    gate.wait_until_reached();
    std::fs::write(marker, b"cleanup effect reached before settlement").unwrap();
    std::hint::black_box(&worker);
    loop {
        std::thread::park();
    }
}

#[test]
#[ignore = "launched by the Phase 7 cleanup crash-matrix parent"]
fn phase_seven_indeterminate_cleanup_process() {
    let root = required_child_root();
    let handoff = recover_with_schedule(
        &root,
        cleanup_fault(MediaFaultDirective::IndeterminateAfterEffect),
    );
    assert!(matches!(
        handoff.cleanup_posture(),
        RecoveryCleanupPosture::Deferred(_)
    ));
}

#[test]
#[ignore = "launched by the Phase 7 cleanup crash-matrix parent"]
fn phase_seven_reopen_after_cleanup_process() {
    let root = required_child_root();
    let PhysicalRecoveryOutcome::Recovered(handoff) =
        reopen_with_schedule(&root, empty_fault_schedule()).finish()
    else {
        panic!("ambiguous cleanup debt cannot invalidate a fresh recovery")
    };
    assert!(!handoff
        .cleanup_posture()
        .evidence()
        .dispositions()
        .is_empty());
}
use worth_store::physical_runtime::certification::MediaFaultDirective;
use worth_store::physical_runtime::{
    PhysicalRecoveryCleanupAdmissionDenialKind, PhysicalRecoveryCleanupCommandStage,
    PhysicalRecoveryCleanupFreshnessReadDenialKind, PhysicalRecoveryCleanupRemovalDenialKind,
    PhysicalRecoveryCleanupRemovalIndeterminate, PhysicalSignalSettlementOutcome,
    PhysicalWorkSchedulerPosture, StoreRecoveryCleanupFreshnessDenial,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, RecoveryCleanupDeferralEvidence, RecoveryCleanupDispositionKind,
    RecoveryCleanupPosture,
};

#[test]
fn ambiguous_cleanup_effect_is_deferred_without_performed_authority() {
    let world = cleanup_world("cleanup-indeterminate");
    let candidate = world.oldest_wal();
    let handoff = recover_with_schedule(
        &world.root,
        cleanup_fault(MediaFaultDirective::IndeterminateAfterEffect),
    );
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("ambiguous cleanup still returns recovered handoff with deferred debt")
    };
    assert!(
        !candidate.exists(),
        "the physical deletion may have escaped"
    );
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().indeterminate_effects, 1);
    assert_eq!(evidence.counters().performed_effects, 0);
    assert!(matches!(
        evidence.deferrals(),
        [RecoveryCleanupDeferralEvidence::IndeterminateEffect { .. }]
    ));
    assert!(evidence.dispositions().iter().any(|disposition| {
        disposition.kind()
            == RecoveryCleanupDispositionKind::Deferred(
                worth_store_recovery_runtime::RecoveryCleanupDeferralReason::IndeterminateEffect,
            )
    }));
}

#[test]
fn owner_sampled_generation_change_rejects_the_stale_cleanup_plan() {
    let world = cleanup_world("cleanup-stale-plan");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    reopened.certification_shift_cleanup_generation();
    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("stale cleanup plan remains maintenance debt, not recovery failure")
    };
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("generation shift defers cleanup")
    };
    assert!(candidate.exists());
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().freshness_evaluations, 1);
    assert_eq!(evidence.counters().actions_attempted, 0);
    assert!(matches!(
        evidence.deferrals(),
        [RecoveryCleanupDeferralEvidence::PublishedGenerationChanged {
            expected,
            observed,
            ..
        }] if *observed == expected + 1
    ));
}

#[test]
fn cleanup_plan_admission_denial_is_retained_before_any_freshness_read() {
    let world = cleanup_world("cleanup-plan-admission-denial");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    reopened.certification_fail_cleanup_plan_admission();
    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("cleanup-plan denial remains deferred maintenance")
    };
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("cleanup-plan denial must be retained")
    };
    assert!(candidate.exists());
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().freshness_evaluations, 1);
    assert_eq!(evidence.counters().freshness_reads_completed, 0);
    assert_eq!(evidence.counters().freshness_bytes_read, 0);
    assert_eq!(evidence.counters().scheduler_commands_submitted, 0);
    assert_eq!(evidence.counters().actions_attempted, 0);
    let [RecoveryCleanupDeferralEvidence::Freshness { failure, .. }] = evidence.deferrals() else {
        panic!("one exact Store cleanup-plan admission denial")
    };
    assert_eq!(
        failure.denial(),
        StoreRecoveryCleanupFreshnessDenial::InvalidCleanupEligibility
    );
    assert!(failure.sample().is_none());
    assert!(failure.read().is_none());
}

#[test]
fn cleanup_media_handle_leak_blocks_the_recovered_handoff() {
    let world = cleanup_world("cleanup-media-handle-leak");
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    reopened.certification_leak_cleanup_media_handle();
    let PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) = reopened.finish() else {
        panic!("one live cleanup media handle must block recovered handoff")
    };
    assert_eq!(
        indeterminate.handoff_failure(),
        Some(
            worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::
                CleanupMediaNotQuiescent,
        )
    );
}

#[test]
fn post_read_eligibility_denial_retains_the_completed_freshness_stage() {
    let world = cleanup_world("cleanup-post-read-eligibility");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    reopened.certification_fail_cleanup_eligibility_after_read();
    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("post-read eligibility debt cannot invalidate recovery")
    };
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("post-read eligibility denial defers cleanup")
    };
    assert!(candidate.exists());
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().actions_attempted, 0);
    assert_eq!(evidence.counters().freshness_evaluations, 1);
    assert_eq!(evidence.counters().freshness_reads_completed, 1);
    assert_eq!(
        evidence.counters().freshness_bytes_read,
        worth_store_physical_format::ROOT_SELECTOR_BYTES as u64
    );
    assert_eq!(evidence.counters().freshness_scheduler_submitted, 1);
    assert_eq!(evidence.counters().freshness_scheduler_settled, 1);
    let [RecoveryCleanupDeferralEvidence::Freshness { failure, .. }] = evidence.deferrals() else {
        panic!("one exact post-read freshness failure")
    };
    assert_eq!(
        failure.denial(),
        StoreRecoveryCleanupFreshnessDenial::InvalidCleanupEligibility
    );
    assert!(failure.sample().is_some());
    assert_eq!(
        failure
            .sample()
            .expect("post-read failure retains exact sample")
            .selector_read()
            .bytes()
            .len(),
        worth_store_physical_format::ROOT_SELECTOR_BYTES
    );
    assert!(failure.read().is_none());
}

#[test]
fn cleanup_cancellation_from_another_plan_defers_without_effect() {
    let first = cleanup_world("cleanup-cancellation-source");
    let second = cleanup_world("cleanup-cancellation-target");
    let first_reopened = reopen_with_schedule(&first.root, empty_fault_schedule());
    let cancellation = first_reopened.cleanup_cancellation_before_first().unwrap();
    let second_candidate = second.oldest_wal();
    let second_reopened = reopen_with_schedule(&second.root, empty_fault_schedule());
    let PhysicalRecoveryOutcome::Recovered(handoff) =
        second_reopened.finish_with_cleanup_cancellation(cancellation)
    else {
        panic!("foreign cleanup cancellation remains optional deferred work")
    };
    assert!(second_candidate.exists());
    assert!(matches!(
        handoff.cleanup_posture().evidence().deferrals(),
        [RecoveryCleanupDeferralEvidence::CancellationBindingMismatch { .. }]
    ));
    assert_eq!(
        handoff
            .cleanup_posture()
            .evidence()
            .counters()
            .performed_effects,
        0
    );
}

#[test]
fn cleanup_scheduler_rejection_after_removal_retains_escaped_effect_without_performed_authority() {
    let world = cleanup_world("cleanup-scheduler-settlement");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    reopened.certification_fail_cleanup_scheduler_settlement_at(
        PhysicalRecoveryCleanupCommandStage::Removal,
    );
    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("cleanup scheduler debt cannot invalidate recovered publication")
    };
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("escaped removal with rejected scheduler settlement is deferred")
    };
    assert!(!candidate.exists());
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().removal_scheduler_submitted, 1);
    assert_eq!(evidence.counters().removal_scheduler_settled, 1);
    assert_eq!(evidence.counters().performed_effects, 0);
    let [RecoveryCleanupDeferralEvidence::IndeterminateEffect {
        evidence:
            PhysicalRecoveryCleanupRemovalIndeterminate::Scheduler {
                posture, signal, ..
            },
        ..
    }] = evidence.deferrals()
    else {
        panic!("one exact scheduler-indeterminate cleanup settlement")
    };
    assert_eq!(*posture, PhysicalWorkSchedulerPosture::RejectedAfterEffect);
    assert_eq!(
        *signal,
        PhysicalSignalSettlementOutcome::ReconciledFromPhysicalTruth
    );
}

#[test]
fn cleanup_background_deferral_cancels_before_the_physical_effect() {
    let world = cleanup_world("cleanup-background-deferral");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    reopened.certification_defer_cleanup_background();
    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("scheduler deferral remains cleanup debt")
    };
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("background pacing deferral retains the candidate")
    };
    assert!(candidate.exists());
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().removal_scheduler_submitted, 1);
    assert_eq!(evidence.counters().removal_scheduler_deferred, 1);
    assert_eq!(evidence.counters().removal_scheduler_cancelled, 1);
    assert_eq!(evidence.counters().removal_scheduler_settled, 0);
    let [RecoveryCleanupDeferralEvidence::DeniedBeforeEffect { denial, .. }] = evidence.deferrals()
    else {
        panic!("one exact pre-effect cleanup deferral")
    };
    let PhysicalRecoveryCleanupRemovalDenialKind::Admission(admission) = denial.kind() else {
        panic!("cleanup deferral retains Store admission evidence")
    };
    assert!(matches!(
        admission.kind(),
        PhysicalRecoveryCleanupAdmissionDenialKind::BackgroundPacing(_)
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
                PhysicalRecoveryCleanupCommandStage::FreshnessRead,
            );
        }
        let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
            panic!("freshness failure remains cleanup debt")
        };
        let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
            panic!("freshness settlement failure defers cleanup")
        };
        assert!(candidate.exists());
        assert!(evidence.performed_removals().is_empty());
        assert_eq!(evidence.counters().actions_attempted, 0);
        assert_eq!(evidence.counters().freshness_reads_completed, 1);
        assert_eq!(evidence.counters().freshness_scheduler_submitted, 1);
        assert_eq!(evidence.counters().freshness_scheduler_settled, 1);
        let [RecoveryCleanupDeferralEvidence::Freshness { failure, .. }] = evidence.deferrals()
        else {
            panic!("one exact freshness failure")
        };
        let kind = failure.read().expect("completed freshness read").kind();
        assert!(if signal {
            matches!(
                kind,
                PhysicalRecoveryCleanupFreshnessReadDenialKind::SignalSettlement(
                    PhysicalSignalSettlementOutcome::DerivedStateUnavailable
                )
            )
        } else {
            matches!(
                kind,
                PhysicalRecoveryCleanupFreshnessReadDenialKind::SchedulerSettlement(
                    PhysicalWorkSchedulerPosture::RejectedAfterEffect
                )
            )
        });
    }
}
