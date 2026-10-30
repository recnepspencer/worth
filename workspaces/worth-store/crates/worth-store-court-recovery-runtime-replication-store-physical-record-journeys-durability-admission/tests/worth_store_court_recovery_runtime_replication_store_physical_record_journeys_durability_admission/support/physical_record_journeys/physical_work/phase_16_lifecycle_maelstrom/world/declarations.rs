pub(crate) use std::path::PathBuf;

pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::PhysicalWorkObservation,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {
    crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::fixture::MaelstromFixture,
    crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::fixture::MaelstromPauseGates,
};

pub(crate) struct MaelstromWorld {
    pub(crate) _parent: tempfile::TempDir,
    pub(crate) root: PathBuf,
    pub(crate) fixture: MaelstromFixture,
    pub(crate) serving: ServingPhysicalRuntime,
    pub(crate) gates: MaelstromPauseGates,
    pub(crate) observer: PhysicalWorkObservation,
    pub(crate) catalog_path: PathBuf,
    pub(crate) original_writeback: Vec<u8>,
}

pub(crate) fn open() -> MaelstromWorld {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let fixture = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::fixture::maelstrom_fixture();
    assert!(fixture.read_delta.is_partitioned());
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile(
        &root,
        fixture.profile.clone(),
    )
    .close();
    let calibration =
        worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile(&root, fixture.profile.clone());
    let calibration_counters = calibration.media_counters();
    let identified_reads_after_open = calibration_counters.identified_operation_attempts_for(
        worth_store_physical_backend::MediaOperationRole::PositionedRead,
    );
    let identified_writes_after_open = calibration_counters.identified_operation_attempts_for(
        worth_store_physical_backend::MediaOperationRole::PositionedWrite,
    );
    calibration.close();
    let catalog_path = root.join("families/records/bootstrap.catalog");
    let original_writeback = std::fs::read(&catalog_path).unwrap()[8..16].to_vec();
    let (serving, gates) = crate::physical_record_journeys::physical_work::phase_16_lifecycle_maelstrom::fixture::open_with_maelstrom_faults(
        &root,
        fixture.profile.clone(),
        identified_reads_after_open,
        identified_writes_after_open,
    );
    assert_open_counters(
        &serving,
        identified_reads_after_open,
        identified_writes_after_open,
    );
    let observer = serving.physical_work_observer();
    MaelstromWorld {
        _parent: parent,
        root,
        fixture,
        serving,
        gates,
        observer,
        catalog_path,
        original_writeback,
    }
}

pub(crate) fn assert_open_counters(
    serving: &ServingPhysicalRuntime,
    identified_reads: u64,
    identified_writes: u64,
) {
    assert_eq!(
        serving.media_counters().identified_operation_attempts_for(
            worth_store_physical_backend::MediaOperationRole::PositionedRead
        ),
        identified_reads
    );
    assert_eq!(
        serving.media_counters().identified_operation_attempts_for(
            worth_store_physical_backend::MediaOperationRole::PositionedWrite
        ),
        identified_writes
    );
}
