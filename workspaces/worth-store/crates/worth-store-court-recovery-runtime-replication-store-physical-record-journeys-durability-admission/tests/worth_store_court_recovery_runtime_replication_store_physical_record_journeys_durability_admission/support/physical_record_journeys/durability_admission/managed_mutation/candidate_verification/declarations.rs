pub(crate) use {std::collections::BTreeSet};
pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalMutationIndeterminateStage, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalRecordId, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadDenial, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_backend::FilesystemAccessPosture, worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole, worth_store_physical_backend::MediaPauseGate};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) const PAYLOAD: &[u8] = b"candidate-verified-before-publication";
/// The candidate byte the interposer flips: the middle of its only page, so the
/// damage lands inside the page body rather than on a file boundary.
pub(crate) fn candidate_damage_offset() -> usize {
    configuration().0.declaration().page_size().bytes() as usize / 2
}

pub(crate) fn published_source(root: &Path) -> PhysicalRecordId {
    let serving = serving_from_initialization(root);
    let (_, placement, _) = configuration();
    let record = completed(prepare(&serving, placement, [63; 32], PAYLOAD).execute())
        .into_acknowledgment()
        .record_ids()
        .next()
        .unwrap();
    serving.close();
    record
}

pub(crate) fn serving_paused_after_write(
    root: &Path,
    ordinal: u64,
) -> (ServingPhysicalRuntime, MediaPauseGate) {
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            worth_store_physical_backend::MediaOperationRole::PositionedWrite,
            ordinal,
            worth_store_physical_backend::MediaFaultDirective::PauseAfter(gate.clone()),
        )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("paused media should admit"),
    };
    let (format, _, access) = configuration();
    let serving = worth_store_testkit_store::physical_record_journeys::success(open_record_store!(media, |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    (serving, gate)
}

pub(crate) fn segment_files(root: &Path) -> BTreeSet<PathBuf> {
    std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect()
}

pub(crate) fn root_generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation()
}

pub(crate) fn read_record(serving: &ServingPhysicalRuntime, record: PhysicalRecordId) -> Vec<u8> {
    let reader = serving.records().unwrap();
    let mut session = reader
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(PAYLOAD.len() as u32).unwrap()),
        )
        .unwrap();
    let mut bytes = vec![0_u8; PAYLOAD.len()];
    let mut filled = 0;
    while filled < bytes.len() {
        let count = session.read_next(&mut bytes[filled..]).unwrap();
        assert!(count > 0);
        filled += count;
    }
    bytes
}