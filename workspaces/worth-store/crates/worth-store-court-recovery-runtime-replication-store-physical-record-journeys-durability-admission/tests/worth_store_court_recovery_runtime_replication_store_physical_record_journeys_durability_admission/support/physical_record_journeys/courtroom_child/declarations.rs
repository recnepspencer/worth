pub(crate) use std::io::Write;
pub(crate) use std::path::Path;

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecoveryJournalCounters,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_backend::MediaCounterSnapshot,
    worth_store_physical_backend::MediaOperationRole,
};

pub(crate) fn writer(root: &Path, locators: std::path::PathBuf, oracle: std::path::PathBuf) {
    let records = crate::courtroom_oracle::read(&oracle);
    assert_eq!(records.len(), 1_402);
    let (format, placement, access) = worth_store_testkit_store::physical_record_journeys::scenario_configuration::courtroom_configuration();
    let serving =
        worth_store_testkit_store::physical_record_journeys::success(initialize_record_store!(
            worth_store_testkit_store::physical_record_journeys::media(root),
            |durability| {
                worth_store::physical_runtime::PhysicalRecordInitialization::new(
                    format, placement, access, durability,
                )
            }
        ));
    let counters_before_workload = serving.media_counters();
    let mut publications =
        crate::physical_record_journeys::courtroom_child::PublicationBarriers::start(&serving);
    let first = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "c5-mixed-record-courtroom",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload(records[0])])
            .unwrap(),
    );
    publications.mark(&serving);
    let first_id = first.settled_members()[0].record_id(0).unwrap();
    let inline = records[1..1_400].iter().copied().map(payload);
    let inline = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "c5-mixed-record-courtroom",
            2,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter(inline).unwrap(),
    );
    publications.mark(&serving);
    let boundary = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "c5-mixed-record-courtroom",
            3,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload(records[1_400])])
            .unwrap(),
    );
    publications.mark(&serving);
    let large = records[1_401];
    let large = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "c5-mixed-record-courtroom",
            4,
        ),
        worth_store::physical_runtime::RecordAppendBatch::builder()
            .push_source(
                worth_store_testkit_store::stream_fixture::RepeatedByteSource::new(
                    large.payload_bytes as u64,
                    large.byte,
                ),
            )
            .build()
            .unwrap(),
    );
    publications.mark(&serving);
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
    worth_store_testkit_store::scenario_process_evidence::emit_process("writer", &serving);
    publications.emit();
    let counters = serving.media_counters();
    println!(
        "C5_COURTROOM_WRITER {} {} {} {} {} {}",
        large.current_root().generation(),
        large_member.mutation_identity().operation_identity().get(),
        counters
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            .saturating_sub(
                counters_before_workload.attempts_for(
                    worth_store_physical_backend::MediaOperationRole::PositionedWrite
                ),
            ),
        counters
            .attempts_for(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState)
            .saturating_sub(counters_before_workload.attempts_for(
                worth_store_physical_backend::MediaOperationRole::SynchronizeFileState
            ),),
        counters
            .attempts_for(worth_store_physical_backend::MediaOperationRole::AtomicReplace)
            .saturating_sub(
                counters_before_workload
                    .attempts_for(worth_store_physical_backend::MediaOperationRole::AtomicReplace),
            ),
        counters
            .attempts_for(
                worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication
            )
            .saturating_sub(counters_before_workload.attempts_for(
                worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication,
            )),
    );
    std::io::stdout().flush().unwrap();
    std::process::exit(0);
}

/// Per-publication catalog replacements and the directory barriers issued
/// outside the recovery journal, so a parent-directory barrier can be checked
/// against each replacement instead of against the whole workload.
pub(crate) struct PublicationBarriers {
    pub(crate) last: (MediaCounterSnapshot, PhysicalRecoveryJournalCounters),
    pub(crate) rows: Vec<(u64, u64, u64)>,
}

impl PublicationBarriers {
    pub(crate) fn start(serving: &ServingPhysicalRuntime) -> Self {
        Self {
            last: Self::observe(serving),
            rows: Vec::new(),
        }
    }

