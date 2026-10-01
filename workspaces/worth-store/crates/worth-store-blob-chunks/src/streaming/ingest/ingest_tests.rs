use worth_store_budgets::CounterEvidenceStrength;
use worth_store_io_scheduler::{
    blob_ingest_background_capacity_for_certification_test,
    blob_ingest_wal_write_background_capacity_for_certification_test,
    foreground_reservation::admitted_point_read_reservation_for_certification_test,
};
use worth_store_physical_backend::BlobBackendChunkWriteObservation;

use crate::test_support::physical_payload_for_bytes;
use crate::{
    reject_full_blob_vec_as_streaming_ingest, reject_scalar_backend_api_as_streaming_ingest,
    BlobStreamingIngest, BlobStreamingIngestDenial, BlobStreamingPressureAdmission,
    BlobStreamingWindow, BlobStreamingWrittenChunk,
};

use super::ingest_test_support::*;

#[test]
fn bounded_window_size_drives_source_streaming_without_changing_chunk_sequence() {
    let small = run_ingest(source_frames(3, 4), 4).unwrap();
    let large = run_ingest(source_frames(5, 8), 8).unwrap();

    assert_eq!(
        small.sequence().chunk_identity_summary(),
        large.sequence().chunk_identity_summary()
    );
    assert_eq!(
        small.frontier().chunk_tree_root(),
        large.frontier().chunk_tree_root()
    );
    assert_eq!(
        small.frontier().logical_content_digest(),
        large.frontier().logical_content_digest()
    );
    assert_eq!(small.counters().bytes_streamed(), 12);
    assert_eq!(large.counters().bytes_streamed(), 12);
    assert_eq!(small.counters().windows_observed(), 4);
    assert_eq!(large.counters().windows_observed(), 3);
    assert_eq!(small.counters().chunks_read(), 3);
    assert_eq!(large.counters().chunks_read(), 3);
    assert_eq!(small.counters().chunks_written(), 3);
    assert_eq!(large.counters().chunks_written(), 3);
    assert_eq!(small.counters().backend_write_observations(), 3);
    assert_eq!(large.counters().backend_write_observations(), 3);
    assert_eq!(small.counters().peak_resident_bytes(), 4);
    assert_eq!(large.counters().peak_resident_bytes(), 4);
    assert_eq!(small.counters().scheduler_yields(), 0);
    assert_eq!(small.counters().scheduler_waits(), 1);
    assert!(small
        .counter_backed_performance_receipt()
        .counter_rows()
        .iter()
        .any(|row| row.name().as_str().ends_with(".chunks_read") && row.observed_count() == 3));
    assert!(small
        .counter_backed_performance_receipt()
        .counter_rows()
        .iter()
        .any(|row| row.name().as_str().ends_with(".scheduler_waits") && row.observed_count() == 1));
}

#[test]
fn whole_object_scalar_and_missing_counter_shortcuts_are_denied() {
    assert_eq!(
        reject_full_blob_vec_as_streaming_ingest(b"whole-object".to_vec()),
        BlobStreamingIngestDenial::WholeObjectMaterializationRejected { bytes: 12 }
    );

    let scalar = BlobBackendChunkWriteObservation::reject_scalar_framed_record_api(4);
    assert_eq!(
        reject_scalar_backend_api_as_streaming_ingest(scalar),
        BlobStreamingIngestDenial::ScalarBackendCertificationRejected
    );

    let missing_exact = run_ingest_with_counter_strength(CounterEvidenceStrength::Sampled)
        .expect_err("sampled counters cannot prove streaming residency");
    assert!(matches!(
        missing_exact,
        BlobStreamingIngestDenial::MissingExactCounters {
            actual: CounterEvidenceStrength::Sampled
        }
    ));

    let scalar_chunk = BlobStreamingWrittenChunk::from_store_chunk_write(
        physical_payload_for_bytes(b"abcd"),
        BlobBackendChunkWriteObservation::reject_scalar_framed_record_api(4),
    );
    assert_eq!(
        scalar_chunk,
        Err(BlobStreamingIngestDenial::ScalarBackendCertificationRejected)
    );
}

