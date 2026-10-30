#[allow(unused_imports)]
use crate::phase_eight_process::production::writer_crash::successor_candidate_admission::*;
use worth_store_physical_format::RecordArtifactFile;
use {
    worth_store::physical_runtime::ArtifactDamage,
    worth_store::physical_runtime::FilesystemObservationBound,
    worth_store::physical_runtime::RecoveryDiscoveryArtifact,
    worth_store::physical_runtime::RecoveryDiscoveryFailure,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryLimitDimension,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial,
    worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial,
    worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateMismatch,
};

use crate::production::writer_crash::manifest_entry_cost;
use crate::production::writer_crash::successor_candidate_cost::candidate_cost;
use {
    crate::history::schedule::in_flight_mutation::MutationCrashWorkload,
    crate::phase_eight_process::production::harness::ProcessWorld,
};
use {
    crate::production::writer_crash::persisted_world::copy_directory,
    crate::production::writer_crash::persisted_world::raw_media_snapshot,
};
use {
    crate::production::writer_crash::recovery_planning::plan_with_limits,
    crate::production::writer_crash::recovery_planning::plan_with_memory,
    crate::production::writer_crash::recovery_planning::successor_limits,
    crate::production::writer_crash::recovery_planning::successor_limits_with_manifest_entries,
    crate::production::writer_crash::recovery_planning::successor_limits_with_observation,
};
use {
    crate::production::writer_crash::successor_candidate_media::candidate_root_path,
    crate::production::writer_crash::successor_candidate_media::mutate_candidate,
    crate::production::writer_crash::successor_candidate_media::remove_candidate_topology,
};

#[test]
fn successor_candidate_denial_is_typed_and_residency_is_admitted_before_allocation() {
    let world = candidate_world(0xC8_09_00_27, 0xC8_19_00_27);
    let generation = world.writer.history.current_root_generation().unwrap() + 1;
    assert_oversized_candidate_is_damage_before_its_read(&world, generation);
    let conflict_root = world.parent_path().join("typed-successor-conflict");
    copy_directory(&world.writer.root, &conflict_root);
    mutate_candidate(&conflict_root, generation, "inflated");
    let blocked = match plan_with_limits(&conflict_root, successor_limits(64 * 1024 * 1024)) {
        Ok(_) => panic!("a semantically conflicting successor must not form a plan"),
        Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
        Err(other) => panic!("successor conflict must block with evidence: {other:?}"),
    };
    assert_eq!(blocked.recovery_effects(), 0);
    let blocked_counters = blocked
        .evidence()
        .root_protocol_counters
        .expect("successor route counters survive semantic conflict");
    assert_eq!(blocked_counters.successor_root_integrity_admissions(), 1);
    assert_eq!(blocked_counters.successor_root_interpretations(), 1);
    assert_eq!(blocked_counters.staged_selector_integrity_admissions(), 0);
    assert_eq!(blocked_counters.closeout_selector_interpretations(), 0);
    assert_eq!(
        blocked.evidence().planning_denial,
        Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(
            worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::Conflict {
                artifact: worth_store_physical_format::RecordArtifactFile::RootManifest { generation },
                generation,
                mismatch: worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateMismatch::RootRoutingFrontier,
            },
        ))
    );

    let independent = candidate_cost(&world.writer.root, generation);
    let candidate_root = world.parent_path().join("candidate-cost-present");
    copy_directory(&world.writer.root, &candidate_root);
    let candidate_plan = plan_with_memory(&candidate_root, 64 * 1024 * 1024).unwrap();
    let candidate_costs = candidate_plan.plan_cost();
    let planning_counters = candidate_plan.planning_counters();
    assert_eq!(
        planning_counters.successor_candidate_reads(),
        independent.reads
    );
    assert_eq!(
        planning_counters.successor_candidate_bytes(),
        independent.raw_bytes
    );
    let counters = candidate_plan.root_protocol_counters();
    assert_eq!(counters.successor_root_integrity_admissions(), 1);
    assert_eq!(counters.successor_root_interpretations(), 1);
    assert_eq!(counters.staged_selector_integrity_admissions(), 1);
    assert_eq!(counters.closeout_selector_interpretations(), 1);
    assert_eq!(
        planning_counters.successor_candidate_peak_bytes(),
        independent.peak_bytes
    );
    let _ = candidate_plan.cancel_before_execution();

    let absent_root = world.parent_path().join("candidate-cost-absent");
    copy_directory(&world.writer.root, &absent_root);
    remove_candidate_topology(&absent_root, generation);
    let absent_plan = plan_with_memory(&absent_root, 64 * 1024 * 1024).unwrap();
    let absent_costs = absent_plan.plan_cost();
    assert_eq!(
        absent_plan.planning_counters().successor_candidate_reads(),
        0
    );
    assert_eq!(
        candidate_costs
            .observation_bytes()
            .checked_sub(absent_costs.observation_bytes()),
        Some(independent.raw_bytes)
    );
    // The raw-media oracle knows this retained candidate storage, not the
    // complete Context/Basis or allocator/conversion overlap. It is a lower
    // bound on the aggregate, not an additive whole-peak allocator oracle.
    let exact_peak = candidate_costs.peak_recovery_bytes();
    assert!(exact_peak >= independent.peak_bytes);
    let exact_observation = absent_costs.observation_bytes() + independent.raw_bytes;
    let _ = absent_plan.cancel_before_execution();
    assert_exact_limits(&world, exact_peak, exact_observation);
}

#[test]
fn successor_candidate_uses_one_cumulative_exact_manifest_entry_limit() {
    let world = candidate_world(0xC8_09_00_67, 0xC8_19_00_67);
    let generation = world.writer.history.current_root_generation().unwrap() + 1;
    let required = crate::production::writer_crash::manifest_entry_cost::required_before_successor(
        &world.writer.root,
    ) + candidate_cost(&world.writer.root, generation).manifest_entries;
    let exact = plan_with_limits(
        &world.writer.root,
        successor_limits_with_manifest_entries(64 * 1024 * 1024, required),
    )
    .expect("the exact raw-media manifest-entry count must be admitted");
    let _ = exact.cancel_before_execution();
    let blocked = match plan_with_limits(
        &world.writer.root,
        successor_limits_with_manifest_entries(64 * 1024 * 1024, required - 1),
    ) {
        Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
        Ok(_) => panic!("one entry below the raw-media requirement must be denied"),
        Err(other) => panic!("manifest-entry admission had wrong outcome: {other:?}"),
    };
    let limit = blocked.cause().limit().expect("denial carries exact limit");
    assert_eq!(
        limit.dimension(),
        worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::ManifestEntries
    );
    assert_eq!(
        (limit.observed(), limit.admitted()),
        (required, required - 1)
    );
}
