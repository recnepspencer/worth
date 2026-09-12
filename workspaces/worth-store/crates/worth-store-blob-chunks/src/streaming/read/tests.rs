use worth_store::physical_runtime::stability::{
    stable_physical_read_receipt_for_certification_test, PhysicalReadExecutionDenial,
    StablePhysicalReadExecutionCounters,
};
use worth_store::physical_runtime::PhysicalOperationAllocationScope;
use worth_store_budgets::CounterEvidenceStrength;
use worth_store_io_scheduler::{
    admit_background_pacing,
    foreground_reservation::{
        admitted_page_write_reservation_for_certification_test,
        admitted_point_read_reservation_for_certification_test,
        admitted_wal_write_reservation_for_certification_test,
    },
    verification_throttled_background_capacity_for_certification_test,
    BackgroundIdleCapacityLeaseRequest, BackgroundPacingOutcome, BackgroundResourceBudget,
    QueueSlot,
};

use worth_store_security::StoreTenantScope;

use crate::publication::test_support::publish_generation_with_bytes_and_chunk_size;
use crate::test_support::{
    admitted_multichunk_sequence_for_scope, blob_scope, physical_payload_for_bytes,
    with_blob_allocation,
};
use crate::{
    reject_full_blob_vec_as_streaming_read, BlobChunkByteRange, BlobChunkOrdinal,
    BlobCorruptionReferenceEdge, BlobCorruptionReferenceEdges, BlobQuarantineAuthority,
    BlobStreamingContentFrontier, BlobStreamingReadAdmission, BlobStreamingReadDenial,
    BlobStreamingReadObservation, BlobStreamingReadObservedChunk, BlobStreamingReadRequest,
    BlobStreamingReadWindow, BlobStreamingVerifiedRead,
};

#[test]
fn verified_read_retains_canonical_allocation_provenance_and_releases_authority() {
    with_blob_allocation(8, |serving, allocation| {
        let verified = BlobStreamingVerifiedRead::verify_bounded(
            request(),
            crate::BlobStreamingReadExecution::new(
                BlobStreamingReadWindow::bounded(8).unwrap(),
                allocation,
                admission(),
                BlobQuarantineAuthority::from_current_store_authority(
                    crate::lifecycle::generation_registry_test_support::current_authority(
                        "phase10.streaming.read.allocation",
                        "quarantine",
                    ),
                ),
                CounterEvidenceStrength::Exact,
            ),
            observations_for(b"abcdefghijkl", 8),
        )
        .unwrap();

        assert_eq!(
            serving
                .residency_observation()
                .counters()
                .active_operation_bytes_for(PhysicalOperationAllocationScope::Blob),
            0
        );
        let observed = verified.residency().allocation();
        assert_eq!(observed.store_identity(), serving.store_identity());
        assert_eq!(
            observed.store_generation(),
            serving.residency_observation().store_generation()
        );
        assert_eq!(observed.runtime_identity(), serving.runtime_identity());
        assert_eq!(observed.allocation_bytes(), 8);
        assert_eq!(verified.residency().peak_resident_bytes(), 4);
    });
}

const READ_CASE: &str = "phase10.streaming.read";
const READ_BYTES: &[u8] = b"abcdefghijkl";

#[test]
fn streaming_read_verification_is_independent_of_read_buffer_size() {
    let small = verify_with_window(4).expect("small read window should verify");
    let large = verify_with_window(8).expect("larger read window should verify");

    assert_eq!(small.chunk_tree_root(), large.chunk_tree_root());
    assert_eq!(
        small.logical_content_digest(),
        large.logical_content_digest()
    );
    assert_eq!(small.counters().bytes_read(), 12);
    assert_eq!(large.counters().bytes_read(), 12);
    assert_eq!(small.counters().chunks_read(), 3);
    assert_eq!(large.counters().chunks_read(), 3);
    assert_eq!(small.counters().chunks_verified(), 3);
    assert_eq!(large.counters().chunks_verified(), 3);
    assert_eq!(small.counters().chunk_checksum_verifications(), 3);
    assert_eq!(large.counters().chunk_checksum_verifications(), 3);
    assert_eq!(
        small.counters().counter_strength(),
        CounterEvidenceStrength::Exact
    );
    assert!(small
        .counter_backed_performance_receipt()
        .counter_rows()
        .iter()
        .any(|row| row.name().as_str().ends_with(".chunks_read") && row.observed_count() == 3));
}

