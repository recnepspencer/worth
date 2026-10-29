pub(crate) use {tempfile::tempdir};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

pub(crate) fn reopen_ignores_external_state(prefix: &str, bytes: &[u8], predicate: &str) {
    let probe = std::env::temp_dir().join(format!("{prefix}-{}", std::process::id()));
    let _ = std::fs::remove_file(&probe);
    let root = tempdir().unwrap();
    let (profile, _, _) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    std::fs::write(&probe, bytes).unwrap();

    let (format, _, access) = worth_store_testkit_store::physical_record_journeys::configuration();
    let reopened = open_record_store!(worth_store_testkit_store::physical_record_journeys::media(root.path()), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            .with_physical_work_profile(profile)
    },)
    .into_raw();
    let _ = std::fs::remove_file(&probe);
    let serving = match reopened {
        worth_proof::TransitionOutcome::Success(serving) => serving,
        _ => panic!(
            "C5_PREDICATE:{predicate}: fresh reopen consumed external derived state instead of physical truth"
        ),
    };
    serving.close();
}