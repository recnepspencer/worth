use std::collections::BTreeSet;

use worth_store_physical_format::{DurablePhysicalRootManifest, RecordArtifactFile};
use worth_store_physical_integrity::PhysicalDamageCause;
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial, PhysicalRecoveryRootProtocolDenial,
    PhysicalRecoverySuccessorCandidateDenial, PhysicalRecoverySuccessorCandidateMismatch,
    RecoveryReportEnvelope, RecoveryReportOutcome,
};

use super::harness::{MutationCrashWorkload, ProcessWorld};

#[path = "writer_crash/capacity_transition.rs"]
mod capacity_transition;
#[path = "writer_crash/manifest_entry_cost.rs"]
mod manifest_entry_cost;
#[path = "writer_crash/persisted_world.rs"]
mod persisted_world;
#[path = "writer_crash/recovery_planning.rs"]
mod recovery_planning;
#[path = "writer_crash/successor_candidate_admission.rs"]
mod successor_candidate_admission;
#[path = "writer_crash/successor_candidate_cost.rs"]
mod successor_candidate_cost;
#[path = "writer_crash/successor_candidate_denials.rs"]
mod successor_candidate_denials;
#[path = "writer_crash/successor_candidate_media.rs"]
mod successor_candidate_media;
#[path = "writer_crash/successor_integrity_expectation.rs"]
mod successor_integrity_expectation;

use persisted_world::{changed_paths, copy_directory, raw_media_snapshot};
use recovery_planning::{
    plan_with_limits, plan_with_memory, successor_limits, successor_limits_with_observation,
};
use successor_candidate_media::{
    candidate_root_path, candidate_topology, hostile_artifact, mutate_candidate,
    selected_root_bytes,
};

const MUTATION_CRASH_SCENARIOS: [(&str, u64, u64); 7] = [
    ("after-group-seal", 0xC8_09_00_02, 0xC8_19_00_02),
    ("after-wal-durability", 0xC8_09_00_03, 0xC8_19_00_03),
    (
        "after-writeback-admission-before-effect",
        0xC8_09_00_04,
        0xC8_19_00_04,
    ),
    ("during-data-settlement", 0xC8_09_00_05, 0xC8_19_00_05),
    ("after-data-settlement", 0xC8_09_00_06, 0xC8_19_00_06),
    ("during-root-publication", 0xC8_09_00_07, 0xC8_19_00_07),
    ("before-terminal-finalization", 0xC8_09_00_08, 0xC8_19_00_08),
];

#[test]
fn killed_writer_recovers_after_each_mutation_effect_boundary() {
    for (stage, schedule_seed, perturbation_seed) in MUTATION_CRASH_SCENARIOS {
        assert_ne!(schedule_seed, perturbation_seed);
        eprintln!(
            "C8 mutation crash stage={stage} schedule={schedule_seed} perturbation={perturbation_seed}"
        );
        let world = ProcessWorld::start_mutation_crash(
            stage,
            MutationCrashWorkload::ExtentWriteback,
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
        super::harness::compare_runtime_and_observer_with_budget(
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
            super::harness::compare_runtime_and_observer_with_budget(
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
            let process_ids = BTreeSet::from([
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
    let world = ProcessWorld::start_mutation_crash(
        "during-root-publication",
        MutationCrashWorkload::InlineRecord,
        0xC8_09_00_17,
        0xC8_19_00_17,
    );
    let selected_generation = world
        .writer
        .history
        .current_root_generation()
        .expect("writer leaves a selected current root");
    let candidate_generation = selected_generation + 1;

    for hostile in [
        "malformed",
        "conflicting",
        "inflated",
        "root-routing-child",
        "segment-membership-child",
        "free-space-child",
        "selected-routing-root",
    ] {
        let root = world.parent_path().join(format!("hostile-{hostile}"));
        copy_directory(&world.writer.root, &root);
        let expected = expected_hostile_denial(&root, candidate_generation, hostile);
        mutate_candidate(&root, candidate_generation, hostile);
        let media_before = raw_media_snapshot(&root);

        let blocked = match plan_with_limits(&root, successor_limits(64 * 1024 * 1024)) {
            Err(PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
            Ok(_) => panic!("hostile successor candidate unexpectedly planned"),
            Err(other) => panic!("hostile successor candidate had wrong outcome: {other:?}"),
        };
        assert_eq!(blocked.recovery_effects(), 0);
        if hostile == "malformed" {
            assert!(matches!(
                blocked.evidence().planning_denial,
                Some(PhysicalRecoveryPlanningDenial::SuccessorCandidate(
                    PhysicalRecoverySuccessorCandidateDenial::RootProtocol {
                        artifact: RecordArtifactFile::RootManifest { generation: observed },
                        generation: denied,
                        denial: PhysicalRecoveryRootProtocolDenial::Integrity(
                            worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(
                                localization
                            )
                        ),
                    }
                )) if observed == candidate_generation
                    && denied == candidate_generation
                    && localization.cause() == PhysicalDamageCause::WrongMagic
            ));
        } else {
            successor_integrity_expectation::require(
                blocked.evidence().planning_denial.clone(),
                expected,
                hostile,
            );
        }

        let report_path = world
            .parent_path()
            .join(format!("hostile-{hostile}.report"));
        let (_, output) = super::harness::run_recovery_with_profile(
            &root,
            &report_path,
            world.parent_path(),
            "c8-phase8-fate-coverage-v1",
        );
        assert!(
            !output.status.success(),
            "blocked recovery must not report success"
        );
        let report = RecoveryReportEnvelope::decode(
            &std::fs::read(report_path).expect("read hostile recovery report"),
        )
        .expect("decode hostile recovery report");
        assert_eq!(report.outcome(), RecoveryReportOutcome::Blocked);
        assert_eq!(report.counters().recovery_effects(), 0);
        let media_after = raw_media_snapshot(&root);
        assert!(
            media_after == media_before,
            "hostile successor denial changed persisted paths: {:?}",
            changed_paths(&media_before, &media_after)
        );
    }
}

fn expected_hostile_denial(
    root: &std::path::Path,
    generation: u64,
    hostile: &str,
) -> PhysicalRecoveryPlanningDenial {
    let artifact = hostile_artifact(root, generation, hostile);
    let denial = match hostile {
        "conflicting" => {
            let bytes = std::fs::read(candidate_root_path(root, generation)).unwrap();
            let (candidate, _) = DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
            let reference = candidate.routing_root().expect("candidate routing root");
            PhysicalRecoverySuccessorCandidateDenial::InvalidArtifact {
                artifact: RecordArtifactFile::RootRoutingBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                generation,
            }
        }
        "inflated" => PhysicalRecoverySuccessorCandidateDenial::Conflict {
            artifact,
            generation,
            mismatch: PhysicalRecoverySuccessorCandidateMismatch::RootRoutingFrontier,
        },
        "selected-routing-root" => PhysicalRecoverySuccessorCandidateDenial::Conflict {
            artifact,
            generation,
            mismatch: PhysicalRecoverySuccessorCandidateMismatch::RecordPlacements,
        },
        _ => PhysicalRecoverySuccessorCandidateDenial::InvalidArtifact {
            artifact,
            generation,
        },
    };
    PhysicalRecoveryPlanningDenial::SuccessorCandidate(denial)
}

#[test]
fn recovery_without_pending_publication_does_not_observe_a_successor_candidate() {
    let world = ProcessWorld::start_mutation_crash(
        "after-group-seal",
        MutationCrashWorkload::ExtentWriteback,
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
