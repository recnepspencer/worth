#[allow(unused_imports)]
use crate::phase_eight_process::production::writer_crash::*;
use std::collections::BTreeSet;

use worth_store_physical_integrity::PhysicalDamageCause;
use {
    worth_store_physical_format::DurablePhysicalRootManifest,
    worth_store_physical_format::RecordArtifactFile,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial,
    worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial,
    worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateMismatch,
    worth_store_recovery_runtime::RecoveryReportEnvelope,
    worth_store_recovery_runtime::RecoveryReportOutcome,
};

use {
    crate::history::schedule::in_flight_mutation::MutationCrashWorkload,
    crate::phase_eight_process::production::harness::ProcessWorld,
};

use {
    crate::production::writer_crash::persisted_world::changed_paths,
    crate::production::writer_crash::persisted_world::copy_directory,
    crate::production::writer_crash::persisted_world::raw_media_snapshot,
};
use {
    crate::production::writer_crash::recovery_planning::plan_with_limits,
    crate::production::writer_crash::recovery_planning::plan_with_memory,
    crate::production::writer_crash::recovery_planning::successor_limits,
    crate::production::writer_crash::recovery_planning::successor_limits_with_observation,
};
use {
    crate::production::writer_crash::successor_candidate_media::candidate_root_path,
    crate::production::writer_crash::successor_candidate_media::candidate_topology,
    crate::production::writer_crash::successor_candidate_media::hostile_artifact,
    crate::production::writer_crash::successor_candidate_media::mutate_candidate,
    crate::production::writer_crash::successor_candidate_media::selected_root_bytes,
};

#[test]
fn killed_writer_recovers_after_each_mutation_effect_boundary() {
    for (stage, schedule_seed, perturbation_seed) in MUTATION_CRASH_SCENARIOS {
        assert_ne!(schedule_seed, perturbation_seed);
        let world = crate::phase_eight_process::production::harness::ProcessWorld::start_mutation_crash(
            stage,
            crate::history::schedule::in_flight_mutation::MutationCrashWorkload::ExtentWriteback,
            schedule_seed,
            perturbation_seed,
        );
        let dead_observer = world.observe(&format!("{stage}-dead"));
        world
            .writer
            .history
            .compare_report(&dead_observer.report)
            .unwrap_or_else(|error| {
                panic!(
                    "mutation crash stage {stage} schedule {schedule_seed} perturbation {perturbation_seed} dead-byte disagreement: {error:?}"
                )
            });

        let candidate_generation = world
            .writer
            .history
            .current_root_generation()
            .expect("writer leaves a selected current root")
            + 1;
        let candidate_before = (stage == "during-root-publication")
            .then(|| candidate_topology(&world.writer.root, candidate_generation));
        let runtime = world.recover_root_with_profile(
            &world.writer.root,
            &format!("{stage}-recovery"),
            "c8-phase8-fate-coverage-v1",
        );
        let observer = world.observe(&format!("{stage}-reopened"));
        let history = world.parent_history_after_recovery(&world.writer.root);
        crate::phase_eight_process::production::harness::compare_runtime_and_observer_with_budget(
            &runtime,
            &observer,
            &history,
            4 * 1024 * 1024,
        );

        assert_ne!(world.writer.process_id, dead_observer.process_id);
        assert_ne!(world.writer.process_id, runtime.process_id);
        assert_ne!(runtime.process_id, observer.process_id);
        if stage == "during-root-publication" {
            assert_eq!(
                candidate_before.as_ref().unwrap(),
                &candidate_topology(&world.writer.root, candidate_generation),
                "first recovery must publish the writer's exact immutable candidate bytes"
            );
            let second = world.recover_root_with_profile(
                &world.writer.root,
                &format!("{stage}-second-recovery"),
                "c8-phase8-fate-coverage-v1",
            );
            let second_observer = world.observe(&format!("{stage}-second-reopened"));
            let second_history = world.parent_history_after_recovery(&world.writer.root);
            crate::phase_eight_process::production::harness::compare_runtime_and_observer_with_budget(
                &second,
                &second_observer,
                &second_history,
                4 * 1024 * 1024,
            );
            assert_eq!(
                candidate_before.as_ref().unwrap(),
                &candidate_topology(&world.writer.root, candidate_generation),
                "fresh reopen must preserve the adopted candidate topology byte-for-byte"
            );
            let process_ids = std::collections::BTreeSet::from([
                world.writer.process_id,
                dead_observer.process_id,
                runtime.process_id,
                observer.process_id,
                second.process_id,
                second_observer.process_id,
            ]);
            assert_eq!(
                process_ids.len(),
                6,
                "every critical role must be a fresh process"
            );
        }
    }
}

