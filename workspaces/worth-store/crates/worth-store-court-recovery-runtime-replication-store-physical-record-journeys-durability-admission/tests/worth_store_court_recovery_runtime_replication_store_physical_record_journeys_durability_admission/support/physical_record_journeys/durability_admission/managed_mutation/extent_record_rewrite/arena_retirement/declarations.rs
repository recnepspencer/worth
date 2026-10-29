pub(crate) use {worth_store_testkit_store::durability_admission::independent_wal_oracle::copy_intent::produced_copy_intents, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction};
pub(crate) use {std::path::Path, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::ServingPhysicalRuntime, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement_crash, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::manifest_fixture::current_extent_route, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request};
pub(crate) use {worth_store::physical_runtime::ArenaEvacuationThreshold, worth_store::physical_runtime::ExtentArenaCapacity, worth_store::physical_runtime::PhysicalArenaEvacuationPreparationOutcome, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordPlacementPolicy};
pub(crate) use {worth_store_physical_format::RecordArtifactFile};

pub(crate) fn arena_disk_bytes(root: &std::path::Path) -> u64 {
    std::fs::read_dir(root.join("families/records/arenas"))
        .unwrap()
        .map(|entry| entry.unwrap().metadata().unwrap().len())
        .sum()
}

pub(crate) fn reopen_with_copy_policy(root: &std::path::Path) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = worth_store_testkit_store::physical_record_journeys::media(root);
    let durability = crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation::fill_durability(&media);
    worth_store_testkit_store::physical_record_journeys::success(media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)))
}

pub(crate) fn reopen_with_small_copy_wal(root: &std::path::Path) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = worth_store_testkit_store::physical_record_journeys::media(root);
    let durability = worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy(
        &media,
        crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement_crash::small_wal_policy(),
    );
    worth_store_testkit_store::physical_record_journeys::success(media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)))
}