#[test]
fn missing_reordered_corrupt_cold_and_whole_expected_paths_deny() {
    let missing = verify_observations(observations_for(b"abcdefghijkl", 4).into_iter().take(2))
        .expect_err("missing tail chunk must deny");
    assert!(matches!(
        missing,
        BlobStreamingReadDenial::MissingChunk { .. }
    ));

    let mut reordered = observations_for(b"abcdefghijkl", 4);
    reordered.swap(0, 1);
    let denial = verify_observations(reordered).expect_err("reordered chunks must deny");
    assert!(matches!(
        denial,
        BlobStreamingReadDenial::ReorderedChunk { .. }
    ));

    let corrupt = observations_for(b"abcdZZZZijkl", 4);
    let denial = verify_observations(corrupt).expect_err("corrupted chunk must deny");
    assert!(matches!(
        denial,
        BlobStreamingReadDenial::CorruptedChunk {
            damage_case,
            diagnostics,
            ..
        } if damage_case == crate::BlobDamageCase::ChecksumMismatch
            && diagnostics.quarantine().counters().quarantine_holds() == 1
            && diagnostics.quarantine().counters().read_detections() == 1
    ));

    let cold = [
        observations_for(b"abcdefghijkl", 4).remove(0),
        BlobStreamingReadObservation::cold_unavailable(
            BlobChunkOrdinal::first().next(),
            BlobChunkByteRange::new(4, 4).unwrap(),
        ),
    ];
    let denial = verify_observations(cold).expect_err("cold-unavailable chunk must deny");
    assert!(matches!(
        denial,
        BlobStreamingReadDenial::ColdChunkUnavailable { .. }
    ));

    let stale_read = verify_with_admission(stale_read_admission())
        .expect_err("stale read hold must deny before verified publication");
    assert!(matches!(
        stale_read,
        BlobStreamingReadDenial::StableReadBytesInsufficient {
            expected: 12,
            actual: 8,
            counters,
        } if counters.stale_read_denials() == 1
            && counters.counter_strength() == CounterEvidenceStrength::Exact
    ));

    assert_eq!(
        reject_full_blob_vec_as_streaming_read(b"abcdefghijkl".to_vec()),
        BlobStreamingReadDenial::WholeObjectExpectedBufferRejected { bytes: 12 }
    );
}

#[test]
fn point_page_and_wal_foreground_reservations_bind_read_admission() {
    for foreground in [
        admitted_point_read_reservation_for_certification_test(),
        admitted_page_write_reservation_for_certification_test(),
        admitted_wal_write_reservation_for_certification_test(),
    ] {
        let admission = BlobStreamingReadAdmission::from_stable_physical_read(
            stable_physical_read_receipt_for_certification_test(12),
            foreground,
            admitted_verification_pressure(),
        )
        .expect("foreground reservation should admit for blob read");
        let verified = verify_with_admission(admission).expect("reserved read should verify");
        assert_eq!(verified.counters().chunks_verified(), 3);
    }
}

#[test]
fn physical_read_denial_translation_preserves_exact_isolation_context() {
    let counters = StablePhysicalReadExecutionCounters::default()
        .with_blocking_io_event()
        .with_hidden_latch_io_denial();
    let denial =
        PhysicalReadExecutionDenial::HiddenStructuralLatchIoWithoutDeclaredCost { counters };

    let BlobStreamingReadDenial::StablePhysicalReadDenied(retained) =
        BlobStreamingReadAdmission::reject_physical_read_denial(denial.clone())
    else {
        panic!("physical read denial must remain a typed stable-read denial");
    };

    assert_eq!(*retained, denial);
}

#[test]
fn streaming_read_request_denies_unrelated_corruption_reference_edges() {
    let (published, visible) =
        publish_generation_with_bytes_and_chunk_size(READ_CASE, READ_BYTES, 4);
    let (unrelated_published, _) =
        publish_generation_with_bytes_and_chunk_size("phase10.streaming.unrelated", READ_BYTES, 4);
    let reference_edges = BlobCorruptionReferenceEdges::from_admitted_edges(&[
        BlobCorruptionReferenceEdge::from_reachability_staging_identity(
            published.staging_identity(),
        ),
        BlobCorruptionReferenceEdge::from_reachability_staging_identity(
            unrelated_published.staging_identity(),
        ),
    ])
    .expect("distinct edge witnesses should construct before request binding");

    let denied =
        BlobStreamingReadRequest::from_published_generation(visible, frontier(), reference_edges)
            .expect_err("unrelated affected edge must deny before read publication");

    assert!(matches!(
        denied,
        BlobStreamingReadDenial::CorruptionReferenceEdgeMismatch(_)
    ));
}

fn verify_with_window(
    window_bytes: u64,
) -> Result<BlobStreamingVerifiedRead, BlobStreamingReadDenial> {
    verify_observations_with_window(
        window_bytes,
        observations_for(b"abcdefghijkl", window_bytes),
    )
}