#[test]
fn hostile_successor_candidates_block_before_recovery_effects() {
    let inline =
        crate::phase_eight_process::production::harness::ProcessWorld::start_mutation_crash(
            "during-root-publication",
            crate::history::schedule::in_flight_mutation::MutationCrashWorkload::InlineRecord,
            0xC8_09_00_17,
            0xC8_19_00_17,
        );
    // Reusing the selected routing root needs a last inline record inside its
    // range. Production record identities are random, so a new inline record
    // can fall outside it; an extent mutation keeps the selected inline tail.
    let extent =
        crate::phase_eight_process::production::harness::ProcessWorld::start_mutation_crash(
            "during-root-publication",
            crate::history::schedule::in_flight_mutation::MutationCrashWorkload::ExtentWriteback,
            0xC8_09_00_18,
            0xC8_19_00_18,
        );

    for (hostile, world) in [
        ("malformed", &inline),
        ("conflicting", &inline),
        ("inflated", &inline),
        ("root-routing-child", &inline),
        ("segment-membership-child", &inline),
        ("free-space-child", &inline),
        ("selected-routing-root", &extent),
    ] {
        let candidate_generation = world
            .writer
            .history
            .current_root_generation()
            .expect("writer leaves a selected current root")
            + 1;
        let root = world.parent_path().join(format!("hostile-{hostile}"));
        copy_directory(&world.writer.root, &root);
        let expected = expected_hostile_denial(&root, candidate_generation, hostile);
        mutate_candidate(&root, candidate_generation, hostile);
        let media_before = raw_media_snapshot(&root);

        let blocked = match plan_with_limits(&root, successor_limits(64 * 1024 * 1024)) {
            Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
            Ok(_) => panic!("hostile successor candidate unexpectedly planned"),
            Err(other) => panic!("hostile successor candidate had wrong outcome: {other:?}"),
        };
        assert_eq!(blocked.recovery_effects(), 0);
        if hostile == "malformed" {
            assert!(matches!(
                blocked.evidence().planning_denial,
                Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(
                    worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::RootProtocol {
                        artifact: worth_store_physical_format::RecordArtifactFile::RootManifest { generation: observed },
                        generation: denied,
                        denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                            worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(
                                localization
                            )
                        ),
                    }
                )) if observed == candidate_generation
                    && denied == candidate_generation
                    && localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::WrongMagic
            ));
        } else {
            crate::production::writer_crash::successor_integrity_expectation::require(
                blocked.evidence().planning_denial.clone(),
                expected,
                hostile,
            );
        }

        let report_path = world
            .parent_path()
            .join(format!("hostile-{hostile}.report"));
        let (_, output) =
            crate::phase_eight_process::production::harness::process::run_recovery_with_profile(
                &root,
                &report_path,
                world.parent_path(),
                "c8-phase8-fate-coverage-v1",
            );
        assert!(
            !output.status.success(),
            "blocked recovery must not report success"
        );
        let report_bytes = std::fs::read(&report_path).unwrap_or_else(|error| {
            panic!(
                "read hostile recovery report for {hostile}: {error}; child status={}; stdout={}; stderr={}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            )
        });
        let report = worth_store_recovery_runtime::RecoveryReportEnvelope::decode(&report_bytes)
            .expect("decode hostile recovery report");
        assert_eq!(
            report.outcome(),
            worth_store_recovery_runtime::RecoveryReportOutcome::Blocked
        );
        assert_eq!(report.counters().recovery_effects(), 0);
        let media_after = raw_media_snapshot(&root);
        assert!(
            media_after == media_before,
            "hostile successor denial changed persisted paths: {:?}",
            changed_paths(&media_before, &media_after)
        );
    }
}

#[test]
fn recovery_without_pending_publication_does_not_observe_a_successor_candidate() {
    let world = crate::phase_eight_process::production::harness::ProcessWorld::start_mutation_crash(
        "after-group-seal",
        crate::history::schedule::in_flight_mutation::MutationCrashWorkload::ExtentWriteback,
        0xC8_09_00_37,
        0xC8_19_00_37,
    );
    let _ = world.recover_root_with_profile(
        &world.writer.root,
        "settle-before-no-publication-proof",
        "c8-phase8-fate-coverage-v1",
    );
    let settled = world.parent_history_after_recovery(&world.writer.root);
    let selected_generation = settled.current_root_generation().unwrap();
    let selected_before = selected_root_bytes(&world.writer.root, selected_generation);
    let baseline = plan_with_memory(&world.writer.root, 64 * 1024 * 1024)
        .expect("settled recovery baseline plans without a successor");
    let baseline_observation = baseline.plan_cost().observation_bytes();
    assert_eq!(baseline.planning_counters().successor_candidate_reads(), 0);
    let _ = baseline.cancel_before_execution();
    let successor = candidate_root_path(&world.writer.root, selected_generation + 1);
    std::fs::write(successor, [0_u8; 32]).expect("write irrelevant malformed successor");

    let planned = plan_with_limits(
        &world.writer.root,
        successor_limits_with_observation(64 * 1024 * 1024, baseline_observation),
    )
    .expect("no pending projection must not inspect successor residue");
    assert_eq!(
        planned.plan_cost().observation_bytes(),
        baseline_observation
    );
    assert_eq!(planned.planning_counters().successor_candidate_reads(), 0);
    assert_eq!(planned.planning_counters().successor_candidate_bytes(), 0);
    assert_eq!(
        planned.planning_counters().successor_candidate_peak_bytes(),
        0
    );
    let _ = planned.cancel_before_execution();
    assert_eq!(
        selected_root_bytes(&world.writer.root, selected_generation),
        selected_before,
        "effect-free planning preserves selected authority"
    );
}
#[path = "capacity_transition/mod.rs"]
pub(crate) mod capacity_transition;
#[path = "rewrite_recovery/mod.rs"]
pub(crate) mod rewrite_recovery;
#[path = "successor_candidate_admission/mod.rs"]
pub(crate) mod successor_candidate_admission;
#[path = "successor_candidate_denials/mod.rs"]
pub(crate) mod successor_candidate_denials;
