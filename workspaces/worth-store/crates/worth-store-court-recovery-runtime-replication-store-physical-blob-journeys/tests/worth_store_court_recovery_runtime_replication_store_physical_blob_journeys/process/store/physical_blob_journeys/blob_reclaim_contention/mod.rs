use {
    std::num::NonZeroU16, std::num::NonZeroU64, std::thread, std::time::Duration,
    std::time::Instant,
};

use worth_proof::TransitionOutcome;
use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::BlobAppendFailure,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::BlobReclaimRetirement,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::RecordAppendBatch,
};
use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_abort::deadline,
    crate::physical_blob_journeys::blob_reclaim::abandoned_prefix,
    crate::physical_blob_journeys::blob_reclaim::request,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
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
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x93; 32]),
        )
        .unwrap();
    let worth_proof::TransitionOutcome::Success(
        worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
    ) = submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"ordinary-contender".as_slice(),
            ])
            .unwrap(),
            placement(),
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                deadline(),
            ),
        )
        .into_raw()
    else {
        panic!("contender should reach actual managed pending admission");
    };
    let before = serving.media_counters();
    let outcome = prepared.execute();
    assert!(matches!(
        outcome,
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate)
            if fate.cause() == worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::AdmissionDeniedBeforeGroupSeal
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

#[cfg(all())]
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
        serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeTerminalFinalization);
    let result = std::thread::scope(|workers| {
        workers.spawn(|| {
            let until = std::time::Instant::now() + std::time::Duration::from_secs(60);
            while std::time::Instant::now() < until {
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
        Err(worth_store::physical_runtime::BlobReclaimFailure::ManifestRetained {
            manifest_record,
            cause:
                worth_store::physical_runtime::BlobAppendFailure::Preparation(_)
                | worth_store::physical_runtime::BlobAppendFailure::ProvenNoEffect(_),
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
                Ok(worth_store_physical_format::BlobRecordV1::DropSetManifestV2(_))
            ))
            .count(),
        1
    );
    assert!(after.iter().any(|(record, bytes)| {
        record.allocation_epoch() == retained.allocation_epoch()
            && record.ordinal() == retained.ordinal()
            && matches!(
                decode_blob_record(bytes),
                Ok(worth_store_physical_format::BlobRecordV1::DropSetManifestV2(_))
            )
    }));
    assert!(!after.iter().any(|(_, bytes)| matches!(
        decode_blob_record(bytes),
        Ok(worth_store_physical_format::BlobRecordV1::OriginalDropReserved(_))
    )));
    assert!(!after.iter().any(|(_, bytes)| matches!(
        decode_blob_record(bytes),
        Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptor(_))
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

#[cfg(all())]
#[test]
fn sealed_original_drop_no_effect_is_cleaned_through_ordinary_reclaim() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.reclaim.manifest.cleanup.scope");
    let token = abandoned_prefix(&serving, &scope);
    let first = serving
        .blobs()
        .unwrap()
        .reclaim(
            worth_store::physical_runtime::BlobReclaimRequest::abandoned(
                token,
                &scope,
                placement(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(5_000)
                    .unwrap(),
                worth_store::physical_runtime::BlobReclaimLimits::new(
                    std::num::NonZeroU64::new(128).unwrap(),
                    std::num::NonZeroU64::new(8 << 20).unwrap(),
                    std::num::NonZeroU16::new(1).unwrap(),
                )
                .unwrap(),
            ),
        )
        .unwrap();
    let manifest_gate =
        serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeTerminalFinalization);
    let result = std::thread::scope(|workers| {
        workers.spawn(|| {
            require_pause_arrival(&manifest_gate, "manifest did not publish");
            let reservation_gate = serving
                .pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeTerminalFinalization);
            manifest_gate.release();
            require_pause_arrival(&reservation_gate, "reservation did not publish");
            let drop_gate =
                serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeEffectCutover);
            reservation_gate.release();
            require_pause_arrival(&drop_gate, "drop did not reach pre-seal seam");
            let advance = serving.certification_advance_physical_signal_clock(
                worth_signal::facade::ClockAdvanceRequest::new(
                    worth_signal::facade::ClockDomain::MonotonicExecution,
                    worth_signal::facade::ClockTick::new(5_000),
                ),
            );
            drop_gate.release();
            advance.unwrap();
        });
        first.wait()
    });
    let retained = match result {
        Err(worth_store::physical_runtime::BlobReclaimFailure::ManifestRetained {
            manifest_record,
            cause: worth_store::physical_runtime::BlobAppendFailure::ProvenNoEffect(fate),
        }) if fate.cause()
            == worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal =>
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
                    Ok(worth_store_physical_format::BlobRecordV1::DropSetManifestV2(_))
                )
        ));
    let reserved = selected
        .iter()
        .find_map(|(record, bytes)| match decode_blob_record(bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::OriginalDropReserved(value))
                if value.manifest_record() == retained =>
            {
                worth_store_physical_format::PersistedRecordIdentity::new(
                    record.allocation_epoch(),
                    record.ordinal(),
                )
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
        let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
            worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
            worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(120_000)
                .unwrap(),
        );
        let worth_proof::TransitionOutcome::Success(handle) =
            serving.checkpoints().start(request).into_raw()
        else {
            panic!("V2 pin checkpoint {ordinal} was not admitted");
        };
        assert!(
            matches!(
                handle.wait(),
                worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
            ),
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
                    Ok(worth_store_physical_format::BlobRecordV1::DropSetManifestV2(_))
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
        assert_eq!(
            receipt.disposition(),
            worth_store::physical_runtime::BlobReclaimDisposition::Dropped
        );
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
    assert_eq!(
        cleanup.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert_eq!(cleanup.dropped_records().len(), 2);
    assert!(cleanup.dropped_records().contains(&retained));
    assert!(cleanup.dropped_records().contains(&reserved));
    assert_eq!(cleanup.remaining_payload_records(), 0);
    assert_eq!(
        cleanup.retirement(),
        worth_store::physical_runtime::BlobReclaimRetirement::Completed
    );
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
        worth_store::physical_runtime::BlobReclaimDisposition::ProvenNoEffect
    );
    serving.close();
}

/// Arrival is a scheduling guard, not a five-second product latency claim.
/// Match the neighboring orphan court's bounded arrival envelope, and release
/// before failing so a late worker cannot strand the scope in `first.wait()`.
#[cfg(all())]
fn require_pause_arrival(
    gate: &worth_store::physical_runtime::production::PhysicalMutationPauseGate,
    message: &str,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while std::time::Instant::now() < deadline {
        if gate.await_arrival() {
            return;
        }
    }
    gate.release();
    panic!("{message}");
}
