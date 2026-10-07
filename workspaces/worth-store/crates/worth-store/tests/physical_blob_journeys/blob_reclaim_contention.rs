use std::{
    num::{NonZeroU16, NonZeroU64},
    thread,
    time::{Duration, Instant},
};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobAppendFailure, BlobReclaimDisposition,
    BlobReclaimFailure, BlobReclaimLimits, BlobReclaimRequest, BlobReclaimRetirement,
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess,
    PhysicalMutationProvenNoEffectCause, PhysicalMutationRequest, RecordAppendBatch,
};
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, PersistedRecordIdentity};

use super::{
    blob_abort::deadline,
    blob_crash::recover_closed_store,
    blob_frontier::selected_blob_records,
    blob_reclaim::{abandoned_prefix, request},
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

#[test]
fn ordinary_contender_cannot_write_wal_or_data_through_held_reclaim_fence() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reclaim.contender.scope");
    let token = abandoned_prefix(&serving, &scope);
    let reclaim = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x93; 32]))
        .unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                RecordAppendBatch::try_from_iter([b"ordinary-contender".as_slice()]).unwrap(),
                placement(),
                PhysicalMutationRequest::platform_durable(key, deadline()),
            )
            .into_raw()
    else {
        panic!("contender should reach actual managed pending admission");
    };
    let before = serving.media_counters();
    let outcome = prepared.execute();
    assert!(matches!(
        outcome,
        PhysicalMutationOutcome::ProvenNoEffect(fate)
            if fate.cause() == PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
    ));
    let after = serving.media_counters();
    assert_eq!(
        after.append_attempts(),
        before.append_attempts(),
        "fence denies before WAL append"
    );
    assert_eq!(
        after.positioned_write_attempts(),
        before.positioned_write_attempts(),
        "fence denies before data writes"
    );
    reclaim.cancel();
    assert!(
        serving.records().is_ok(),
        "cancellation releases the reader fence"
    );
    serving.close();
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn descriptor_pre_effect_denial_retains_manifest_and_releases_fence() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reclaim.partial.scope");
    let token = abandoned_prefix(&serving, &scope);
    let before = selected_blob_records(&serving);
    let reclaim = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap();
    // The first publication is the manifest. Intervene only after its root
    // advance, so the tightened retained-growth admission applies only to the
    // descriptor's pre-WAL planning, not the manifest publication.
    let gate =
        serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeTerminalFinalization);
    let result = thread::scope(|workers| {
        workers.spawn(|| {
            let until = Instant::now() + Duration::from_secs(60);
            while Instant::now() < until {
                if gate.await_arrival() {
                    serving.certification_limit_candidate_growth_bytes(1);
                    gate.release();
                    return;
                }
            }
            gate.release();
            panic!("manifest did not reach terminal-finalization seam");
        });
        reclaim.wait()
    });
    let retained = match result {
        Err(BlobReclaimFailure::ManifestRetained {
            manifest_record,
            cause: BlobAppendFailure::Preparation(_) | BlobAppendFailure::ProvenNoEffect(_),
        }) => manifest_record,
        other => panic!("expected known manifest-only partial outcome: {other:?}"),
    };
    let after = selected_blob_records(&serving);
    for retained in before {
        assert!(after.contains(&retained), "no payload was dropped");
    }
    assert_eq!(
        after
            .iter()
            .filter(|(_, bytes)| matches!(
                decode_blob_record(bytes),
                Ok(BlobRecordV1::DropSetManifestV2(_))
            ))
            .count(),
        1
    );
    assert!(after.iter().any(|(record, bytes)| {
        record.allocation_epoch() == retained.allocation_epoch()
            && record.ordinal() == retained.ordinal()
            && matches!(
                decode_blob_record(bytes),
                Ok(BlobRecordV1::DropSetManifestV2(_))
            )
    }));
    assert!(!after.iter().any(|(_, bytes)| matches!(
        decode_blob_record(bytes),
        Ok(BlobRecordV1::OriginalDropReserved(_))
    )));
    assert!(!after.iter().any(|(_, bytes)| matches!(
        decode_blob_record(bytes),
        Ok(BlobRecordV1::ReclaimDescriptor(_))
    )));
    serving.certification_limit_candidate_growth_bytes(8 * 1024 * 1024 - 64 * 1024);
    // Reinspection must be usable immediately; no restart disguises a stuck
    // fence or a lost session claim after this exactly known partial result.
    serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap()
        .cancel();
    serving.close();
}

