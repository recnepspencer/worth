use std::io::Write;
use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, PhysicalRecordInitialization, PhysicalRecordOpen,
    RecordAppendBatch, RecordByteLimit, RecordReadLimits, RecordScanOutcome, RecordScanRequest,
};
use worth_store_physical_backend::MediaOperationRole;

pub(super) fn writer(root: &Path, locators: std::path::PathBuf, oracle: std::path::PathBuf) {
    let records = super::courtroom_oracle::read(&oracle);
    assert_eq!(records.len(), 1_402);
    let (format, placement, access) = super::scenario_configuration::courtroom_configuration();
    let serving = super::success(initialize_record_store!(super::media(root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let counters_before_workload = serving.media_counters();
    let first = super::durable_publication::publish_single(
        &serving,
        placement,
        super::durable_publication::certification_material("c5-mixed-record-courtroom", 1),
        RecordAppendBatch::try_from_iter([payload(records[0])]).unwrap(),
    );
    let first_id = first.settled_members()[0].record_id(0).unwrap();
    let inline = records[1..1_400].iter().copied().map(payload);
    let inline = super::durable_publication::publish_single(
        &serving,
        placement,
        super::durable_publication::certification_material("c5-mixed-record-courtroom", 2),
        RecordAppendBatch::try_from_iter(inline).unwrap(),
    );
    let boundary = super::durable_publication::publish_single(
        &serving,
        placement,
        super::durable_publication::certification_material("c5-mixed-record-courtroom", 3),
        RecordAppendBatch::try_from_iter([payload(records[1_400])]).unwrap(),
    );
    let large = records[1_401];
    let large = super::durable_publication::publish_single(
        &serving,
        placement,
        super::durable_publication::certification_material("c5-mixed-record-courtroom", 4),
        RecordAppendBatch::builder()
            .push_source(super::stream_fixture::RepeatedByteSource::new(
                large.payload_bytes as u64,
                large.byte,
            ))
            .build()
            .unwrap(),
    );
    let inline_member = &inline.settled_members()[0];
    let boundary_member = &boundary.settled_members()[0];
    let large_member = &large.settled_members()[0];

    let mut writer = std::io::BufWriter::new(std::fs::File::create(locators).unwrap());
    write_locator(&mut writer, serving.store_identity(), first_id);
    for index in 0..inline_member.persisted_records().len() {
        let record = inline_member
            .record_id(index)
            .expect("every persisted courtroom record has one serving identity");
        write_locator(&mut writer, serving.store_identity(), record);
    }
    write_locator(
        &mut writer,
        serving.store_identity(),
        boundary_member.record_id(0).unwrap(),
    );
    write_locator(
        &mut writer,
        serving.store_identity(),
        large_member.record_id(0).unwrap(),
    );
    writer.flush().unwrap();
    super::scenario_evidence::emit_process("writer", &serving);
    let counters = serving.media_counters();
    println!(
        "C5_COURTROOM_WRITER {} {} {} {} {} {}",
        large.current_root().generation(),
        large_member.mutation_identity().operation_identity().get(),
        counters
            .attempts_for(MediaOperationRole::PositionedWrite)
            .saturating_sub(
                counters_before_workload.attempts_for(MediaOperationRole::PositionedWrite),
            ),
        counters
            .attempts_for(MediaOperationRole::SynchronizeFileState)
            .saturating_sub(
                counters_before_workload.attempts_for(MediaOperationRole::SynchronizeFileState),
            ),
        counters
            .attempts_for(MediaOperationRole::AtomicReplace)
            .saturating_sub(
                counters_before_workload.attempts_for(MediaOperationRole::AtomicReplace),
            ),
        counters
            .attempts_for(MediaOperationRole::SynchronizeDirectoryPublication)
            .saturating_sub(
                counters_before_workload
                    .attempts_for(MediaOperationRole::SynchronizeDirectoryPublication,)
            ),
    );
    std::io::stdout().flush().unwrap();
    std::process::exit(0);
}

pub(super) fn reopener(root: &Path, evidence: std::path::PathBuf) {
    let (format, _, access) = super::scenario_configuration::courtroom_configuration();
    let serving = super::success(open_record_store!(super::media(root), |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    let mut rows = std::fs::read_to_string(evidence)
        .unwrap()
        .lines()
        .map(parse_locator)
        .enumerate()
        .collect::<Vec<_>>();
    rows.reverse();
    let mut point_rows = Vec::with_capacity(rows.len());
    for (index, locator) in &rows {
        let mut read = serving
            .records()
            .expect("read protection admission")
            .open_external(
                *locator,
                RecordReadLimits::new(RecordByteLimit::new(u32::MAX).unwrap()),
            )
            .unwrap();
        let mut buffer = vec![0_u8; 65_536];
        let mut payload = Sha256::new();
        let mut length = 0_u64;
        loop {
            let count = read.read_next(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            payload.update(&buffer[..count]);
            length += count as u64;
        }
        point_rows.push((*index, locator.encode(), length, payload.finalize()));
    }
    point_rows.sort_by_key(|row| row.0);
    let mut point_digest = Sha256::new();
    for (_, locator, length, payload) in point_rows {
        point_digest.update(locator);
        point_digest.update(length.to_le_bytes());
        point_digest.update(payload);
    }

    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(RecordScanRequest::from_start())
        .unwrap();
    let mut scratch = vec![0_u8; 131_072];
    let mut records = 0_usize;
    let mut deferred = 0_usize;
    let mut scan_digest = Sha256::new();
    let final_scan = loop {
        match scan.read_next_into(&mut scratch).unwrap() {
            RecordScanOutcome::Batch(batch) => {
                records += batch.records().len();
                for record in batch.records() {
                    scan_digest.update(record.record_id().allocation_epoch());
                    scan_digest.update(record.record_id().ordinal().to_le_bytes());
                    scan_digest.update(record.declared_payload_bytes().to_le_bytes());
                    deferred += usize::from(record.payload_is_deferred());
                }
            }
            RecordScanOutcome::Completed(completed) => {
                break completed.observation();
            }
        }
    };
    super::scenario_evidence::emit_process("reopener", &serving);
    println!(
        "C5_COURTROOM_REOPEN {records} {deferred} {} {} {} {} {} {}",
        serving
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .root_generation(),
        super::courtroom_oracle::hex(&point_digest.finalize()),
        super::courtroom_oracle::hex(&scan_digest.finalize()),
        final_scan.manifest_blocks(),
        final_scan.manifest_comparisons(),
        final_scan.payload_bytes(),
    );
    std::io::stdout().flush().unwrap();
    serving.close();
}

fn write_locator(
    writer: &mut impl Write,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    record: worth_store::physical_runtime::PhysicalRecordId,
) {
    let locator = ExternalPhysicalRecordLocator::new(store, record);
    writeln!(writer, "{}", super::child_process::hex(&locator.encode())).unwrap();
}

fn parse_locator(line: &str) -> ExternalPhysicalRecordLocator {
    super::child_process::decode_locator(line)
}

fn payload(record: super::courtroom_oracle::OracleRecord) -> Vec<u8> {
    vec![record.byte; record.payload_bytes]
}
