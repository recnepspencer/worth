pub(crate) use {worth_store_physical_format::RecordArtifactFile};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial, worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial};

pub(crate) use {crate::history::schedule::in_flight_mutation::MutationCrashWorkload, crate::phase_eight_process::production::harness::ProcessWorld};
pub(crate) use {crate::production::writer_crash::persisted_world::copy_directory};
pub(crate) use {crate::production::writer_crash::recovery_planning::plan_with_limits, crate::production::writer_crash::recovery_planning::successor_limits};
pub(crate) use {crate::production::writer_crash::successor_candidate_cost::candidate_cost};
pub(crate) use {crate::production::writer_crash::successor_candidate_media::mutate_candidate};

pub(crate) fn candidate_world(schedule: u64, perturbation: u64) -> ProcessWorld {
    crate::phase_eight_process::production::harness::ProcessWorld::start_mutation_crash(
        "during-root-publication",
        crate::history::schedule::in_flight_mutation::MutationCrashWorkload::InlineRecord,
        schedule,
        perturbation,
    )
}

pub(crate) fn blocked_plan(root: &std::path::Path) -> worth_store_recovery_runtime::PhysicalRecoveryBlock {
    match plan_with_limits(root, successor_limits(64 * 1024 * 1024)) {
        Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) => *blocked,
        Ok(_) => panic!("hostile successor candidate unexpectedly planned"),
        Err(other) => panic!("hostile successor candidate had wrong outcome: {other:?}"),
    }
}