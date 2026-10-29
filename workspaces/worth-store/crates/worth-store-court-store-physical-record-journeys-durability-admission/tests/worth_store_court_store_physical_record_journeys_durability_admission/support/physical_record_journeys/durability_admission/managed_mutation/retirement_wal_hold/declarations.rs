pub(crate) use {std::num::NonZeroU32, std::num::NonZeroU64};
pub(crate) use {std::path::Path};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::TemporalDuration};
pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::file_retirement_payloads};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::retirement_charge_oracle::assert_one_page_released_net_of_wal, worth_store_testkit_store::physical_record_journeys::retirement_charge_oracle::wal_bytes};

pub(crate) const SEGMENT_BYTES: u64 = 35_268;

pub(crate) fn wal_policy() -> PhysicalWalPolicy {
    worth_store::physical_runtime::PhysicalWalPolicy::segmented(
        worth_store::physical_runtime::WalSegmentByteLimit::new(std::num::NonZeroU64::new(SEGMENT_BYTES).unwrap()),
        worth_store::physical_runtime::WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(16).unwrap()),
    )
}

pub(crate) fn open_initialized(root: &Path, wal: PhysicalWalPolicy) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    let media = worth_store_testkit_store::physical_record_journeys::media(root);
    let durability = worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy(&media, wal);
    worth_store_testkit_store::physical_record_journeys::success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    )
}

pub(crate) fn open_existing(root: &Path, wal: PhysicalWalPolicy) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    let media = worth_store_testkit_store::physical_record_journeys::media(root);
    let durability = worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy(&media, wal);
    worth_store_testkit_store::physical_record_journeys::success(media.open_record_store(worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)))
}

pub(crate) fn checkpoint(serving: &ServingPhysicalRuntime) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([41; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(30_000).expect("deadline is positive"),
        ),
    );
    let handle = match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint admission did not produce a handle"),
    };
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_) => {}
        _ => panic!("checkpoint after WAL rotation must complete"),
    }
}

pub(crate) fn intent_is_behind_the_active_tail(root: &Path) -> bool {
    let names = wal_names(root);
    let Some(newest) = names.last() else {
        return false;
    };
    names.len() > 1 && !file_contains_retirement(newest)
}

pub(crate) fn retirement_wal_files(root: &Path) -> Vec<String> {
    wal_names(root)
        .into_iter()
        .filter(|name| file_contains_retirement(name))
        .collect()
}

pub(crate) fn wal_names(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    collect_files(&root.join("families").join("wal"), &mut names);
    // Segment ordinals are decimal, so shorter names sort first.
    names.sort_by(|left, right| left.len().cmp(&right.len()).then_with(|| left.cmp(right)));
    names
}

pub(crate) fn collect_files(path: &Path, names: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, names);
        } else {
            names.push(path.to_string_lossy().into_owned());
        }
    }
}

pub(crate) fn file_contains_retirement(path: &str) -> bool {
    !file_retirement_payloads(std::path::Path::new(path)).is_empty()
}