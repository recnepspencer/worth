use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial,
    PhysicalRecoverySuccessorCandidateDenial,
};

use super::super::harness::{MutationCrashWorkload, ProcessWorld};
use super::persisted_world::copy_directory;
use super::recovery_planning::{plan_with_limits, successor_limits};
use super::successor_candidate_cost::candidate_cost;
use super::successor_candidate_media::mutate_candidate;

#[test]
fn crc_valid_noncanonical_successor_metadata_is_rejected() {
    let world = candidate_world(0xC8_09_00_47, 0xC8_19_00_47);
    let generation = world.writer.history.current_root_generation().unwrap() + 1;
    for (hostile, expected) in [
        (
            "noncanonical-root",
            RecordArtifactFile::RootManifest { generation },
        ),
        (
            "noncanonical-free-header",
            RecordArtifactFile::FreeSpaceManifest { generation },
        ),
    ] {
        let root = world.parent_path().join(hostile);
        copy_directory(&world.writer.root, &root);
        mutate_candidate(&root, generation, hostile);
        let blocked = blocked_plan(&root);
        assert_eq!(blocked.recovery_effects(), 0);
        match hostile {
            "noncanonical-root" => {
                assert!(
                    matches!(
                        blocked.evidence().planning_denial,
                        Some(PhysicalRecoveryPlanningDenial::SuccessorCandidate(
                            PhysicalRecoverySuccessorCandidateDenial::RootProtocol {
                                artifact,
                                ..
                            }
                        )) if artifact == expected
                    ),
                    "unexpected successor-root denial: {:?}",
                    blocked.evidence().planning_denial
                );
                let counters = blocked.evidence().root_protocol_counters.unwrap();
                assert_eq!(counters.successor_root_integrity_admissions(), 0);
                assert_eq!(counters.successor_root_interpretations(), 0);
            }
            "noncanonical-free-header" => assert!(
                matches!(
                    blocked.evidence().planning_denial,
                    Some(PhysicalRecoveryPlanningDenial::SuccessorCandidate(
                    PhysicalRecoverySuccessorCandidateDenial::RootProtocol {
                        artifact,
                        denial: worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                            worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(localization)
                        ),
                        ..
                    }
                )) if artifact == expected
                    && localization.cause() == worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure
                    && localization.damaged_range() == worth_store_physical_integrity::PhysicalByteRange::new(36, 8).unwrap()
                    && localization.field() == Some(worth_store_physical_integrity::PhysicalFormatField::Reserved)
                ),
                "unexpected successor free-space denial: {:?}",
                blocked.evidence().planning_denial
            ),
            _ => unreachable!(),
        }
    }
}

#[test]
fn failed_successor_observation_reports_each_exact_retained_prefix_peak() {
    let world = candidate_world(0xC8_09_00_57, 0xC8_19_00_57);
    let generation = world.writer.history.current_root_generation().unwrap() + 1;
    let expected = candidate_cost(&world.writer.root, generation).partial_peaks;
    for (hostile, peak) in [
        ("root-routing-child", expected.root_routing),
        ("segment-membership-child", expected.segment_membership),
        ("free-space-child", expected.free_space),
    ] {
        let root = world.parent_path().join(format!("partial-{hostile}-peak"));
        copy_directory(&world.writer.root, &root);
        mutate_candidate(&root, generation, hostile);
        let blocked = blocked_plan(&root);
        let counters = blocked.evidence().planning_counters.unwrap();
        assert!(counters.successor_candidate_reads() > 1);
        assert!(counters.successor_candidate_bytes() > 0);
        assert_eq!(counters.successor_candidate_peak_bytes(), peak);
    }
}

fn candidate_world(schedule: u64, perturbation: u64) -> ProcessWorld {
    ProcessWorld::start_mutation_crash(
        "during-root-publication",
        MutationCrashWorkload::InlineRecord,
        schedule,
        perturbation,
    )
}

fn blocked_plan(root: &std::path::Path) -> worth_store_recovery_runtime::PhysicalRecoveryBlock {
    match plan_with_limits(root, successor_limits(64 * 1024 * 1024)) {
        Err(PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
        Ok(_) => panic!("hostile successor candidate unexpectedly planned"),
        Err(other) => panic!("hostile successor candidate had wrong outcome: {other:?}"),
    }
}
