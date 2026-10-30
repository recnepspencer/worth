pub(crate) use {std::num::NonZeroU16, std::num::NonZeroU32, std::num::NonZeroU64};

pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_blob_chunks::BlobChunkSize;
pub(crate) use {
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::BlobReclaimLimitDenial,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimReceipt,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::BlobReclaimRetirement,
    worth_store::physical_runtime::BlobReclaimRetirementBudget,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::BlobTerminalLimits,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

pub(crate) use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::configuration,
    worth_store_testkit_store::fixture::serving_from_initialization_with_placement_and_tail,
    worth_store_testkit_store::fixture::serving_from_open_with_retained_wal_tail,
};

pub(crate) const CHUNK_BYTES: usize = 64 * 1024;
pub(crate) const CHUNKS: usize = 1025;
pub(crate) const INSPECTION_BYTE_DENIAL: u64 = 8 * 1024 * 1024;
// Three selected-root passes × 1,024 × (64KiB + 140B frame) = 201,756,672B.
// 256MiB covers custody too; encoded inspection is distinct from <=8MiB RAM.
pub(crate) const FULL_INSPECTION_ALLOWANCE: u64 = 256 * 1024 * 1024;
// The second batch reserves all 1,024 chunk extents before its first WAL effect.
// The common 64MiB tail cannot cover those extents plus retained WAL/candidates;
// admit a finite 128MiB tail on every open without weakening capacity checks.
pub(crate) const BATCH_RETAINED_WAL_TAIL_BYTES: u64 = 128 * 1024 * 1024;
pub(crate) const SCOPE: &str = "c11.blob.reclaim.batch-boundary.scope";

pub(crate) fn batch_placement() -> AdmittedRecordPlacementPolicy {
    let (format, _, _) = configuration();
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        // 8MiB index admits 2,047 ranges, enough for 1,024 isolated releases
        // plus frontier residue without future allocation coalescence.
        .arena_index_bytes(
            worth_store::physical_runtime::RecordByteLimit::new(8 * 1024 * 1024).unwrap(),
        )
        .admit(format)
        .unwrap()
}

pub(crate) fn abandoned_1025_chunk_ingest(
    serving: &ServingPhysicalRuntime,
    scope: &worth_store::physical_runtime::AdmittedBlobScope,
    placement: AdmittedRecordPlacementPolicy,
) -> BlobResumeToken {
    let blobs = serving.blobs().unwrap();
    let limits = worth_store::physical_runtime::BlobReadLimits::new(
        std::num::NonZeroU64::new(4096).unwrap(),
    );
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        (CHUNKS * CHUNK_BYTES) as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(1024).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement, CHUNK_BYTES as u64, limits)
        .unwrap();
    let token = ingest.resume_token();
    let mut source = [0_u8; CHUNK_BYTES];
    for ordinal in 0..CHUNKS {
        source.fill((ordinal % 251) as u8);
        ingest.push(&source).unwrap();
        if (ordinal + 1) % 64 == 0 {
            physical_checkpoint(serving, ordinal + 1);
        }
    }
    assert_eq!(ingest.frontier().next_chunk_ordinal(), CHUNKS as u64);
    drop(ingest);
    blobs
        .abort_ingest(
            token,
            scope,
            placement,
            deadline(),
            worth_store::physical_runtime::BlobTerminalLimits::new(
                std::num::NonZeroU64::new(4096).unwrap(),
            ),
        )
        .unwrap();
    token
}

pub(crate) fn physical_checkpoint(serving: &ServingPhysicalRuntime, completed_chunks: usize) {
    let mut key = [0xb3; 32];
    key[..8].copy_from_slice(&(completed_chunks as u64).to_le_bytes());
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(120_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("checkpoint admission failed after {completed_chunks} chunks");
    };
    assert!(
        matches!(
            handle.wait(),
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "checkpoint did not complete after {completed_chunks} chunks"
    );
}

pub(crate) fn request<'a>(
    token: BlobResumeToken,
    scope: &'a worth_store::physical_runtime::AdmittedBlobScope,
    placement: AdmittedRecordPlacementPolicy,
    inspected_bytes: u64,
) -> BlobReclaimRequest<'a> {
    worth_store::physical_runtime::BlobReclaimRequest::abandoned(
        token,
        scope,
        placement,
        deadline(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(4096).unwrap(),
            std::num::NonZeroU64::new(inspected_bytes).unwrap(),
            std::num::NonZeroU16::new(1024).unwrap(),
        )
        .unwrap(),
    )
}

pub(crate) fn assert_durable_batch(stage: &str, receipt: &BlobReclaimReceipt) {
    assert_eq!(
        receipt.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped,
        "{stage}: {receipt:?}"
    );
    assert_eq!(
        receipt.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed,
        "{stage}: displaced={} released={} receipt={receipt:?}",
        receipt.displaced_extents().len(),
        receipt.bytes_released(),
    );
    assert!(receipt.bytes_released() > 0, "{stage}: {receipt:?}");
    assert_eq!(
        receipt.bytes_released(),
        receipt
            .displaced_extents()
            .iter()
            .map(|extent| extent.range().length())
            .sum::<u64>()
    );
    let work = receipt.observation();
    assert!(work.inspected_payload_bytes() <= FULL_INSPECTION_ALLOWANCE);
    assert!(work.admitted_memory_bytes() <= INSPECTION_BYTE_DENIAL);
}

