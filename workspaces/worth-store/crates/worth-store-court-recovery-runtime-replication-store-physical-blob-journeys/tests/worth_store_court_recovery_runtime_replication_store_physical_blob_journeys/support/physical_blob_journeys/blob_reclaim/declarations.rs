pub(crate) use {std::num::NonZeroU16, std::num::NonZeroU64};

pub(crate) use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDeferral, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimFailure, worth_store::physical_runtime::BlobReclaimLimits, worth_store::physical_runtime::BlobReclaimReceipt, worth_store::physical_runtime::BlobReclaimRequest, worth_store::physical_runtime::BlobReclaimRetirement, worth_store::physical_runtime::BlobResumeToken, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalWorkCounterStage, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkPressureClass, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

pub(crate) use {crate::physical_blob_journeys::blob_abort::deadline, crate::physical_blob_journeys::blob_abort::limits as terminal_limits, crate::physical_blob_journeys::blob_crash::establish_recovery_frontier, crate::blob_crash::recovery::recover_closed_store, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};

pub(crate) const CHUNK: usize = 64 << 10;

pub(crate) fn assert_completed_retirement(receipt: &BlobReclaimReceipt) {
    assert_eq!(
        receipt.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed,
        "{receipt:?}"
    );
    assert!(receipt.bytes_released() > 0);
    assert_eq!(
        receipt.bytes_released(),
        receipt
            .displaced_extents()
            .iter()
            .map(|extent| extent.range().length())
            .sum::<u64>()
    );
}

pub(crate) fn abandoned_prefix(
    serving: &ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
) -> BlobResumeToken {
    let blobs = serving.blobs().unwrap();
    let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (CHUNK * 3) as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(32).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, limits)
        .unwrap();
    ingest.push(&[0x31; CHUNK]).unwrap();
    ingest.push(&[0x52; CHUNK]).unwrap();
    ingest.checkpoint().unwrap();
    let token = ingest.resume_token();
    drop(ingest);
    blobs
        .abort_ingest(token, scope, placement(), deadline(), terminal_limits())
        .unwrap();
    token
}

pub(crate) fn request(token: BlobResumeToken, scope: &AdmittedBlobScope) -> BlobReclaimRequest<'_> {
    worth_store::physical_runtime::BlobReclaimRequest::abandoned(
        token,
        scope,
        placement(),
        deadline(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(128).unwrap(),
            std::num::NonZeroU64::new(8 << 20).unwrap(),
            std::num::NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    )
}

pub(crate) fn count_kind(
    records: &[(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)],
    kind: u8,
) -> usize {
    records
        .iter()
        .filter(|(_, bytes)| bytes.starts_with(b"WRC11BLB"))
        .filter(|(_, bytes)| match decode_blob_record(bytes).unwrap() {
            worth_store_physical_format::BlobRecordV1::SessionDeclared(_) => kind == 1,
            worth_store_physical_format::BlobRecordV1::Chunk(_) => kind == 2,
            worth_store_physical_format::BlobRecordV1::SessionFrontier(_) => kind == 5,
            worth_store_physical_format::BlobRecordV1::SessionAbandoned(_) => kind == 6,
            _ => false,
        })
        .count()
}

pub(crate) fn assert_intact(root: &std::path::Path, scenario: &str) {
    let report = observe_closed_store_named(root, "c11-blob-reclaim", scenario);
    assert_eq!(report["completeness"], "complete", "{report}");
    let arena_rows: Vec<_> = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["family"] == "extent_arena_frame")
        .map(|row| format!("{}: {}", row["generation"], row["outcome"]))
        .collect();
    let arena_file_exists = root
        .join("families/records/arenas/arena-0000000000000001.data")
        .exists();
    for artifact in report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["family"]
                .as_str()
                .is_some_and(|family| family.starts_with("blob_"))
        })
    {
        assert_eq!(
            artifact["outcome"]["posture"],
            "intact",
            "scenario={scenario} arena_file_exists={arena_file_exists} arenas={arena_rows:?} artifact={artifact}"
        );
    }
}