fn verify_observations(
    observations: impl IntoIterator<Item = BlobStreamingReadObservation>,
) -> Result<BlobStreamingVerifiedRead, BlobStreamingReadDenial> {
    verify_observations_with_window(4, observations)
}

fn verify_with_admission(
    admission: BlobStreamingReadAdmission,
) -> Result<BlobStreamingVerifiedRead, BlobStreamingReadDenial> {
    with_blob_allocation(8, |_, allocation| {
        BlobStreamingVerifiedRead::verify_bounded(
            request(),
            crate::BlobStreamingReadExecution::new(
                BlobStreamingReadWindow::bounded(8)?,
                allocation,
                admission,
                BlobQuarantineAuthority::from_current_store_authority(
                    crate::lifecycle::generation_registry_test_support::current_authority(
                        "phase10.streaming.read.quarantine",
                        "quarantine",
                    ),
                ),
                CounterEvidenceStrength::Exact,
            ),
            observations_for(b"abcdefghijkl", 8),
        )
    })
}

fn verify_observations_with_window(
    window_bytes: u64,
    observations: impl IntoIterator<Item = BlobStreamingReadObservation>,
) -> Result<BlobStreamingVerifiedRead, BlobStreamingReadDenial> {
    with_blob_allocation(window_bytes, |_, allocation| {
        BlobStreamingVerifiedRead::verify_bounded(
            request(),
            crate::BlobStreamingReadExecution::new(
                BlobStreamingReadWindow::bounded(window_bytes)?,
                allocation,
                admission(),
                BlobQuarantineAuthority::from_current_store_authority(
                    crate::lifecycle::generation_registry_test_support::current_authority(
                        "phase10.streaming.read.quarantine",
                        "quarantine",
                    ),
                ),
                CounterEvidenceStrength::Exact,
            ),
            observations,
        )
    })
}

fn request() -> BlobStreamingReadRequest {
    let (published, visible) =
        publish_generation_with_bytes_and_chunk_size(READ_CASE, READ_BYTES, 4);
    let reference_edges = BlobCorruptionReferenceEdges::from_reachability_staging_identity(
        published.staging_identity(),
    )
    .expect("published reachability staging identity should provide corruption reference edge");
    BlobStreamingReadRequest::from_published_generation(visible, frontier(), reference_edges)
        .expect("published generation should bind streaming read request")
}

fn frontier() -> BlobStreamingContentFrontier {
    let sequence = admitted_multichunk_sequence_for_scope(
        blob_scope(READ_CASE, StoreTenantScope::TenantPhysicalBoundary),
        READ_BYTES,
        4,
    );
    BlobStreamingContentFrontier::from_sequence(&sequence)
}

fn observations_for(bytes: &[u8], window_bytes: u64) -> Vec<BlobStreamingReadObservation> {
    bytes
        .chunks(4)
        .enumerate()
        .map(|(index, chunk)| {
            let ordinal = ordinal(index as u64);
            let observed = BlobStreamingReadObservedChunk::from_store_payload(
                ordinal,
                index as u64 * 4,
                physical_payload_for_bytes(chunk),
                BlobStreamingReadWindow::bounded(window_bytes).unwrap(),
            )
            .expect("bounded read chunk should admit");
            BlobStreamingReadObservation::from_chunk(observed)
        })
        .collect()
}

fn admission() -> BlobStreamingReadAdmission {
    BlobStreamingReadAdmission::from_stable_physical_read(
        stable_physical_read_receipt_for_certification_test(12),
        admitted_point_read_reservation_for_certification_test(),
        admitted_verification_pressure(),
    )
    .expect("stable physical read admission should bind")
}

fn stale_read_admission() -> BlobStreamingReadAdmission {
    BlobStreamingReadAdmission::from_stable_physical_read(
        stable_physical_read_receipt_for_certification_test(8),
        admitted_point_read_reservation_for_certification_test(),
        admitted_verification_pressure(),
    )
    .expect("stable read shortfall should deny during verification, not admission")
}

fn admitted_verification_pressure() -> BackgroundPacingOutcome {
    admit_background_pacing(BackgroundIdleCapacityLeaseRequest::new(
        verification_throttled_background_capacity_for_certification_test(
            read_pressure_budget(),
            read_pressure_budget(),
        ),
    ))
}

fn read_pressure_budget() -> BackgroundResourceBudget {
    BackgroundResourceBudget::new().with_queue_slots(QueueSlot::new(2).unwrap())
}

fn ordinal(value: u64) -> BlobChunkOrdinal {
    let mut ordinal = BlobChunkOrdinal::first();
    for _ in 0..value {
        ordinal = ordinal.next();
    }
    ordinal
}