/// Native FIFO retirement may need more than the initial `wait()` quantum.
/// Neither a pending result nor exhausted continuation work proves release.
pub(crate) fn finish_durable_batch(
    serving: &ServingPhysicalRuntime,
    stage: &str,
    receipt: &mut BlobReclaimReceipt,
) {
    let expected = receipt
        .displaced_extents()
        .iter()
        .map(|extent| extent.range().length())
        .sum::<u64>();
    let budget = worth_store::physical_runtime::BlobReclaimRetirementBudget::new(
        std::num::NonZeroU32::new(128).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
            .unwrap(),
    );
    let mut previous = receipt.bytes_released();
    assert!(previous <= expected, "{stage}: excessive native credit");
    for _ in 0..64 {
        if receipt.retirement() == worth_store::physical_runtime::BlobReclaimRetirement::Completed {
            assert_eq!(receipt.bytes_released(), expected, "{stage}: short credit");
            return;
        }
        serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(receipt, budget)
            .unwrap();
        let released = receipt.bytes_released();
        assert!(
            released >= previous && released <= expected,
            "{stage}: invalid native credit"
        );
        previous = released;
    }
    panic!("{stage}: retirement remained pending after bounded continuation: {receipt:?}");
}

/// Read one selected frame at a time; retain only the authenticated occurrence
/// identities needed to compare the two drop receipts with the source custody.
pub(crate) fn selected_chunk_identities(
    serving: &ServingPhysicalRuntime,
    token: BlobResumeToken,
) -> Vec<PersistedRecordIdentity> {
    let session: [u8; 16] = token.encode()[24..40].try_into().unwrap();
    let mut by_ordinal = vec![None; CHUNKS];
    let mut selected_chunks = 0;
    let mut declaration_seen = false;
    let mut abandonment_seen = false;
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap())
                .with_payload_limit(
                    worth_store::physical_runtime::RecordByteLimit::new(236).unwrap(),
                ),
        )
        .unwrap();
    let mut scratch = [0_u8; 4096];
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) =
        scan.read_next_into(&mut scratch).unwrap()
    {
        for row in batch.records() {
            let record = row.record_id();
            let bytes = read_selected(serving, record, row.declared_payload_bytes());
            if !bytes.starts_with(b"WRC11BLB") {
                continue;
            }
            match decode_blob_record(&bytes).unwrap() {
                worth_store_physical_format::BlobRecordV1::SessionDeclared(value)
                    if value.session() == session =>
                {
                    assert!(!declaration_seen, "duplicate selected declaration");
                    declaration_seen = true;
                }
                worth_store_physical_format::BlobRecordV1::SessionAbandoned(value)
                    if value.session() == session =>
                {
                    assert!(!abandonment_seen, "duplicate selected abandonment");
                    abandonment_seen = true;
                }
                worth_store_physical_format::BlobRecordV1::Chunk(value)
                    if value.occurrence().session() == session =>
                {
                    let ordinal = usize::try_from(value.occurrence().ordinal()).unwrap();
                    assert!(ordinal < CHUNKS);
                    assert_eq!(value.bytes().len(), CHUNK_BYTES);
                    assert!(value
                        .bytes()
                        .iter()
                        .all(|byte| *byte == (ordinal % 251) as u8));
                    let identity = persisted(record);
                    assert!(by_ordinal[ordinal].replace(identity).is_none());
                    selected_chunks += 1;
                }
                worth_store_physical_format::BlobRecordV1::GenerationPublished(value)
                    if value.session() == session =>
                {
                    panic!("abandoned ingest published generation")
                }
                _ => {}
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    assert!(
        declaration_seen && abandonment_seen,
        "selected terminal custody"
    );
    assert_eq!(selected_chunks, by_ordinal.iter().flatten().count());
    by_ordinal.into_iter().flatten().collect()
}

pub(crate) fn read_selected(
    serving: &ServingPhysicalRuntime,
    record: PhysicalRecordId,
    length: u64,
) -> Vec<u8> {
    let limit = worth_store::physical_runtime::RecordByteLimit::new(u32::try_from(length).unwrap())
        .unwrap();
    let mut stream = serving
        .records()
        .unwrap()
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(limit),
        )
        .unwrap();
    let mut bytes = vec![0; length as usize];
    let mut used = 0;
    while used < bytes.len() {
        let amount = stream.read_next(&mut bytes[used..]).unwrap();
        assert!(amount > 0, "selected frame truncated");
        used += amount;
    }
    bytes
}

pub(crate) fn persisted(record: PhysicalRecordId) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new(
        record.allocation_epoch(),
        record.ordinal(),
    )
    .unwrap()
}

pub(crate) fn root_generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}

pub(crate) fn deadline() -> PhysicalMutationDeadline {
    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(1_800_000).unwrap()
}
