//! Process-kill proof for the V2 manifest-residue selector/Completed gap.

pub(crate) use {
    std::fs, std::num::NonZeroU16, std::num::NonZeroU64, std::path::Path, std::thread,
    std::time::Duration, std::time::Instant,
};

pub(crate) use {
    worth_store::physical_runtime::production::PhysicalMutationCheckpoint,
    worth_store::physical_runtime::production::PhysicalMutationPauseGate,
    worth_store::physical_runtime::BlobAppendFailure,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimFailure,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
    worth_store::physical_runtime::PhysicalRetirementDenial,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::PersistedRecordIdentity,
};

pub(crate) use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_crash::marker_path,
    crate::physical_blob_journeys::blob_crash::write_marker,
    crate::physical_blob_journeys::blob_reclaim::abandoned_prefix,
    crate::physical_blob_journeys::blob_reclaim::request,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

pub(crate) const SELECTOR_ROLE: &str = "crash-reclaim-cleanup-selector";
pub(crate) const INTENT_ROLE: &str = "crash-reclaim-cleanup-intent";
pub(crate) const SCOPE: &str = "c11.blob.reclaim.cleanup-selector.scope";

pub(crate) fn assert_cleanup_reopens_and_retires_exact_metadata_once(role: &'static str) {
    let world = kill_at(role, std::time::Duration::from_secs(240));
    let metadata = std::fs::read(
        world
            .root
            .parent()
            .unwrap()
            .join("cleanup-metadata-identities"),
    )
    .unwrap();
    assert_eq!(
        metadata.len(),
        48,
        "child must bind exactly manifest and Reserved"
    );
    let manifest = identity(&metadata[..24]);
    let reserved = identity(&metadata[24..]);
    assert_ne!(manifest, reserved);

    // The independent C8 executable must finish the typed cleanup Intent,
    // whether or not its selected selector was synced before the kill.
    recover_closed_store(&world.root);
    let serving = serving_from_open(&world.root);
    let selected = selected_blob_records(&serving);
    assert!(
        !selected.iter().any(|(record, _)| {
            let identity = persisted(*record);
            identity == manifest || identity == reserved
        }),
        "recovered cleanup selector must remove both exact metadata records"
    );
    assert!(
        !selected.iter().any(|(_, bytes)| matches!(
            decode_blob_record(bytes),
            Ok(worth_store_physical_format::BlobRecordV1::OriginalDropReserved(value)) if value.manifest_record() == manifest
        ) || matches!(
            decode_blob_record(bytes),
            Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptor(value)) if value.manifest_record() == manifest
        )),
        "no selected metadata may still refer to the exact retired orphan manifest"
    );

    let before = serving.certification_charged_growth_bytes();
    serving.retire_displaced_segment().unwrap();
    let after_manifest = serving.certification_charged_growth_bytes();
    serving.retire_displaced_segment().unwrap();
    let after_reserved = serving.certification_charged_growth_bytes();
    assert!(
        before > after_manifest && after_manifest > after_reserved,
        "both displaced native metadata extents must receive separate positive credit"
    );
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent),
        "cleanup must reconstruct exactly two native obligations"
    );
    serving.close();

    recover_closed_store(&world.root);
    let reopened = serving_from_open(&world.root);
    assert_eq!(
        reopened.certification_charged_growth_bytes(),
        after_reserved,
        "second recovery cannot charge the retired extents again"
    );
    assert_eq!(
        reopened.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent),
        "exact metadata extents must receive credit once only"
    );
    reopened.close();
}

