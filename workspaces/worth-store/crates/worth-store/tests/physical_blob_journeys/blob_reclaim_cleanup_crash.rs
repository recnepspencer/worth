//! Process-kill proof for the V2 manifest-residue selector/Completed gap.

use std::{
    fs,
    num::{NonZeroU16, NonZeroU64},
    path::Path,
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobAppendFailure, BlobReclaimDisposition,
    BlobReclaimFailure, BlobReclaimLimits, BlobReclaimRequest, PhysicalMutationDeadline,
    PhysicalMutationProvenNoEffectCause, PhysicalRetirementDenial,
};
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, PersistedRecordIdentity};

use super::{
    blob_crash::{
        establish_recovery_frontier, kill_at, marker_path, recover_closed_store, write_marker,
    },
    blob_frontier::selected_blob_records,
    blob_reclaim::{abandoned_prefix, request},
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

const ROLE: &str = "crash-reclaim-cleanup-selector";
const SCOPE: &str = "c11.blob.reclaim.cleanup-selector.scope";

#[test]
fn selected_v2_cleanup_without_completed_reopens_and_retires_exact_metadata_once() {
    let world = kill_at(ROLE, Duration::from_secs(240));
    let metadata = fs::read(
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

    // The independent C8 executable must finish the typed cleanup Intent
    // whose selected selector was synced before the child was killed.
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
            Ok(BlobRecordV1::OriginalDropReserved(value)) if value.manifest_record() == manifest
        ) || matches!(
            decode_blob_record(bytes),
            Ok(BlobRecordV1::ReclaimDescriptor(value)) if value.manifest_record() == manifest
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
        Err(PhysicalRetirementDenial::Absent),
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
        Err(PhysicalRetirementDenial::Absent),
        "exact metadata extents must receive credit once only"
    );
    reopened.close();
}

pub(super) fn child(root: &Path) {
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
            assert!(manifest_gate.await_arrival(), "V2 manifest did not publish");
            let reservation_gate = serving
                .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeTerminalFinalization);
            manifest_gate.release();
            assert!(
                reservation_gate.await_arrival(),
                "OriginalDropReserved did not publish"
            );
            let drop_gate =
                serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeEffectCutover);
            reservation_gate.release();
            assert!(
                drop_gate.await_arrival(),
                "original drop did not reach pre-seal seam"
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
    let manifest_record = match result {
        Err(BlobReclaimFailure::ManifestRetained {
            manifest_record,
            cause: BlobAppendFailure::ProvenNoEffect(fate),
        }) if fate.cause()
            == PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal =>
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
                Ok(BlobRecordV1::DropSetManifestV2(value)) => value,
                other => panic!("retained record is not a V2 manifest: {other:?}"),
            })
        })
        .expect("V2 manifest remains selected");
    let reserved = selected
        .iter()
        .find_map(|(record, bytes)| match decode_blob_record(bytes) {
            Ok(BlobRecordV1::OriginalDropReserved(value))
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
    fs::write(
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
        assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
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

    let gate = serving.pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterRootReplacement);
    let marker = marker_path(root);
    thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(120);
            while Instant::now() < deadline {
                if gate.await_arrival() {
                    write_marker(
                        &marker,
                        [0xc1; 16],
                        token.encode()[24..40].try_into().unwrap(),
                    );
                    return;
                }
            }
            panic!("cleanup selector never reached namespace-synced seam");
        });
        let _ = serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope))
            .unwrap()
            .wait();
        panic!("cleanup escaped selected-root gate before process kill");
    });
}

fn persisted(record: worth_store::physical_runtime::PhysicalRecordId) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal()).unwrap()
}

fn append_identity(bytes: &mut Vec<u8>, identity: PersistedRecordIdentity) {
    bytes.extend_from_slice(&identity.allocation_epoch());
    bytes.extend_from_slice(&identity.ordinal().to_le_bytes());
}

fn identity(bytes: &[u8]) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new(
        bytes[..16].try_into().unwrap(),
        u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
    )
    .unwrap()
}