#[cfg(feature = "certification-test-authority")]
#[test]
fn sealed_original_drop_no_effect_is_cleaned_through_ordinary_reclaim() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reclaim.manifest.cleanup.scope");
    let token = abandoned_prefix(&serving, &scope);
    let first = serving
        .blobs()
        .unwrap()
        .reclaim(BlobReclaimRequest::abandoned(
            token,
            &scope,
            placement(),
            PhysicalMutationDeadline::after_milliseconds(5_000).unwrap(),
            BlobReclaimLimits::new(
                NonZeroU64::new(128).unwrap(),
                NonZeroU64::new(8 << 20).unwrap(),
                NonZeroU16::new(1).unwrap(),
            )
            .unwrap(),
        ))
        .unwrap();
    let manifest_gate =
        serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeTerminalFinalization);
    let result = thread::scope(|workers| {
        workers.spawn(|| {
            assert!(manifest_gate.await_arrival(), "manifest did not publish");
            let reservation_gate = serving
                .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeTerminalFinalization);
            manifest_gate.release();
            assert!(
                reservation_gate.await_arrival(),
                "reservation did not publish"
            );
            let drop_gate =
                serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeEffectCutover);
            reservation_gate.release();
            assert!(
                drop_gate.await_arrival(),
                "drop did not reach pre-seal seam"
            );
            serving
                .certification_advance_physical_signal_clock(
                    worth_signal::facade::ClockAdvanceRequest::new(
                        worth_signal::facade::ClockDomain::MonotonicExecution,
                        worth_signal::facade::ClockTick::new(5_000),
                    ),
                )
                .unwrap();
            drop_gate.release();
        });
        first.wait()
    });
    let retained = match result {
        Err(BlobReclaimFailure::ManifestRetained {
            manifest_record,
            cause: BlobAppendFailure::ProvenNoEffect(fate),
        }) if fate.cause()
            == PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal =>
        {
            manifest_record
        }
        other => panic!("expected sealed original-drop no-effect: {other:?}"),
    };
    let selected = selected_blob_records(&serving);
    assert!(selected
        .iter()
        .any(
            |(record, bytes)| record.allocation_epoch() == retained.allocation_epoch()
                && record.ordinal() == retained.ordinal()
                && matches!(
                    decode_blob_record(bytes),
                    Ok(BlobRecordV1::DropSetManifestV2(_))
                )
        ));
    let reserved = selected
        .iter()
        .find_map(|(record, bytes)| match decode_blob_record(bytes) {
            Ok(BlobRecordV1::OriginalDropReserved(value))
                if value.manifest_record() == retained =>
            {
                PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal())
            }
            _ => None,
        })
        .expect("selected original reservation remains custody");
    // The original ProvenNoEffect binding is past the ordinary four-generation
    // retention horizon, yet selected V2 custody must pin it into every
    // completed checkpoint artifact and across a close/recovery/reopen.
    for ordinal in 0..6_u64 {
        let mut key = [0xc7; 32];
        key[..8].copy_from_slice(&ordinal.to_le_bytes());
        let request = PhysicalCheckpointRequest::fuzzy(
            PhysicalCheckpointIdempotencyKey::new(key),
            PhysicalCheckpointDeadline::after_milliseconds(120_000).unwrap(),
        );
        let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw()
        else {
            panic!("V2 pin checkpoint {ordinal} was not admitted");
        };
        assert!(
            matches!(handle.wait(), PhysicalCheckpointOutcome::Completed(_)),
            "V2 pin checkpoint {ordinal} did not complete"
        );
    }
    serving.close();
    recover_closed_store(directory.path());
    let serving = serving_from_open(directory.path());
    assert!(selected_blob_records(&serving)
        .iter()
        .any(|(record, bytes)| {
            record.allocation_epoch() == retained.allocation_epoch()
                && record.ordinal() == retained.ordinal()
                && matches!(
                    decode_blob_record(bytes),
                    Ok(BlobRecordV1::DropSetManifestV2(_))
                )
        }));
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
        assert_ne!(receipt.dropped_records(), &[retained]);
    }
    let cleanup = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(cleanup.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(cleanup.dropped_records().len(), 2);
    assert!(cleanup.dropped_records().contains(&retained));
    assert!(cleanup.dropped_records().contains(&reserved));
    assert_eq!(cleanup.remaining_payload_records(), 0);
    assert_eq!(cleanup.retirement(), BlobReclaimRetirement::Completed);
    assert!(cleanup.bytes_released() > 0);
    assert!(!selected_blob_records(&serving).iter().any(|(record, _)| {
        [retained, reserved].iter().any(|removed| {
            record.allocation_epoch() == removed.allocation_epoch()
                && record.ordinal() == removed.ordinal()
        })
    }));
    let repeated = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        repeated.disposition(),
        BlobReclaimDisposition::ProvenNoEffect
    );
    serving.close();
}