pub(crate) fn child(root: &Path, role: &str) {
    let serving = serving_from_initialization(root);
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope(SCOPE);
    let token = abandoned_prefix(&serving, &scope);

    // Publish the V2 manifest and OriginalDropReserved. Expire only the
    // original drop before group seal, yielding a sealed, discoverable
    // ProvenNoEffect rather than inferring fate from descriptor absence.
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
            await_cleanup_seam(&manifest_gate, "V2 manifest");
            let reservation_gate = serving
                .pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeTerminalFinalization);
            manifest_gate.release();
            await_cleanup_seam(&reservation_gate, "OriginalDropReserved");
            let drop_gate =
                serving.pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeEffectCutover);
            reservation_gate.release();
            await_cleanup_seam(&drop_gate, "original drop before group seal");
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
    let manifest_record = match result {
        Err(worth_store::physical_runtime::BlobReclaimFailure::ManifestRetained {
            manifest_record,
            cause: worth_store::physical_runtime::BlobAppendFailure::ProvenNoEffect(fate),
        }) if fate.cause()
            == worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal =>
        {
            manifest_record
        }
        other => panic!("fixture needs the sealed original-drop no-effect: {other:?}"),
    };
    let selected = selected_blob_records(&serving);
    let manifest = selected
        .iter()
        .find_map(|(record, bytes)| {
            (persisted(*record) == manifest_record).then(|| match decode_blob_record(bytes) {
                Ok(worth_store_physical_format::BlobRecordV1::DropSetManifestV2(value)) => value,
                other => panic!("retained record is not a V2 manifest: {other:?}"),
            })
        })
        .expect("V2 manifest remains selected");
    let reserved = selected
        .iter()
        .find_map(|(record, bytes)| match decode_blob_record(bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::OriginalDropReserved(value))
                if value.manifest_record() == manifest_record =>
            {
                Some(persisted(*record))
            }
            _ => None,
        })
        .expect("exact Reserved frame remains selected");
    assert_eq!(manifest.count(), 1);
    assert_eq!(manifest.dropped().len(), 1);
    let mut identities = Vec::with_capacity(48);
    append_identity(&mut identities, manifest_record);
    append_identity(&mut identities, reserved);
    std::fs::write(
        root.parent().unwrap().join("cleanup-metadata-identities"),
        identities,
    )
    .unwrap();

    // The three one-record drops remove only payload; metadata is still
    // selected and the fourth reclaim must enter V2 residue maintenance.
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
        assert!(!receipt.dropped_records().contains(&manifest_record));
        assert!(!receipt.dropped_records().contains(&reserved));
    }
    let selected = selected_blob_records(&serving);
    assert!(selected
        .iter()
        .any(|(record, _)| persisted(*record) == manifest_record));
    assert!(selected
        .iter()
        .any(|(record, _)| persisted(*record) == reserved));
    assert!(!selected
        .iter()
        .any(|(record, _)| manifest.dropped().contains(&persisted(*record))));

    let gate = serving.pause_physical_mutation_at(match role {
        SELECTOR_ROLE => worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterRootReplacement,
        INTENT_ROLE => worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability,
        _ => panic!("unknown cleanup crash seam: {role}"),
    });
    let marker = marker_path(root);
    std::thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
            while std::time::Instant::now() < deadline {
                if gate.await_arrival() {
                    write_marker(
                        &marker,
                        [0xc1; 16],
                        token.encode()[24..40].try_into().unwrap(),
                    );
                    return;
                }
            }
            panic!("cleanup never reached its {role} seam");
        });
        let _ = serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope))
            .unwrap()
            .wait();
        panic!("cleanup escaped its {role} gate before process kill");
    });
}

pub(crate) fn persisted(
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new(
        record.allocation_epoch(),
        record.ordinal(),
    )
    .unwrap()
}

pub(crate) fn append_identity(bytes: &mut Vec<u8>, identity: PersistedRecordIdentity) {
    bytes.extend_from_slice(&identity.allocation_epoch());
    bytes.extend_from_slice(&identity.ordinal().to_le_bytes());
}

pub(crate) fn identity(bytes: &[u8]) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new(
        bytes[..16].try_into().unwrap(),
        u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
    )
    .unwrap()
}

// The gate's five-second wait is a notification window, not a claim about
// scheduling a durable mutation on a loaded CI host. Keep the exact seam and
// simulated-clock expiry, with a bounded aggregate arrival wait.
pub(crate) fn await_cleanup_seam(gate: &PhysicalMutationPauseGate, phase: &str) {
    let started = std::time::Instant::now();
    while !gate.await_arrival() {
        if started.elapsed() >= std::time::Duration::from_secs(30) {
            gate.release();
            panic!(
                "cleanup stalled at {phase} ({:?}) after {:?}",
                gate.checkpoint(),
                started.elapsed()
            );
        }
    }
}
