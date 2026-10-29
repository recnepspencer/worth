use {std::num::NonZeroU64};

use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalWorkCounterStage, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkPressureClass};
use {worth_store_blob_chunks::BlobChunkSize};
use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1};

use {worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization};

#[test]
fn selected_chunk_relocation_keeps_old_and_new_reads_byte_exact() {
    const CHUNK: usize = 256 * 1024;
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let blobs = serving.blobs().unwrap();
    let scope = admitted_blob_scope("c11.blob.relocation.selected");
    let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let payload = (0..CHUNK)
        .map(|index| ((index * 17 + 3) % 251) as u8)
        .collect::<Vec<_>>();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        CHUNK as u64 + 1,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, limits)
        .unwrap();
    ingest.push(&payload).unwrap();
    ingest.push(&[0xa5]).unwrap();
    let published = ingest.finish().unwrap();
    let before_records = selected_blob_records(&serving);
    let before_root = serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get();

    let mut old_read = blobs
        .read(published, &scope, 0, CHUNK as u64, limits)
        .unwrap();
    let hold = blobs
        .hold_chunk_for_relocation(published, &scope, 0, limits)
        .unwrap();
    let key = serving
        .record_submission()
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([151; 32]))
        .unwrap();
    let request = worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
        key,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    );
    let mut move_session = blobs
        .begin_chunk_relocation(hold, placement(), request)
        .unwrap();
    let compaction_before = serving.physical_work_counters().count_under_pressure(
        worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactPublication,
        worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundCompaction,
        worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
    );
    for _ in 0..512 {
        if move_session.progress().phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        move_session
            .advance()
            .unwrap_or_else(|failure| match failure {
                worth_store::physical_runtime::BlobMovementFailure::Copy(error) => {
                    panic!("copy at {:?}: {error:?}", move_session.progress().phase)
                }
                other => panic!("movement at {:?}: {other:?}", move_session.progress().phase),
            });
    }
    assert_eq!(
        move_session.progress().phase,
        worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
    );
    let receipt = move_session
        .publish()
        .unwrap_or_else(|failure| match failure {
            worth_store::physical_runtime::BlobMovementFailure::Indeterminate(value) => panic!(
                "indeterminate at {:?} after {} effects",
                value.stage(),
                value.completed_effect_count()
            ),
            other => panic!("publication failure: {other:?}"),
        });
    assert_eq!(receipt.logical_chunk_bytes(), CHUNK as u64);
    assert_eq!(receipt.root_generation(), before_root + 1);
    assert_eq!(selected_blob_records(&serving), before_records);
    assert!(
        serving.physical_work_counters().count_under_pressure(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactPublication,
            worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundBlobMovement,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ) > 0
    );
    assert_eq!(
        serving.physical_work_counters().count_under_pressure(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactPublication,
            worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundCompaction,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        compaction_before
    );
    assert!(before_records.iter().any(|(record, bytes)| {
        record.allocation_epoch() == receipt.record().allocation_epoch()
            && record.ordinal() == receipt.record().ordinal()
            && matches!(decode_blob_record(bytes), Ok(worth_store_physical_format::BlobRecordV1::Chunk(_)))
    }));

    let mut new_read = blobs
        .read(published, &scope, 0, CHUNK as u64, limits)
        .unwrap();
    let mut old_bytes = vec![0; CHUNK];
    let mut new_bytes = vec![0; CHUNK];
    assert_eq!(old_read.read_next(&mut old_bytes).unwrap(), CHUNK);
    assert_eq!(new_read.read_next(&mut new_bytes).unwrap(), CHUNK);
    assert_eq!(old_bytes, payload);
    assert_eq!(new_bytes, payload);
}
