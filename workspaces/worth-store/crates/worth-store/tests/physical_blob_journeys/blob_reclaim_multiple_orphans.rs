//! Two independently terminal custody attempts must clean one at a time.

use std::{
    num::{NonZeroU16, NonZeroU64},
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::{
    production::{PhysicalMutationCheckpoint, PhysicalMutationPauseGate},
    BlobAppendFailure, BlobReclaimDisposition, BlobReclaimFailure, BlobReclaimLimits,
    BlobReclaimRequest, PhysicalMutationDeadline, PhysicalMutationProvenNoEffectCause,
};
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, PersistedRecordIdentity};

use super::{
    blob_frontier::selected_blob_records,
    blob_reclaim::{abandoned_prefix, request},
    fixture::{admitted_blob_scope, placement, serving_from_initialization},
};

#[test]
fn two_terminal_orphans_clean_in_two_bounded_reclaims() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reclaim.two-orphans.scope");
    let token = abandoned_prefix(&serving, &scope);
    let mut orphans = Vec::new();

    for iteration in 0..2 {
        let first = serving
            .blobs()
            .unwrap()
            .reclaim(BlobReclaimRequest::abandoned(
                token,
                &scope,
                placement(),
                PhysicalMutationDeadline::after_milliseconds((iteration + 1) * 5_000).unwrap(),
                BlobReclaimLimits::new(
                    NonZeroU64::new(128).unwrap(),
                    NonZeroU64::new(8 << 20).unwrap(),
                    NonZeroU16::new(1).unwrap(),
                )
                .unwrap(),
            ))
            .unwrap();
        let manifest_gate = serving
            .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeTerminalFinalization);
        let (result, reached) = thread::scope(|workers| {
            let worker = workers.spawn(|| {
                if !arrives(&manifest_gate) {
                    return "manifest";
                }
                let reservation_gate = serving.pause_physical_mutation_at(
                    PhysicalMutationCheckpoint::BeforeTerminalFinalization,
                );
                manifest_gate.release();
                if !arrives(&reservation_gate) {
                    return "reservation";
                }
                let drop_gate = serving
                    .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeEffectCutover);
                reservation_gate.release();
                if !arrives(&drop_gate) {
                    return "drop";
                }
                serving
                    .certification_advance_physical_signal_clock(
                        worth_signal::facade::ClockAdvanceRequest::new(
                            worth_signal::facade::ClockDomain::MonotonicExecution,
                            worth_signal::facade::ClockTick::new((iteration + 1) * 5_000),
                        ),
                    )
                    .unwrap();
                drop_gate.release();
                "complete"
            });
            let result = first.wait();
            (result, worker.join().unwrap())
        });
        assert_eq!(
            reached, "complete",
            "attempt {iteration} ended at {reached}: {result:?}"
        );
        let manifest = match result {
            Err(BlobReclaimFailure::ManifestRetained {
                manifest_record,
                cause: BlobAppendFailure::ProvenNoEffect(fate),
            }) if fate.cause()
                == PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal =>
            {
                manifest_record
            }
            other => panic!("fixture requires exact sealed no-effect: {other:?}"),
        };
        let reserved = selected_blob_records(&serving)
            .iter()
            .find_map(|(record, bytes)| match decode_blob_record(bytes) {
                Ok(BlobRecordV1::OriginalDropReserved(value))
                    if value.manifest_record() == manifest =>
                {
                    PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal())
                }
                _ => None,
            })
            .expect("selected exact Reserved frame");
        assert!(!orphans.iter().any(|(old, _)| *old == manifest));
        orphans.push((manifest, reserved));
    }

    for remaining in [2, 1, 0] {
        let receipt = serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope))
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
        assert_eq!(receipt.remaining_payload_records(), remaining);
    }
    // With payload gone, admission scans the same selected C.5 roster twice
    // for session custody and four more times for one metadata cleanup. The
    // separately authenticated declaration read charges its 156-byte ceiling.
    let selected = selected_blob_records(&serving);
    let selected_bytes = selected
        .iter()
        .map(|(_, frame)| frame.len() as u64)
        .sum::<u64>();
    let expected_inspected = 156 + 6 * selected_bytes;
    let expected_records = 1 + 6 * selected.len() as u64;
    let observed = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap();
    assert_eq!(
        observed.observation().inspected_payload_bytes(),
        expected_inspected
    );
    assert_eq!(observed.observation().inspected_records(), expected_records);
    assert_eq!(observed.cancel(), BlobReclaimDisposition::ProvenNoEffect);
    let root_before = serving.records().unwrap().protected_root().root();
    let appends_before = serving.media_counters().append_attempts();
    let writes_before = serving.media_counters().positioned_write_attempts();
    let tight = BlobReclaimRequest::abandoned(
        token,
        &scope,
        placement(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(expected_inspected - 1).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    assert!(matches!(
        serving.blobs().unwrap().reclaim(tight),
        Err(BlobReclaimFailure::InspectedByteBoundExhausted)
    ));
    assert_eq!(
        serving.records().unwrap().protected_root().root(),
        root_before
    );
    assert_eq!(serving.media_counters().append_attempts(), appends_before);
    assert_eq!(
        serving.media_counters().positioned_write_attempts(),
        writes_before
    );
    orphans.sort_unstable_by_key(|(manifest, _)| *manifest);
    for (manifest, reserved) in orphans {
        let cleanup = serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope))
            .unwrap()
            .wait()
            .unwrap();
        assert_eq!(cleanup.disposition(), BlobReclaimDisposition::Dropped);
        assert_eq!(cleanup.remaining_payload_records(), 0);
        assert_eq!(cleanup.dropped_records().len(), 2);
        assert!(cleanup.dropped_records().contains(&manifest));
        assert!(cleanup.dropped_records().contains(&reserved));
        assert!(!selected_blob_records(&serving).iter().any(|(record, _)| {
            let selected =
                PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal()).unwrap();
            selected == manifest || selected == reserved
        }));
    }
    assert_eq!(
        serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope))
            .unwrap()
            .wait()
            .unwrap()
            .disposition(),
        BlobReclaimDisposition::ProvenNoEffect,
    );
    serving.close();
}

/// Waits for the paused mutation to arrive at `gate`. A single bounded wait can
/// expire under parallel load and leave the gate holding the mutation with no
/// one to release it; the same long deadline the crash journeys use keeps the
/// arrival deterministic.
fn arrives(gate: &PhysicalMutationPauseGate) -> bool {
    let deadline = Instant::now() + Duration::from_secs(120);
    while Instant::now() < deadline {
        if gate.await_arrival() {
            return true;
        }
    }
    false
}
