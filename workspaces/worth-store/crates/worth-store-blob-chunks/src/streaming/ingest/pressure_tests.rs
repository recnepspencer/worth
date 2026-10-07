use worth_store_budgets::CounterEvidenceStrength;
use worth_store_io_scheduler::{
    blob_ingest_background_capacity_for_certification_test,
    blob_ingest_deferred_background_capacity_for_certification_test,
    blob_ingest_denied_background_capacity_for_certification_test,
    blob_ingest_throttled_background_capacity_for_certification_test,
    blob_ingest_zero_admitted_throttle_background_capacity_for_certification_test,
    checkpoint_flush_wal_background_capacity_for_certification_test, BackgroundCapacityAdmission,
    BackgroundIoPressureClass, BackgroundResourceBudget, QueueSlot,
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
fn non_blob_io_qos_background_capacity_cannot_enter_blob_ingest_pressure() {
    let denial = BlobStreamingPressureAdmission::from_io_qos_background_capacity(
        checkpoint_flush_wal_background_capacity_for_certification_test(background_budget()),
        1,
        false,
    )
    .expect_err("non-blob S.6 background capacity must not satisfy blob ingest pressure");

    assert_eq!(
        denial,
        BlobStreamingIngestDenial::BackgroundPressureClassMismatch {
            actual: BackgroundIoPressureClass::CheckpointFlush
        }
    );
}

#[test]
fn throttled_blob_pressure_carries_admitted_capacity_through_ingest() {
    let requested = two_slot_budget();
    let admitted = background_budget();
    let pressure = BlobStreamingPressureAdmission::from_io_qos_background_capacity(
        blob_ingest_throttled_background_capacity_for_certification_test(requested, admitted),
        0,
        false,
    )
    .expect("throttled S.6 blob pressure should admit into blob pressure");
    let ingest =
        run_ingest(pressure, source_frames()).expect("partial capacity should pace ingest");

    assert_eq!(ingest.counters().scheduler_throttles(), 1);
    assert_eq!(ingest.counters().scheduler_admissions(), 1);
    assert_eq!(ingest.counters().scheduler_yields(), 0);
    assert_eq!(ingest.counters().chunks_read(), 3);
}

#[test]
fn non_admitted_pressure_denies_before_source_consumption() {
    let deferred = deny_before_source(
        blob_ingest_deferred_background_capacity_for_certification_test(background_budget()),
        0,
        false,
    );
    assert!(matches!(
        deferred,
        BlobStreamingIngestDenial::BackgroundPressureDeferred { counters }
            if counters.denials() == 1
    ));

    let denied = deny_before_source(
        blob_ingest_denied_background_capacity_for_certification_test(
            three_slot_budget(),
            background_budget(),
            background_budget(),
        ),
        0,
        false,
    );
    assert!(matches!(
        denied,
        BlobStreamingIngestDenial::BackgroundPressureDenied { counters, .. }
            if counters.denials() == 1
    ));

    let yielded = deny_before_source(
        blob_ingest_background_capacity_for_certification_test(background_budget()),
        1,
        false,
    );
    assert!(matches!(
        yielded,
        BlobStreamingIngestDenial::BackgroundPressureYielded { counters }
            if counters.scheduler_yields() == 1 && counters.denials() == 1
    ));

    let zero_capacity_throttle = deny_before_source(
        blob_ingest_zero_admitted_throttle_background_capacity_for_certification_test(
            background_budget(),
        ),
        0,
        false,
    );
    assert!(matches!(
        zero_capacity_throttle,
        BlobStreamingIngestDenial::BackgroundPressureThrottledWithoutAdmittedCapacity {
            counters
        } if counters.scheduler_throttles() == 1
    ));

    let violation = deny_before_source(
        blob_ingest_background_capacity_for_certification_test(background_budget()),
        1,
        true,
    );
    assert!(matches!(
        violation,
        BlobStreamingIngestDenial::BackgroundPressureViolation { counters }
            if counters.denials() == 1
    ));
}

fn deny_before_source(
    capacity: BackgroundCapacityAdmission,
    foreground_pressure_events: u64,
    late_yield: bool,
) -> BlobStreamingIngestDenial {
    let pressure = BlobStreamingPressureAdmission::from_io_qos_background_capacity(
        capacity,
        foreground_pressure_events,
        late_yield,
    )
    .expect("class-correct S.6 blob pressure admission should build");
    run_ingest(pressure, PanicOnSourcePoll)
        .expect_err("non-admitted pressure must deny before polling source frames")
}

fn run_ingest(
    pressure: BlobStreamingPressureAdmission,
    source_frames: impl IntoIterator<Item = BlobStreamingSourceFrame>,
) -> Result<BlobStreamingIngest, BlobStreamingIngestDenial> {
    BlobStreamingIngest::verify_bounded_content(
        request(),
        BlobStreamingWindow::bounded(4).unwrap(),
        pressure,
        CounterEvidenceStrength::Exact,
        source_frames,
        &mut TestChunkWriter,
    )
}

fn request() -> BlobStreamingIngestRequest {
    BlobStreamingIngestRequest::new(
        blob_scope(
            "phase9.streaming.pressure",
            StoreTenantScope::TenantPhysicalBoundary,
        ),
        BlobChunkingRuleAdmission::fixed_size(BlobChunkSize::from_bytes(4).unwrap()).unwrap(),
        12,
    )
    .unwrap()
}

fn source_frames() -> Vec<BlobStreamingSourceFrame> {
    b"abcdefghijkl"
        .chunks(3)
        .map(|chunk| {
            BlobStreamingSourceFrame::from_bounded_bytes(
                chunk.to_vec(),
                BlobStreamingWindow::bounded(4).unwrap(),
            )
            .expect("bounded source frame should admit")
        })
        .collect()
}

fn background_budget() -> BackgroundResourceBudget {
    BackgroundResourceBudget::new().with_queue_slots(QueueSlot::new(1).unwrap())
}

fn two_slot_budget() -> BackgroundResourceBudget {
    BackgroundResourceBudget::new().with_queue_slots(QueueSlot::new(2).unwrap())
}

fn three_slot_budget() -> BackgroundResourceBudget {
    BackgroundResourceBudget::new().with_queue_slots(QueueSlot::new(3).unwrap())
}

struct PanicOnSourcePoll;

impl Iterator for PanicOnSourcePoll {
    type Item = BlobStreamingSourceFrame;

    fn next(&mut self) -> Option<Self::Item> {
        panic!("scheduler denial must occur before source consumption")
    }
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
