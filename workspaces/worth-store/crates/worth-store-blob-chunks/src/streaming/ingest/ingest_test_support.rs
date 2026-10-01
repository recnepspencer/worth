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

pub(super) fn run_ingest(
    frames: Vec<BlobStreamingSourceFrame>,
    window_bytes: u64,
) -> Result<BlobStreamingIngest, BlobStreamingIngestDenial> {
    run_ingest_with_window(frames, window_bytes, CounterEvidenceStrength::Exact)
}

pub(super) fn run_ingest_with_counter_strength(
    strength: CounterEvidenceStrength,
) -> Result<BlobStreamingIngest, BlobStreamingIngestDenial> {
    run_ingest_with_window(source_frames(3, 4), 4, strength)
}

fn run_ingest_with_window(
    frames: Vec<BlobStreamingSourceFrame>,
    window_bytes: u64,
    strength: CounterEvidenceStrength,
) -> Result<BlobStreamingIngest, BlobStreamingIngestDenial> {
    let window = BlobStreamingWindow::bounded(window_bytes)?;
    BlobStreamingIngest::verify_bounded_content(
        request(),
        window,
        pressure_admission(),
        strength,
        frames,
        &mut TestChunkWriter::new(),
    )
}

pub(super) fn run_ingest_with_pressure(
    pressure: BlobStreamingPressureAdmission,
) -> Result<BlobStreamingIngest, BlobStreamingIngestDenial> {
    BlobStreamingIngest::verify_bounded_content(
        request(),
        BlobStreamingWindow::bounded(4).unwrap(),
        pressure,
        CounterEvidenceStrength::Exact,
        source_frames(3, 4),
        &mut TestChunkWriter::new(),
    )
}

pub(super) fn request() -> BlobStreamingIngestRequest {
    request_for_total_bytes(12)
}

pub(super) fn request_for_total_bytes(total_bytes: u64) -> BlobStreamingIngestRequest {
    BlobStreamingIngestRequest::new(
        blob_scope("phase9.streaming", StoreTenantScope::TenantPhysicalBoundary),
        rule(),
        total_bytes,
    )
    .unwrap()
}

pub(super) fn source_frames(
    frame_bytes: usize,
    window_bytes: u64,
) -> Vec<BlobStreamingSourceFrame> {
    b"abcdefghijkl"
        .chunks(frame_bytes)
        .map(|chunk| source_frame(chunk, window_bytes))
        .collect()
}

pub(super) fn source_frame(bytes: &[u8], window_bytes: u64) -> BlobStreamingSourceFrame {
    BlobStreamingSourceFrame::from_bounded_bytes(
        bytes.to_vec(),
        BlobStreamingWindow::bounded(window_bytes).unwrap(),
    )
    .expect("bounded source frame should admit")
}

pub(super) fn pressure_admission() -> BlobStreamingPressureAdmission {
    BlobStreamingPressureAdmission::from_io_qos_background_capacity(
        blob_ingest_background_capacity_for_certification_test(background_budget()),
        0,
        false,
    )
    .expect("S.6 foreground page-write backed blob pressure should admit")
}

pub(super) fn background_budget() -> BackgroundResourceBudget {
    BackgroundResourceBudget::new().with_queue_slots(QueueSlot::new(1).unwrap())
}

fn rule() -> BlobChunkingRuleAdmission {
    BlobChunkingRuleAdmission::fixed_size(BlobChunkSize::from_bytes(4).unwrap()).unwrap()
}

pub(super) struct TestChunkWriter {
    ordinal_offset: u64,
}

impl TestChunkWriter {
    pub(super) const fn new() -> Self {
        Self { ordinal_offset: 0 }
    }

    pub(super) const fn with_ordinal_offset(ordinal_offset: u64) -> Self {
        Self { ordinal_offset }
    }
}

impl BlobStreamingChunkWriter for TestChunkWriter {
    fn write_streaming_chunk(
        &mut self,
        ordinal: BlobChunkOrdinal,
        bytes: &[u8],
    ) -> Result<BlobStreamingWrittenChunk, BlobStreamingIngestDenial> {
        let payload = physical_payload_for_bytes(bytes);
        let backend = BlobBackendChunkWriteSession::for_certification_test_authority()
            .observe_store_chunk_payload(ordinal.get() + self.ordinal_offset, &payload)
            .expect("backend chunk write should observe");
        BlobStreamingWrittenChunk::from_store_chunk_write(payload, backend)
    }
}

pub(super) struct SubstitutingChunkWriter;

impl BlobStreamingChunkWriter for SubstitutingChunkWriter {
    fn write_streaming_chunk(
        &mut self,
        ordinal: BlobChunkOrdinal,
        bytes: &[u8],
    ) -> Result<BlobStreamingWrittenChunk, BlobStreamingIngestDenial> {
        let replacement = vec![b'X'; bytes.len()];
        let payload = physical_payload_for_bytes(&replacement);
        let backend = BlobBackendChunkWriteSession::for_certification_test_authority()
            .observe_store_chunk_payload(ordinal.get(), &payload)
            .expect("backend chunk write should observe");
        BlobStreamingWrittenChunk::from_store_chunk_write(payload, backend)
    }
}
