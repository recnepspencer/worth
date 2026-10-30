pub(crate) use worth_store::physical_runtime::*;
pub(crate) use {
    std::collections::BTreeMap, std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path,
    std::path::PathBuf,
};
pub(crate) use {
    worth_store_physical_format::PhysicalArtifactReadTarget,
    worth_store_physical_format::PhysicalGeneration,
    worth_store_physical_format::PhysicalGenerationAuthority,
    worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalSegmentId,
    worth_store_physical_format::RecordArtifactFile,
};
pub(crate) use {
    worth_store_physical_integrity::PhysicalArtifactScope,
    worth_store_physical_integrity::PhysicalByteRange,
};
pub(crate) use {
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

pub(crate) const RESIDENT_BYTES: u64 = 64 * 1024;
pub(crate) const PAGE_BYTES: u32 = 16 * 1024;
pub(crate) const RECORD_BYTES: usize = 7_000;
pub(crate) const RECORDS_PER_BATCH: usize = 40;
pub(crate) const BATCHES: usize = 8;

pub(crate) struct PressureWorld {
    pub(crate) serving: ServingPhysicalRuntime,
    pub(crate) targets: Vec<PhysicalIntegrityScrubTarget>,
    pub(crate) records: Vec<PhysicalRecordId>,
}

impl PressureWorld {
    pub(crate) fn new(root: &Path) -> Self {
        let (format, placement, access) = configuration();
        let policy = residency(format);
        let producer = success(initialize_record_store!(media(root), |durability| {
            PhysicalRecordInitialization::new(format, placement, access, durability)
                .with_residency_policy(policy)
        }));
        let mut records = Vec::new();
        for batch in 0..BATCHES {
            let payload = vec![batch as u8; RECORD_BYTES];
            let publication = worth_store_testkit_store::durable_publication::root::publish_single(
                &producer,
                placement,
                worth_store_testkit_store::durable_publication::material::certification_material(
                    "integrity-scrub-resident-pressure",
                    batch as u64 + 1,
                ),
                RecordAppendBatch::try_from_iter(
                    (0..RECORDS_PER_BATCH).map(|_| payload.as_slice()),
                )
                .unwrap(),
            );
            for index in 0..RECORDS_PER_BATCH {
                records.push(publication.settled_members()[0].record_id(index).unwrap());
            }
        }
        assert!(!producer.close().residency().requires_inspection());
        // The clean published manifest supplies coordinates, not expected integrity
        // verdicts. The public scrub must acquire and inspect every requested page.
        let published = worth_store_offline_verifier::walk_current_durable_record_manifest(
            root,
            format.declaration(),
        )
        .unwrap();
        assert!(published.payload_bytes() >= 32 * RESIDENT_BYTES);
        let serving = success(open_record_store!(media(root), |durability| {
            PhysicalRecordOpen::new(format, access, durability).with_residency_policy(policy)
        }));
        let targets = published
            .segment_pages()
            .iter()
            .map(|page| {
                let cell = worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
                    .page_cell(
                        worth_store_physical_format::PhysicalSegmentId::from_raw(page.segment()).unwrap(),
                        worth_store_physical_format::PhysicalPageId::from_raw(page.page()).unwrap(),
                    )
                    .with_page_generation(
                        worth_store_physical_format::PhysicalGeneration::from_raw(page.page_generation()).unwrap(),
                    );
                PhysicalIntegrityScrubTarget::new(
                    worth_store_physical_format::PhysicalArtifactReadTarget::Record(worth_store_physical_format::RecordArtifactFile::Segment {
                        segment: page.segment(),
                        generation: page.data_generation(),
                    }),
                    worth_store_physical_integrity::PhysicalArtifactScope::inline_page(
                        serving.store_identity(),
                        format.declaration(),
                        cell,
                        worth_store_physical_integrity::PhysicalByteRange::new(
                            u64::from(page.frame_index()) * u64::from(PAGE_BYTES),
                            u64::from(PAGE_BYTES),
                        )
                        .unwrap(),
                    ),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        assert!(targets.len() > (RESIDENT_BYTES / u64::from(PAGE_BYTES)) as usize);
        Self {
            serving,
            targets,
            records,
        }
    }
}

pub(crate) fn read_record(
    serving: &ServingPhysicalRuntime,
    records: &[PhysicalRecordId],
    index: usize,
) {
    let limits = RecordReadLimits::new(RecordByteLimit::new(RECORD_BYTES as u32).unwrap());
    let mut record = serving
        .records()
        .expect("read protection admission")
        .open(records[index], limits)
        .unwrap();
    let mut bytes = [0; RECORD_BYTES];
    assert_eq!(record.read_next(&mut bytes).unwrap(), RECORD_BYTES);
    assert!(bytes
        .iter()
        .all(|byte| *byte == (index / RECORDS_PER_BATCH) as u8));
    assert_eq!(record.read_next(&mut [0; 1]).unwrap(), 0);
}

pub(crate) fn snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn walk(root: &Path, directory: &Path, result: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let kind = entry.file_type().unwrap();
            assert!(!kind.is_symlink());
            let path = entry.path();
            if kind.is_dir() {
                result.insert(path.strip_prefix(root).unwrap().to_owned(), None);
                walk(root, &path, result);
            } else {
                assert!(kind.is_file());
                let relative = path.strip_prefix(root).unwrap();
                // The OS locks the live mutation lease. Its payload is explicitly
                // non-authoritative in C9; preserve its path/type, not its contents.
                let contents = if relative == std::path::Path::new("namespace/mutation.lock") {
                    Vec::new()
                } else {
                    std::fs::read(&path).unwrap()
                };
                result.insert(relative.to_owned(), Some(contents));
            }
        }
    }
    let mut result = std::collections::BTreeMap::new();
    walk(root, root, &mut result);
    result
}

pub(crate) fn residency(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    let bytes = |value| std::num::NonZeroU64::new(value).unwrap();
    let count = |value| std::num::NonZeroU32::new(value).unwrap();
    let operations = 16 * 1024 * 1024;
    let mut policy = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(operations + 2 * RESIDENT_BYTES + 16 * 1024))
        .resident_bytes(bytes(RESIDENT_BYTES))
        .metadata_bytes(bytes(16 * 1024))
        .frame_entries(count(8))
        .pinned_frames(count(8))
        .pin_leases(count(8))
        .dirty_frames(count(2))
        .dirty_replacement_bytes(bytes(RESIDENT_BYTES))
        .operation_bytes(bytes(operations));
    for scope in [
        PhysicalOperationAllocationScope::ForegroundRead,
        PhysicalOperationAllocationScope::ForegroundWrite,
        PhysicalOperationAllocationScope::Recovery,
        PhysicalOperationAllocationScope::Maintenance,
        PhysicalOperationAllocationScope::Verification,
        PhysicalOperationAllocationScope::Blob,
    ] {
        policy = policy.scope_bytes(scope, bytes(operations));
    }
    policy
        .scope_bytes(
            PhysicalOperationAllocationScope::Scrub,
            bytes(u64::from(PAGE_BYTES)),
        )
        .speculative_frames(PhysicalSpeculativeWorkKind::Prefetch, count(8))
        .speculative_frames(PhysicalSpeculativeWorkKind::ReadAhead, count(8))
        .speculative_frames(PhysicalSpeculativeWorkKind::WriteBehind, count(2))
        .admit(format)
        .into_result()
        .unwrap()
}