    pub(crate) fn observe(
        serving: &ServingPhysicalRuntime,
    ) -> (MediaCounterSnapshot, PhysicalRecoveryJournalCounters) {
        (
            serving.media_counters(),
            serving.physical_recovery_journal_counters(),
        )
    }

    pub(crate) fn mark(&mut self, serving: &ServingPhysicalRuntime) {
        let (media, journal) = Self::observe(serving);
        let (last_media, last_journal) = self.last;
        let delta = |role| media.attempts_for(role) - last_media.attempts_for(role);
        let replacements = delta(worth_store_physical_backend::MediaOperationRole::AtomicReplace);
        let journal_barriers = journal.directory_barriers() - last_journal.directory_barriers();
        let namespace_barriers = delta(
            worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication,
        ) - journal_barriers;
        self.rows
            .push((replacements, namespace_barriers, journal_barriers));
        self.last = (media, journal);
    }

    pub(crate) fn emit(&self) {
        for (replacements, namespace, journal) in &self.rows {
            println!("C5_COURTROOM_PUBLICATION {replacements} {namespace} {journal}");
        }
    }
}

pub(crate) fn reopener(root: &Path, evidence: std::path::PathBuf) {
    let (format, _, access) = worth_store_testkit_store::physical_record_journeys::scenario_configuration::courtroom_configuration();
    let serving = worth_store_testkit_store::physical_record_journeys::success(open_record_store!(
        worth_store_testkit_store::physical_record_journeys::media(root),
        |durability| {
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
        }
    ));
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
                worth_store::physical_runtime::RecordReadLimits::new(
                    worth_store::physical_runtime::RecordByteLimit::new(u32::MAX).unwrap(),
                ),
            )
            .unwrap();
        let mut buffer = vec![0_u8; 65_536];
        let mut payload = sha2::Sha256::new();
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
    let mut point_digest = sha2::Sha256::new();
    for (_, locator, length, payload) in point_rows {
        point_digest.update(locator);
        point_digest.update(length.to_le_bytes());
        point_digest.update(payload);
    }

    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start())
        .unwrap();
    let mut scratch = vec![0_u8; 131_072];
    let mut records = 0_usize;
    let mut deferred = 0_usize;
    let mut scan_digest = sha2::Sha256::new();
    let final_scan = loop {
        match scan.read_next_into(&mut scratch).unwrap() {
            worth_store::physical_runtime::RecordScanOutcome::Batch(batch) => {
                records += batch.records().len();
                for record in batch.records() {
                    scan_digest.update(record.record_id().allocation_epoch());
                    scan_digest.update(record.record_id().ordinal().to_le_bytes());
                    scan_digest.update(record.declared_payload_bytes().to_le_bytes());
                    deferred += usize::from(record.payload_is_deferred());
                }
            }
            worth_store::physical_runtime::RecordScanOutcome::Completed(completed) => {
                break completed.observation();
            }
        }
    };
    worth_store_testkit_store::scenario_process_evidence::emit_process("reopener", &serving);
    println!(
        "C5_COURTROOM_REOPEN {records} {deferred} {} {} {} {} {} {}",
        serving
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .root_generation(),
        crate::courtroom_oracle::hex(&point_digest.finalize()),
        crate::courtroom_oracle::hex(&scan_digest.finalize()),
        final_scan.manifest_blocks(),
        final_scan.manifest_comparisons(),
        final_scan.payload_bytes(),
    );
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub(crate) fn write_locator(
    writer: &mut impl Write,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    record: worth_store::physical_runtime::PhysicalRecordId,
) {
    let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(store, record);
    writeln!(
        writer,
        "{}",
        worth_store_testkit_store::child_process::locator_codec::hex(&locator.encode())
    )
    .unwrap();
}

pub(crate) fn parse_locator(line: &str) -> ExternalPhysicalRecordLocator {
    worth_store_testkit_store::child_process::locator_codec::decode_locator(line)
}

pub(crate) fn payload(record: crate::courtroom_oracle::OracleRecord) -> Vec<u8> {
    vec![record.byte; record.payload_bytes]
}
