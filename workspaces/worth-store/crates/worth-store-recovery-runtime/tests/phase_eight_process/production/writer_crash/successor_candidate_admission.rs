use worth_store::physical_runtime::{RecoveryDiscoveryByteLimitScope, RecoveryDiscoveryFailure};
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome, PhysicalRecoveryPlanningDenial,
    PhysicalRecoverySuccessorCandidateDenial, PhysicalRecoverySuccessorCandidateMismatch,
};

use super::super::harness::{MutationCrashWorkload, ProcessWorld};
use super::manifest_entry_cost;
use super::persisted_world::{copy_directory, raw_media_snapshot};
use super::recovery_planning::{
    plan_with_limits, plan_with_memory, successor_limits, successor_limits_with_manifest_entries,
    successor_limits_with_observation,
};
use super::successor_candidate_cost::candidate_cost;
use super::successor_candidate_media::{
    candidate_root_path, mutate_candidate, remove_candidate_topology,
};

#[test]
fn successor_candidate_denial_is_typed_and_residency_is_admitted_before_allocation() {
    let world = candidate_world(0xC8_09_00_27, 0xC8_19_00_27);
    let generation = world.writer.history.current_root_generation().unwrap() + 1;
    assert_resident_denial_before_candidate_read(&world, generation);
    let conflict_root = world.parent_path().join("typed-successor-conflict");
    copy_directory(&world.writer.root, &conflict_root);
    mutate_candidate(&conflict_root, generation, "inflated");
    let blocked = match plan_with_limits(&conflict_root, successor_limits(64 * 1024 * 1024)) {
        Ok(_) => panic!("a semantically conflicting successor must not form a plan"),
        Err(PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
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
        Some(PhysicalRecoveryPlanningDenial::SuccessorCandidate(
            PhysicalRecoverySuccessorCandidateDenial::Conflict {
                artifact: RecordArtifactFile::RootManifest { generation },
                generation,
                mismatch: PhysicalRecoverySuccessorCandidateMismatch::RootRoutingFrontier,
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

fn assert_resident_denial_before_candidate_read(world: &ProcessWorld, generation: u64) {
    const RESIDENT_LIMIT: u64 = 64 << 20;
    let root = world.parent_path().join("candidate-read-resident-denied");
    copy_directory(&world.writer.root, &root);
    let candidate = candidate_root_path(&root, generation);
    let original_length = std::fs::metadata(&candidate).unwrap().len();
    let original_media = raw_media_snapshot(&root);
    // Keep the genuine writer's candidate prefix, but make its unselected
    // file larger than the remaining resident window. set_len does not build
    // a 64 MiB test buffer; the production backend must stop at metadata.
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&candidate)
        .unwrap();
    file.set_len(RESIDENT_LIMIT).unwrap();
    drop(file);
    let blocked = match plan_with_limits(
        &root,
        successor_limits_with_observation(RESIDENT_LIMIT, 128 << 20),
    ) {
        Err(PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
        Ok(_) => panic!("oversized candidate must deny before its buffer allocation"),
        Err(other) => panic!("candidate resident admission must block: {other:?}"),
    };
    assert_eq!(blocked.recovery_effects(), 0);
    let evidence = blocked.evidence();
    assert!(matches!(evidence.planning_denial, Some(
        PhysicalRecoveryPlanningDenial::SuccessorCandidate(
            PhysicalRecoverySuccessorCandidateDenial::RecoveryMemoryBytes {
                artifact: RecordArtifactFile::RootManifest { generation: observed_generation },
                generation: denial_generation,
                observed,
                admitted: RESIDENT_LIMIT,
            }
        )
    ) if observed_generation == generation && denial_generation == generation && observed > RESIDENT_LIMIT));
    let limit = evidence.limit.unwrap();
    assert_eq!(
        limit.dimension,
        PhysicalRecoveryLimitDimension::RecoveryMemoryBytes
    );
    assert_eq!(limit.admitted, RESIDENT_LIMIT);
    let counters = evidence.planning_counters.unwrap();
    assert_eq!(counters.successor_candidate_reads(), 0);
    assert_eq!(counters.successor_candidate_bytes(), 0);
    assert_eq!(
        evidence
            .root_protocol_counters
            .unwrap()
            .successor_root_integrity_admissions(),
        0
    );
    assert_eq!(std::fs::metadata(&candidate).unwrap().len(), RESIDENT_LIMIT);
    // Restore only fixture padding and compare all bytes, including WAL and
    // selected namespace, with the genuine writer's unchanged starting state.
    std::fs::OpenOptions::new()
        .write(true)
        .open(&candidate)
        .unwrap()
        .set_len(original_length)
        .unwrap();
    assert_eq!(raw_media_snapshot(&root), original_media);
}

fn assert_exact_limits(world: &ProcessWorld, exact_peak: u64, exact_observation: u64) {
    let exact_root = world.parent_path().join("candidate-memory-exact");
    copy_directory(&world.writer.root, &exact_root);
    let exact = plan_with_limits(
        &exact_root,
        successor_limits_with_observation(exact_peak, exact_observation),
    )
    .expect("the exact candidate-inclusive peak must be admitted");
    assert_eq!(exact.plan_cost().peak_recovery_bytes(), exact_peak);
    assert_eq!(exact.plan_cost().observation_bytes(), exact_observation);
    let _ = exact.cancel_before_execution();

    let denied_root = world.parent_path().join("candidate-memory-one-over");
    copy_directory(&world.writer.root, &denied_root);
    let media_before = raw_media_snapshot(&denied_root);
    let blocked = match plan_with_memory(&denied_root, exact_peak - 1) {
        Ok(_) => panic!("one byte below the candidate-inclusive peak must be denied"),
        Err(PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
        Err(other) => panic!("memory admission must block: {other:?}"),
    };
    let limit = blocked
        .evidence()
        .limit
        .expect("memory denial carries a limit");
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(raw_media_snapshot(&denied_root), media_before);
    assert!(
        matches!(
            blocked.evidence().planning_denial,
            Some(PhysicalRecoveryPlanningDenial::Cost(
                worth_store_recovery_physics::RecoveryPlanCostDenial::RecoveryMemoryBytes,
            )) | Some(PhysicalRecoveryPlanningDenial::SuccessorCandidate(
                PhysicalRecoverySuccessorCandidateDenial::RecoveryMemoryBytes { .. },
            ))
        ),
        "must deny at a resident boundary, not malformed media or another limit"
    );
    assert_eq!(
        limit.dimension,
        PhysicalRecoveryLimitDimension::RecoveryMemoryBytes
    );
    assert_eq!(
        (limit.observed, limit.admitted),
        (exact_peak, exact_peak - 1)
    );
    assert_exact_observation_limit(world, exact_observation);
}

fn assert_exact_observation_limit(world: &ProcessWorld, exact_observation: u64) {
    let root = world.parent_path().join("candidate-observation-one-over");
    copy_directory(&world.writer.root, &root);
    let blocked = match plan_with_limits(
        &root,
        successor_limits_with_observation(64 * 1024 * 1024, exact_observation - 1),
    ) {
        Ok(_) => panic!("one byte below observed successor media must be denied"),
        Err(PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
        Err(other) => panic!("observation admission must block: {other:?}"),
    };
    let limit = blocked
        .evidence()
        .limit
        .expect("byte denial carries a limit");
    let Some(PhysicalRecoveryPlanningDenial::SuccessorCandidate(
        PhysicalRecoverySuccessorCandidateDenial::Discovery {
            failure:
                RecoveryDiscoveryFailure::ByteLimitExceeded {
                    observed,
                    admitted,
                    scope,
                },
            ..
        },
    )) = blocked.evidence().planning_denial
    else {
        panic!("candidate byte denial must preserve exact backend evidence")
    };
    assert_eq!(scope, RecoveryDiscoveryByteLimitScope::Observation);
    assert_eq!(
        limit.dimension,
        PhysicalRecoveryLimitDimension::ObservationBytes
    );
    assert_eq!(
        (limit.observed, limit.admitted),
        (exact_observation, exact_observation - 1)
    );
    assert_eq!(
        (exact_observation - 1 - admitted) + observed,
        exact_observation
    );
}

#[test]
fn successor_candidate_uses_one_cumulative_exact_manifest_entry_limit() {
    let world = candidate_world(0xC8_09_00_67, 0xC8_19_00_67);
    let generation = world.writer.history.current_root_generation().unwrap() + 1;
    let required = manifest_entry_cost::required_before_successor(&world.writer.root)
        + candidate_cost(&world.writer.root, generation).manifest_entries;
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
        Err(PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
        Ok(_) => panic!("one entry below the raw-media requirement must be denied"),
        Err(other) => panic!("manifest-entry admission had wrong outcome: {other:?}"),
    };
    let limit = blocked
        .evidence()
        .limit
        .expect("denial carries exact limit");
    assert_eq!(
        limit.dimension,
        PhysicalRecoveryLimitDimension::ManifestEntries
    );
    assert_eq!((limit.observed, limit.admitted), (required, required - 1));
}

fn candidate_world(schedule: u64, perturbation: u64) -> ProcessWorld {
    ProcessWorld::start_mutation_crash(
        "during-root-publication",
        MutationCrashWorkload::ExtentWriteback,
        schedule,
        perturbation,
    )
}
