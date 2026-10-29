pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use {worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::produced_rewrite_payloads};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
pub(crate) use {worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

/// Every media role that can change bytes or the namespace.
pub(crate) const EFFECT_ROLES: [MediaOperationRole; 7] = [
    worth_store_physical_backend::MediaOperationRole::CreateNew,
    worth_store_physical_backend::MediaOperationRole::PositionedWrite,
    worth_store_physical_backend::MediaOperationRole::Append,
    worth_store_physical_backend::MediaOperationRole::Truncate,
    worth_store_physical_backend::MediaOperationRole::Allocate,
    worth_store_physical_backend::MediaOperationRole::AtomicReplace,
    worth_store_physical_backend::MediaOperationRole::Delete,
];

pub(crate) fn published_source(root: &Path) -> PathBuf {
    let serving = serving_from_initialization(root);
    let (_, placement, _) = configuration();
    match prepare(&serving, placement, [61; 32], b"damaged-rewrite-source").execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {}
        _ => panic!("the source append did not complete"),
    }
    serving.close();
    let mut segments = segment_files(root);
    assert_eq!(segments.len(), 1, "one append publishes one segment");
    segments.pop().unwrap()
}

pub(crate) fn segment_files(root: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect()
}

pub(crate) fn baseline(serving: &ServingPhysicalRuntime) -> (u64, u64) {
    let counters = serving.media_counters();
    (
        EFFECT_ROLES
            .iter()
            .map(|role| counters.attempts_for(*role))
            .sum(),
        serving
            .resident_admission_counters()
            .refusals_before_owner_entry(),
    )
}