use worth_store_budgets::CounterEvidenceStrength;
use worth_store_io_scheduler::{
    blob_ingest_background_capacity_for_certification_test, BackgroundResourceBudget, QueueSlot,
};
use worth_store_physical_backend::BlobBackendChunkWriteSession;
use worth_store_security::StoreTenantScope;

use crate::test_support::{blob_scope, physical_payload_for_bytes};
use crate::{
    BlobChunkOrdinal, BlobChunkSize, BlobChunkingRuleAdmission, BlobStreamingChunkWriter,
    BlobStreamingIngest, BlobStreamingIngestDenial, BlobStreamingIngestRequest,
    BlobStreamingPressureAdmission, BlobStreamingSourceFrame, BlobStreamingWindow,
    BlobStreamingWrittenChunk,
};

#[test]
fn semantic_result_and_bounded_window_are_stable_across_source_window_sizes() {
    let small = run_ingest(source_frames(3, 4), 4).unwrap();
    let large = run_ingest(source_frames(5, 8), 8).unwrap();

    assert_eq!(
        small.sequence().chunk_identity_summary(),
        large.sequence().chunk_identity_summary()
    );
    assert_eq!(small.counters().peak_resident_bytes(), 4);
    assert_eq!(large.counters().peak_resident_bytes(), 4);
    assert_eq!(
        small.frontier().chunk_tree_root(),
        large.frontier().chunk_tree_root()
    );
    assert_eq!(
        small.frontier().logical_content_digest(),
        large.frontier().logical_content_digest()
    );
    assert_eq!(
        small.counters().chunks_read(),
        large.counters().chunks_read()
    );
    assert_eq!(
        small.counters().chunks_written(),
        large.counters().chunks_written()
    );
    assert_eq!(
        small.counters().backend_write_observations(),
        large.counters().backend_write_observations()
    );
}

fn run_ingest(
    frames: Vec<BlobStreamingSourceFrame>,
    window_bytes: u64,
) -> Result<BlobStreamingIngest, BlobStreamingIngestDenial> {
    BlobStreamingIngest::verify_bounded_content(
        request(),
        BlobStreamingWindow::bounded(window_bytes)?,
        pressure_admission(),
        CounterEvidenceStrength::Exact,
        frames,
        &mut TestChunkWriter,
    )
}

fn request() -> BlobStreamingIngestRequest {
    BlobStreamingIngestRequest::new(
        blob_scope(
            "phase9.streaming.equivalence",
            StoreTenantScope::TenantPhysicalBoundary,
        ),
        BlobChunkingRuleAdmission::fixed_size(BlobChunkSize::from_bytes(4).unwrap()).unwrap(),
        12,
    )
    .unwrap()
}

fn source_frames(frame_bytes: usize, window_bytes: u64) -> Vec<BlobStreamingSourceFrame> {
    b"abcdefghijkl"
        .chunks(frame_bytes)
        .map(|chunk| {
            BlobStreamingSourceFrame::from_bounded_bytes(
                chunk.to_vec(),
                BlobStreamingWindow::bounded(window_bytes).unwrap(),
            )
            .expect("bounded source frame should admit")
        })
        .collect()
}

fn pressure_admission() -> BlobStreamingPressureAdmission {
    BlobStreamingPressureAdmission::from_io_qos_background_capacity(
        blob_ingest_background_capacity_for_certification_test(background_budget()),
        0,
        false,
    )
    .expect("S.6 blob pressure should admit")
}

fn background_budget() -> BackgroundResourceBudget {
    BackgroundResourceBudget::new().with_queue_slots(QueueSlot::new(1).unwrap())
}

struct TestChunkWriter;

impl BlobStreamingChunkWriter for TestChunkWriter {
    fn write_streaming_chunk(
        &mut self,
        ordinal: BlobChunkOrdinal,
        bytes: &[u8],
    ) -> Result<BlobStreamingWrittenChunk, BlobStreamingIngestDenial> {
        let payload = physical_payload_for_bytes(bytes);
        let backend = BlobBackendChunkWriteSession::for_certification_test_authority()
            .observe_store_chunk_payload(ordinal.get(), &payload)
            .expect("backend chunk write should observe");
        BlobStreamingWrittenChunk::from_store_chunk_write(payload, backend)
    }
}
