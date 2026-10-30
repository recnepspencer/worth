pub use std::io::Write;
pub use std::path::Path;

pub use {sha2::Digest, sha2::Sha256};
pub use {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::PhysicalWorkEffectFate,
    worth_store::physical_runtime::PhysicalWritebackExecution,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};
pub use {
    worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement,
    worth_store_physical_backend::MediaOperationRole,
};
pub use {
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

pub const REOPENED_PAYLOAD: &[u8] = b"writeback-survives-reopen";

pub fn writer(root: &Path) {
    let (profile, _, _) =
        crate::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture();
    let serving =
        crate::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile(root, profile);
    let (_, placement, _) = crate::physical_record_journeys::configuration();
    let published = crate::durable_publication::root::publish_single(
        &serving,
        placement,
        crate::durable_publication::material::certification_material("fresh-reopen-writeback", 1),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([REOPENED_PAYLOAD])
            .unwrap(),
    );
    let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(
        serving.store_identity(),
        published.settled_members()[0].record_id(0).unwrap(),
    );
    let coordinate = worth_store_physical_format::RecordFrameCoordinate::new(
        worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
        8,
        8,
    )
    .unwrap();
    let expected =
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap()[8..16].to_vec();
    let residency = serving.certification_physical_residency();
    let writeback_before = serving.residency_observation().writebacks();
    let positioned_writes_before = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    let residency_before = residency.counters();
    let lease = residency.pin_exact(coordinate).unwrap();
    let dirty = residency
        .admit_dirty_frame(lease, |source, target| target.copy_from_slice(source))
        .unwrap();
    let prepared = residency
        .prepare_writeback(
            dirty,
            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::FileDataSynchronization,
        )
        .unwrap();
    let ready = residency.request_writeback(prepared).unwrap();
    let admitted = residency.admit_writeback(ready).unwrap();
    let settlement = match residency.execute_writeback(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWritebackExecution::Clean(settlement) => settlement,
        worth_store::physical_runtime::PhysicalWritebackExecution::Retryable(_) => {
            panic!("unfaulted physical writeback unexpectedly required retry")
        }
        worth_store::physical_runtime::PhysicalWritebackExecution::InspectionRequired(
            inspection,
        ) => {
            panic!(
                "unfaulted physical writeback unexpectedly required inspection: {:?}",
                inspection.settlement()
            )
        }
    };
    assert_eq!(
        settlement.effect_fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    assert_eq!(residency.counters().dirty_frames(), 0);
    let writeback_after = serving.residency_observation().writebacks();
    assert_eq!(writeback_after.attempts(), writeback_before.attempts() + 1);
    assert_eq!(
        writeback_after.exact_receipts(),
        writeback_before.exact_receipts() + 1
    );
    assert_eq!(
        writeback_after.inspection_required(),
        writeback_before.inspection_required()
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            .saturating_sub(positioned_writes_before),
        1
    );
    assert_eq!(
        residency
            .counters()
            .writebacks()
            .saturating_sub(residency_before.writebacks()),
        1
    );
    println!(
        "PHYSICAL_WRITEBACK {} {}",
        hex(&sha2::Sha256::digest(expected)),
        hex(&locator.encode())
    );
    std::io::stdout().flush().unwrap();
    std::process::exit(0);
}

pub fn observer(root: &Path, expected_digest: &str) {
    let bytes = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    assert_eq!(hex(&sha2::Sha256::digest(&bytes[8..16])), expected_digest);
    println!("PHYSICAL_WRITEBACK_OBSERVED");
    std::io::stdout().flush().unwrap();
}

pub fn reopener(root: &Path, encoded_locator: &str) {
    let serving = crate::physical_record_journeys::serving_from_open(root);
    assert!(!serving.observed_non_authoritative_residue());
    let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::decode(unhex(
        encoded_locator,
    ))
    .unwrap();
    let record = serving
        .records()
        .expect("read protection admission")
        .open_external(
            locator,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(1024).unwrap(),
            ),
        )
        .unwrap();
    let (bytes, _) = crate::physical_record_journeys::read_record(record, REOPENED_PAYLOAD.len());
    assert_eq!(bytes, REOPENED_PAYLOAD);
    println!("PHYSICAL_WRITEBACK_REOPENED");
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn unhex(value: &str) -> [u8; 40] {
    let bytes = value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digits = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(digits, 16).unwrap()
        })
        .collect::<Vec<_>>();
    bytes
        .try_into()
        .expect("external physical record locator evidence is exactly 40 bytes")
}
