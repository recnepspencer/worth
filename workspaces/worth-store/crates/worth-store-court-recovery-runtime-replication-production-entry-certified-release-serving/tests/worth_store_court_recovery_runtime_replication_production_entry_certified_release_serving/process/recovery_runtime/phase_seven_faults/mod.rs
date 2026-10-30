#![cfg(all())]

use {
    crate::phase_seven_faults::world::assert_child_succeeded,
    crate::phase_seven_faults::world::cleanup_fault,
    crate::phase_seven_faults::world::cleanup_world,
    crate::phase_seven_faults::world::empty_fault_schedule,
    crate::phase_seven_faults::world::paused_cleanup_schedule,
    crate::phase_seven_faults::world::recover_with_schedule,
    crate::phase_seven_faults::world::reopen_with_schedule,
    crate::phase_seven_faults::world::required_child_root,
    crate::phase_seven_faults::world::required_crash_marker,
    crate::phase_seven_faults::world::run_child,
    crate::phase_seven_faults::world::spawn_crashing_child,
};

#[test]
fn killed_cleanup_after_the_deletion_effect_reopens_in_a_fresh_process() {
    let world = cleanup_world("cleanup-crash-restart");
    let candidate = world.oldest_wal();
    let marker = world.root.parent().unwrap().join("cleanup-effect-reached");
    let mut first = spawn_crashing_child(
        "process::recovery_runtime::phase_seven_faults::child_process::killed_cleanup_process",
        &world.root,
        &marker,
    );
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
    let second = run_child(
        "process::recovery_runtime::phase_seven_faults::child_process::reopen_after_cleanup_process",
        &world.root,
    );
    assert_child_succeeded("post-cleanup recovery", &second);
}

#[test]
fn ambiguous_cleanup_effect_is_deferred_without_performed_authority() {
    let world = cleanup_world("cleanup-indeterminate");
    let candidate = world.oldest_wal();
    let handoff = recover_with_schedule(
        &world.root,
        cleanup_fault(worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect),
    );
    let worth_store_recovery_runtime::RecoveryCleanupPosture::Deferred(evidence) =
        handoff.cleanup_posture()
    else {
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
        [worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence::IndeterminateEffect { .. }]
    ));
    assert!(evidence.dispositions().iter().any(|disposition| {
        disposition.kind()
            == worth_store_recovery_runtime::RecoveryCleanupDispositionKind::Deferred(
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
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        reopened.finish()
    else {
        panic!("stale cleanup plan remains maintenance debt, not recovery failure")
    };
    let worth_store_recovery_runtime::RecoveryCleanupPosture::Deferred(evidence) =
        handoff.cleanup_posture()
    else {
        panic!("generation shift defers cleanup")
    };
    assert!(candidate.exists());
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().freshness_evaluations, 1);
    assert_eq!(evidence.counters().actions_attempted, 0);
    assert!(matches!(
        evidence.deferrals(),
        [worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence::PublishedGenerationChanged {
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
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        reopened.finish()
    else {
        panic!("cleanup-plan denial remains deferred maintenance")
    };
    let worth_store_recovery_runtime::RecoveryCleanupPosture::Deferred(evidence) =
        handoff.cleanup_posture()
    else {
        panic!("cleanup-plan denial must be retained")
    };
    assert!(candidate.exists());
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().freshness_evaluations, 1);
    assert_eq!(evidence.counters().freshness_reads_completed, 0);
    assert_eq!(evidence.counters().freshness_bytes_read, 0);
    assert_eq!(evidence.counters().scheduler_commands_submitted, 0);
    assert_eq!(evidence.counters().actions_attempted, 0);
    let [worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence::Freshness {
        failure, ..
    }] = evidence.deferrals()
    else {
        panic!("one exact Store cleanup-plan admission denial")
    };
    assert_eq!(
        failure.denial(),
        worth_store::physical_runtime::StoreRecoveryCleanupFreshnessDenial::InvalidCleanupEligibility
    );
    assert!(failure.sample().is_none());
    assert!(failure.read().is_none());
}

#[test]
fn cleanup_media_handle_leak_blocks_the_recovered_handoff() {
    let world = cleanup_world("cleanup-media-handle-leak");
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    reopened.certification_leak_cleanup_media_handle();
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        indeterminate,
    ) = reopened.finish()
    else {
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
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        reopened.finish()
    else {
        panic!("post-read eligibility debt cannot invalidate recovery")
    };
    let worth_store_recovery_runtime::RecoveryCleanupPosture::Deferred(evidence) =
        handoff.cleanup_posture()
    else {
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
    let [worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence::Freshness {
        failure, ..
    }] = evidence.deferrals()
    else {
        panic!("one exact post-read freshness failure")
    };
    assert_eq!(
        failure.denial(),
        worth_store::physical_runtime::StoreRecoveryCleanupFreshnessDenial::InvalidCleanupEligibility
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
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        second_reopened.finish_with_cleanup_cancellation(cancellation)
    else {
        panic!("foreign cleanup cancellation remains optional deferred work")
    };
    assert!(second_candidate.exists());
    assert!(matches!(
        handoff.cleanup_posture().evidence().deferrals(),
        [worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence::CancellationBindingMismatch { .. }]
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

#[cfg(all())]
#[path = "revalidation/mod.rs"]
pub(crate) mod revalidation;

#[path = "child_process/mod.rs"]
pub(crate) mod child_process;

#[path = "scheduler_admission/mod.rs"]
pub(crate) mod scheduler_admission;
