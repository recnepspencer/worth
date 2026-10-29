pub(crate) use {worth_store::physical_runtime::ArtifactDamage, worth_store::physical_runtime::FilesystemObservationBound, worth_store::physical_runtime::RecoveryDiscoveryArtifact, worth_store::physical_runtime::RecoveryDiscoveryFailure};
pub(crate) use {worth_store_physical_format::RecordArtifactFile};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryLimitDimension, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial, worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial, worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateMismatch};

pub(crate) use {crate::history::schedule::in_flight_mutation::MutationCrashWorkload, crate::phase_eight_process::production::harness::ProcessWorld};
pub(crate) use {crate::production::writer_crash::manifest_entry_cost};
pub(crate) use {crate::production::writer_crash::persisted_world::copy_directory, crate::production::writer_crash::persisted_world::raw_media_snapshot};
pub(crate) use {crate::production::writer_crash::recovery_planning::plan_with_limits, crate::production::writer_crash::recovery_planning::plan_with_memory, crate::production::writer_crash::recovery_planning::successor_limits, crate::production::writer_crash::recovery_planning::successor_limits_with_manifest_entries, crate::production::writer_crash::recovery_planning::successor_limits_with_observation};
pub(crate) use {crate::production::writer_crash::successor_candidate_cost::candidate_cost};
pub(crate) use {crate::production::writer_crash::successor_candidate_media::candidate_root_path, crate::production::writer_crash::successor_candidate_media::mutate_candidate, crate::production::writer_crash::successor_candidate_media::remove_candidate_topology};

/// A candidate root is at most one page. One as long as all the memory
/// recovery admits is damage, not that memory running out: no limit is named,
/// and it is refused on its length, before a byte of it is read or held.
pub(crate) fn assert_oversized_candidate_is_damage_before_its_read(world: &ProcessWorld, generation: u64) {
    const RESIDENT_LIMIT: u64 = 64 << 20;
    let root = world.parent_path().join("candidate-read-oversized");
    copy_directory(&world.writer.root, &root);
    let candidate = candidate_root_path(&root, generation);
    let original_length = std::fs::metadata(&candidate).unwrap().len();
    let original_media = raw_media_snapshot(&root);
    let (_, format) = worth_store_physical_format::DurablePhysicalRootManifest::decode(
        &std::fs::read(&candidate).unwrap(),
        u16::MAX,
    )
    .unwrap();
    let page = u64::from(format.page_size().bytes());
    // Keep the genuine writer's candidate prefix, but make its unselected
    // file as long as the whole resident limit. set_len does not build a
    // 64 MiB test buffer; the production backend must stop at metadata.
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
        Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
        Ok(_) => panic!("oversized candidate must deny before its buffer allocation"),
        Err(other) => panic!("an oversized candidate must block: {other:?}"),
    };
    assert_eq!(blocked.recovery_effects(), 0);
    let evidence = blocked.evidence();
    let Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(
        worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::Discovery {
            artifact,
            generation: denial_generation,
            failure: worth_store::physical_runtime::RecoveryDiscoveryFailure::Damage(damage),
        },
    )) = &evidence.planning_denial
    else {
        panic!(
            "an oversized candidate root is damage: {:?}",
            evidence.planning_denial
        )
    };
    assert_eq!(
        (*artifact, *denial_generation),
        (worth_store_physical_format::RecordArtifactFile::RootManifest { generation }, generation)
    );
    // The length is the file's and the ceiling the root's own, under every
    // budget of this recovery.
    assert_eq!(
        damage,
        &worth_store::physical_runtime::ArtifactDamage::PastCeiling {
            artifact: worth_store::physical_runtime::RecoveryDiscoveryArtifact::Record(*artifact),
            length: RESIDENT_LIMIT,
            ceiling: page,
        }
    );
    assert_eq!(blocked.cause().limit(), None, "damage names no limit");
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

pub(crate) fn assert_exact_limits(world: &ProcessWorld, exact_peak: u64, exact_observation: u64) {
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
        Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
        Err(other) => panic!("memory admission must block: {other:?}"),
    };
    let limit = blocked
        .cause()
        .limit()
        .expect("memory denial carries a limit");
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(raw_media_snapshot(&denied_root), media_before);
    assert!(
        matches!(
            blocked.evidence().planning_denial,
            Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::Cost(
                worth_store_recovery_physics::RecoveryPlanCostDenial::RecoveryMemoryBytes,
            )) | Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(
                worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::RecoveryMemoryBytes { .. },
            ))
        ),
        "must deny at a resident boundary, not malformed media or another limit"
    );
    assert_eq!(
        limit.dimension(),
        worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::RecoveryMemoryBytes
    );
    assert_eq!(
        (limit.observed(), limit.admitted()),
        (exact_peak, exact_peak - 1)
    );
    assert_exact_observation_limit(world, exact_observation);
}

pub(crate) fn assert_exact_observation_limit(world: &ProcessWorld, exact_observation: u64) {
    let root = world.parent_path().join("candidate-observation-one-over");
    copy_directory(&world.writer.root, &root);
    let blocked = match plan_with_limits(
        &root,
        successor_limits_with_observation(64 * 1024 * 1024, exact_observation - 1),
    ) {
        Ok(_) => panic!("one byte below observed successor media must be denied"),
        Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked)) => blocked,
        Err(other) => panic!("observation admission must block: {other:?}"),
    };
    let limit = blocked
        .cause()
        .limit()
        .expect("byte denial carries a limit");
    let Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(
        worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::Discovery {
            failure: worth_store::physical_runtime::RecoveryDiscoveryFailure::Limit(past),
            ..
        },
    )) = blocked.evidence().planning_denial
    else {
        panic!("candidate byte denial must preserve exact backend evidence")
    };
    let (observed, admitted) = (past.observed(), past.admitted());
    assert_eq!(
        past.dimension(),
        worth_store::physical_runtime::FilesystemObservationBound::ObservationBytes
    );
    assert_eq!(
        limit.dimension(),
        worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::ObservationBytes
    );
    assert_eq!(
        (limit.observed(), limit.admitted()),
        (exact_observation, exact_observation - 1)
    );
    assert_eq!(
        (exact_observation - 1 - admitted) + observed,
        exact_observation
    );
}

pub(crate) fn candidate_world(schedule: u64, perturbation: u64) -> ProcessWorld {
    crate::phase_eight_process::production::harness::ProcessWorld::start_mutation_crash(
        "during-root-publication",
        crate::history::schedule::in_flight_mutation::MutationCrashWorkload::ExtentWriteback,
        schedule,
        perturbation,
    )
}