#[test]
fn pressure_violation_denies_before_blob_ingest_consumes_source_frames() {
    let pressure = BlobStreamingPressureAdmission::from_io_qos_background_capacity(
        blob_ingest_background_capacity_for_certification_test(background_budget()),
        1,
        true,
    )
    .expect("S.6 pressure admission should build");
    let denial = BlobStreamingIngest::verify_bounded_content(
        request(),
        BlobStreamingWindow::bounded(4).unwrap(),
        pressure,
        CounterEvidenceStrength::Exact,
        source_frames(3, 4),
        &mut TestChunkWriter::new(),
    )
    .expect_err("S.6 pressure violation must deny blob ingest");
    assert!(matches!(
        denial,
        BlobStreamingIngestDenial::BackgroundPressureViolation { counters }
            if counters.denials() == 1
    ));
}

#[test]
fn full_object_source_frame_and_unbound_read_reservation_pressure_are_denied() {
    let whole_object = BlobStreamingIngest::verify_bounded_content(
        request_for_total_bytes(4),
        BlobStreamingWindow::bounded(4).unwrap(),
        pressure_admission(),
        CounterEvidenceStrength::Exact,
        [source_frame(b"abcd", 4)],
        &mut TestChunkWriter::new(),
    )
    .expect_err("source frame cannot materialize the whole object");
    assert_eq!(
        whole_object,
        BlobStreamingIngestDenial::WholeObjectMaterializationRejected { bytes: 4 }
    );

    let read_lane = BlobStreamingPressureAdmission::reject_unbound_foreground_reservation(
        admitted_point_read_reservation_for_certification_test(),
    );
    assert!(matches!(
        read_lane,
        BlobStreamingIngestDenial::ForegroundReservationLaneMismatch { .. }
    ));
}

#[test]
fn backend_write_observations_must_match_chunk_order_and_bytes() {
    let denial = BlobStreamingIngest::verify_bounded_content(
        request(),
        BlobStreamingWindow::bounded(4).unwrap(),
        pressure_admission(),
        CounterEvidenceStrength::Exact,
        source_frames(3, 4),
        &mut TestChunkWriter::with_ordinal_offset(1),
    )
    .expect_err("mismatched backend ordinal must deny");

    assert_eq!(
        denial,
        BlobStreamingIngestDenial::BackendWriteOrdinalMismatch {
            expected: 0,
            actual: 1
        }
    );
}

#[test]
fn backend_writer_payload_must_match_pending_source_bytes() {
    let denial = BlobStreamingIngest::verify_bounded_content(
        request(),
        BlobStreamingWindow::bounded(4).unwrap(),
        pressure_admission(),
        CounterEvidenceStrength::Exact,
        source_frames(3, 4),
        &mut SubstitutingChunkWriter,
    )
    .expect_err("writer-chosen payload bytes cannot replace streamed source bytes");

    assert_eq!(
        denial,
        BlobStreamingIngestDenial::BackendWritePayloadMismatch { ordinal: 0 }
    );
}

#[test]
fn blob_ingest_pressure_admits_against_wal_foreground_reservation() {
    let pressure = BlobStreamingPressureAdmission::from_io_qos_background_capacity(
        blob_ingest_wal_write_background_capacity_for_certification_test(background_budget()),
        0,
        false,
    )
    .expect("S.6 foreground WAL backed blob pressure should admit");
    let ingest =
        run_ingest_with_pressure(pressure).expect("WAL-backed ingest pressure should admit");
    assert_eq!(ingest.counters().scheduler_yields(), 0);
    assert_eq!(ingest.counters().scheduler_admissions(), 1);
    assert_eq!(ingest.counters().scheduler_waits(), 1);
}
