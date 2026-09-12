use std::io::Write;
use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, PhysicalWorkEffectFate, PhysicalWritebackExecution,
    RecordAppendBatch, RecordByteLimit, RecordReadLimits,
};
use worth_store_physical_backend::{ArtifactRangeWriteDurabilityRequirement, MediaOperationRole};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

const REOPENED_PAYLOAD: &[u8] = b"writeback-survives-reopen";

#[test]
fn ordinary_record_and_exact_writeback_survive_fresh_store_admission() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let writer = super::run_child("physical_writeback_writer", &root, None);
    let evidence = writer
        .lines()
        .find_map(|line| line.strip_prefix("PHYSICAL_WRITEBACK "))
        .expect("writer must publish digest and record locator");
    let (digest, locator) = evidence
        .split_once(' ')
        .expect("writer evidence must separate digest from locator");
    let observer = super::run_child("physical_writeback_observer", &root, Some(digest));
    assert!(observer
        .lines()
        .any(|line| line == "PHYSICAL_WRITEBACK_OBSERVED"));
    let reopener = super::run_child("physical_writeback_reopener", &root, Some(locator));
    assert!(reopener
        .lines()
        .any(|line| line == "PHYSICAL_WRITEBACK_REOPENED"));
}

pub(super) fn writer(root: &Path) {
    let (profile, _, _) = super::physical_work::work_fixture();
    let serving =
        super::physical_work::serving_from_initialization_with_work_profile(root, profile);
    let (_, placement, _) = super::configuration();
    let published = super::durable_publication::publish_single(
        &serving,
        placement,
        super::durable_publication::certification_material("fresh-reopen-writeback", 1),
        RecordAppendBatch::try_from_iter([REOPENED_PAYLOAD]).unwrap(),
    );
    let locator = ExternalPhysicalRecordLocator::new(
        serving.store_identity(),
        published.settled_members()[0].record_id(0).unwrap(),
    );
    let coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 8, 8).unwrap();
    let expected =
        std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap()[8..16].to_vec();
    let residency = serving.certification_physical_residency();
    let writeback_before = serving.residency_observation().writebacks();
    let positioned_writes_before = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    let residency_before = residency.counters();
    let lease = residency.pin_exact(coordinate).unwrap();
    let dirty = residency
        .admit_dirty_frame(lease, |source, target| target.copy_from_slice(source))
        .unwrap();
    let prepared = residency
        .prepare_writeback(
            dirty,
            ArtifactRangeWriteDurabilityRequirement::FileDataSynchronization,
        )
        .unwrap();
    let ready = residency.request_writeback(prepared).unwrap();
    let admitted = residency.admit_writeback(ready).unwrap();
    let settlement = match residency.execute_writeback(admitted).unwrap() {
        PhysicalWritebackExecution::Clean(settlement) => settlement,
        PhysicalWritebackExecution::Retryable(_) => {
            panic!("unfaulted physical writeback unexpectedly required retry")
        }
        PhysicalWritebackExecution::InspectionRequired(inspection) => {
            panic!(
                "unfaulted physical writeback unexpectedly required inspection: {:?}",
                inspection.settlement()
            )
        }
    };
    assert_eq!(
        settlement.effect_fate(),
        PhysicalWorkEffectFate::WriteCompleted
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
            .attempts_for(MediaOperationRole::PositionedWrite)
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
        hex(&Sha256::digest(expected)),
        hex(&locator.encode())
    );
    std::io::stdout().flush().unwrap();
    std::process::exit(0);
}

pub(super) fn observer(root: &Path, expected_digest: &str) {
    let bytes = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    assert_eq!(hex(&Sha256::digest(&bytes[8..16])), expected_digest);
    println!("PHYSICAL_WRITEBACK_OBSERVED");
    std::io::stdout().flush().unwrap();
}

pub(super) fn reopener(root: &Path, encoded_locator: &str) {
    let serving = super::serving_from_open(root);
    assert!(!serving.observed_non_authoritative_residue());
    let locator = ExternalPhysicalRecordLocator::decode(unhex(encoded_locator)).unwrap();
    let record = serving
        .records()
        .expect("read protection admission")
        .open_external(
            locator,
            RecordReadLimits::new(RecordByteLimit::new(1024).unwrap()),
        )
        .unwrap();
    let (bytes, _) = super::read_record(record, REOPENED_PAYLOAD.len());
    assert_eq!(bytes, REOPENED_PAYLOAD);
    println!("PHYSICAL_WRITEBACK_REOPENED");
    std::io::stdout().flush().unwrap();
    serving.close();
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(value: &str) -> [u8; 40] {
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
