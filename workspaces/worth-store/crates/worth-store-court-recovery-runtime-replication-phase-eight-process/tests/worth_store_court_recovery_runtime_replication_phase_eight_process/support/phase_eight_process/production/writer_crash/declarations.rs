pub(crate) use std::collections::BTreeSet;

pub(crate) use worth_store_physical_integrity::PhysicalDamageCause;
pub(crate) use {
    worth_store_physical_format::DurablePhysicalRootManifest,
    worth_store_physical_format::RecordArtifactFile,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial,
    worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial,
    worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateMismatch,
    worth_store_recovery_runtime::RecoveryReportEnvelope,
    worth_store_recovery_runtime::RecoveryReportOutcome,
};

pub(crate) use {
    crate::history::schedule::in_flight_mutation::MutationCrashWorkload,
    crate::phase_eight_process::production::harness::ProcessWorld,
};

pub(crate) use {
    crate::production::writer_crash::persisted_world::changed_paths,
    crate::production::writer_crash::persisted_world::copy_directory,
    crate::production::writer_crash::persisted_world::raw_media_snapshot,
};
pub(crate) use {
    crate::production::writer_crash::recovery_planning::plan_with_limits,
    crate::production::writer_crash::recovery_planning::plan_with_memory,
    crate::production::writer_crash::recovery_planning::successor_limits,
    crate::production::writer_crash::recovery_planning::successor_limits_with_observation,
};
pub(crate) use {
    crate::production::writer_crash::successor_candidate_media::candidate_root_path,
    crate::production::writer_crash::successor_candidate_media::candidate_topology,
    crate::production::writer_crash::successor_candidate_media::hostile_artifact,
    crate::production::writer_crash::successor_candidate_media::mutate_candidate,
    crate::production::writer_crash::successor_candidate_media::selected_root_bytes,
};

pub(crate) const MUTATION_CRASH_SCENARIOS: [(&str, u64, u64); 7] = [
    ("after-group-seal", 0xC8_09_00_02, 0xC8_19_00_02),
    ("after-wal-durability", 0xC8_09_00_03, 0xC8_19_00_03),
    (
        "after-arena-write-admission-before-effect",
        0xC8_09_00_04,
        0xC8_19_00_04,
    ),
    ("during-data-settlement", 0xC8_09_00_05, 0xC8_19_00_05),
    ("after-data-settlement", 0xC8_09_00_06, 0xC8_19_00_06),
    ("during-root-publication", 0xC8_09_00_07, 0xC8_19_00_07),
    ("before-terminal-finalization", 0xC8_09_00_08, 0xC8_19_00_08),
];

pub(crate) fn expected_hostile_denial(
    root: &std::path::Path,
    generation: u64,
    hostile: &str,
) -> PhysicalRecoveryPlanningDenial {
    let artifact = hostile_artifact(root, generation, hostile);
    let denial = match hostile {
        "conflicting" => {
            let bytes = std::fs::read(candidate_root_path(root, generation)).unwrap();
            let (candidate, _) = worth_store_physical_format::DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
            let reference = candidate.routing_root().expect("candidate routing root");
            worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::InvalidArtifact {
                artifact: worth_store_physical_format::RecordArtifactFile::RootRoutingBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                generation,
            }
        }
        "inflated" => worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::Conflict {
            artifact,
            generation,
            mismatch: worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateMismatch::RootRoutingFrontier,
        },
        "selected-routing-root" => worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::Conflict {
            artifact,
            generation,
            mismatch: worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateMismatch::RecordPlacements,
        },
        _ => worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::InvalidArtifact {
            artifact,
            generation,
        },
    };
    worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(denial)
}
