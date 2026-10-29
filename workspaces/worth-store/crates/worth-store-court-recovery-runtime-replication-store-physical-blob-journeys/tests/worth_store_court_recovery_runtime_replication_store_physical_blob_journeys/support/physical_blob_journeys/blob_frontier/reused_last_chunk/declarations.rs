pub(crate) use {std::num::NonZeroU64, std::path::Path};

pub(crate) use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestSession, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalBlobFacade, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

pub(crate) use {worth_store_testkit_store::physical_blob_journeys::blob_frontier::assert_record_binding, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, worth_store_testkit_store::physical_blob_journeys::blob_frontier::CHUNK};
pub(crate) use {crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_with_limits, crate::physical_blob_journeys::blob_ingest_process::observed_without_damage, crate::physical_blob_journeys::blob_ingest_process::selected_root};
pub(crate) use {worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};

pub(crate) const REUSED_FILL: u8 = 63;

/// An undamaged observation in which the selected root, which routes the
/// reuse claims, and every blob claim are positively intact.
pub(crate) fn observed_clean(root: &Path, scenario: &str) -> Vec<serde_json::Value> {
    let artifacts = observed_without_damage(root, "c11-blob-frontier", scenario);
    for artifact in selected_root(&artifacts) {
        assert_eq!(artifact["outcome"]["posture"], "intact", "{artifact}");
    }
    for artifact in &artifacts {
        if artifact["family"]
            .as_str()
            .is_some_and(|family| family.starts_with("blob_"))
        {
            assert_eq!(artifact["outcome"]["posture"], "intact", "{artifact}");
        }
    }
    artifacts
}

pub(crate) fn count(artifacts: &[serde_json::Value], family: &str) -> usize {
    artifacts
        .iter()
        .filter(|row| row["family"] == family)
        .count()
}

pub(crate) fn publish_reused_chunk(serving: &ServingPhysicalRuntime, scope: &AdmittedBlobScope) {
    let blobs = serving.blobs().unwrap();
    let mut ingest = begin(&blobs, scope, 1);
    ingest.push(&[REUSED_FILL; CHUNK / 2]).unwrap();
    ingest.push(&[REUSED_FILL; CHUNK / 2]).unwrap();
    ingest.finish().unwrap();
}

pub(crate) fn begin<'runtime>(
    blobs: &PhysicalBlobFacade<'runtime>,
    scope: &AdmittedBlobScope,
    chunks: usize,
) -> BlobIngestSession<'runtime> {
    let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        blobs.issue_object_id(limits).unwrap(),
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (chunks * CHUNK) as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(300_000).unwrap(),
    )
    .unwrap();
    blobs
        .begin_ingest(declaration, placement(), (CHUNK / 2) as u64, limits)
        .unwrap()
}