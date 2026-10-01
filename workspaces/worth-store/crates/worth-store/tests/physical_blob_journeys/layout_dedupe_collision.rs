#![cfg(feature = "certification-test-authority")]

use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobDedupeFailure, BlobIngestDeclaration, BlobIngestFailure,
    BlobReadLimits, BlobReclaimDeferral, BlobReclaimDisposition, BlobReclaimFailure,
    LayoutRebuildLimits, PhysicalLayoutDenial, PhysicalMutationDeadline,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    decode_blob_record, BlobChunkFrameV1, BlobChunkOccurrenceV1, BlobRecordV1,
    DecodedBlobChunkFrameV1,
};

#[path = "layout_dedupe_collision/selected_evidence.rs"]
mod selected_evidence;
use selected_evidence::{persisted, selected_chunk};

use super::{
    blob_abort::limits as terminal_limits,
    blob_crash::establish_recovery_frontier,
    blob_frontier::selected_blob_records,
    blob_ingest_process::observe_closed_store_named,
    blob_reclaim::request as reclaim_request,
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

const CHUNK_BYTES: usize = 64 * 1024;

#[test]
fn forced_unequal_candidate_is_durably_quarantined_and_rebuild_never_reuses_its_key() {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.layout.dedupe.collision");
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let deadline = PhysicalMutationDeadline::after_milliseconds(30_000).unwrap();
    let original = vec![0x31; CHUNK_BYTES];
    let different = vec![0x72; CHUNK_BYTES];
    let digest = {
        let occurrence = BlobChunkOccurrenceV1::new([1; 16], [2; 16], 0).unwrap();
        let frame = BlobChunkFrameV1::encode(occurrence, CHUNK_BYTES as u32, &original).unwrap();
        DecodedBlobChunkFrameV1::decode(&frame)
            .unwrap()
            .stored_digest()
    };

    let serving = serving_from_initialization(root.path());
    let blobs = serving.blobs().unwrap();
    let first_object = blobs.issue_object_id(limits).unwrap();
    let first = BlobIngestDeclaration::new(
        first_object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline,
    )
    .unwrap();
    let mut first = blobs
        .begin_ingest(first, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap();
    first.push(&original[..CHUNK_BYTES / 2]).unwrap();
    first.push(&original[CHUNK_BYTES / 2..]).unwrap();
    first.finish().unwrap();

    let conflicting_object = blobs.issue_object_id(limits).unwrap();
    let conflicting = BlobIngestDeclaration::new(
        conflicting_object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline,
    )
    .unwrap();
    let mut conflicting = blobs
        .begin_ingest(conflicting, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap();
    let conflicting_token = conflicting.resume_token();
    assert!(conflicting.certification_force_next_dedupe_lookup_digest(digest));
    conflicting.push(&different[..CHUNK_BYTES / 2]).unwrap();
    assert!(matches!(
        conflicting.push(&different[CHUNK_BYTES / 2..]),
        Err(BlobIngestFailure::Dedupe(BlobDedupeFailure::DigestCollisionDenied {
            digest: disputed, ..
        })) if disputed == digest
    ));
    drop(conflicting);

    let selected = selected_blob_records(&serving);
    let quarantines = selected
        .iter()
        .filter_map(|(_, bytes)| match decode_blob_record(bytes) {
            Ok(BlobRecordV1::DedupeQuarantine(value)) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        quarantines.len(),
        1,
        "one selected quarantine, not just an in-memory denial"
    );
    let quarantine = quarantines[0];
    assert_eq!(quarantine.disputed_digest(), digest);
    let source = selected_chunk(&selected, quarantine.source_chunk());
    let conflict = selected_chunk(&selected, quarantine.conflicting_chunk());
    assert_eq!(source.bytes(), original.as_slice());
    assert_eq!(conflict.bytes(), different.as_slice());
    assert_ne!(source.bytes(), conflict.bytes());
    assert_ne!(
        source.stored_digest(),
        conflict.stored_digest(),
        "the certification seam redirects only the lookup key, not canonical chunk hashing"
    );
    establish_recovery_frontier(&serving);
    blobs
        .abort_ingest(
            conflicting_token,
            &scope,
            placement(),
            deadline,
            terminal_limits(),
        )
        .unwrap();
    let before_reclaim = selected_blob_records(&serving);
    assert!(matches!(
        blobs.reclaim(reclaim_request(conflicting_token, &scope)),
        Err(BlobReclaimFailure::Deferred(
            BlobReclaimDeferral::SharedReferences
        ))
    ));
    let after_reclaim = selected_blob_records(&serving);
    assert_eq!(
        after_reclaim, before_reclaim,
        "shared-reference deferral is pre-effect"
    );
    assert!(after_reclaim
        .iter()
        .any(|(record, _)| persisted(*record) == quarantine.source_chunk()));
    assert!(after_reclaim
        .iter()
        .any(|(record, _)| persisted(*record) == quarantine.conflicting_chunk()));
    assert!(after_reclaim.iter().any(|(_, frame)| matches!(
        decode_blob_record(frame),
        Ok(BlobRecordV1::DedupeQuarantine(_))
    )));
    drop(blobs);
    assert!(matches!(
        serving
            .layouts()
            .unwrap()
            .btree(DurableArtifactFamilyId::DedupeIndex),
        Err(PhysicalLayoutDenial::DedupeQuarantinePending { .. })
    ));
    serving.close();

    let reopened = serving_from_open(root.path());
    assert!(matches!(
        reopened
            .layouts()
            .unwrap()
            .btree(DurableArtifactFamilyId::DedupeIndex),
        Err(PhysicalLayoutDenial::DedupeQuarantinePending { .. })
    ));
    let rebuilt = reopened
        .layouts()
        .unwrap()
        .rebuild(
            DurableArtifactFamilyId::DedupeIndex,
            LayoutRebuildLimits::new(
                NonZeroU64::new(1_000).unwrap(),
                NonZeroU64::new(1_000).unwrap(),
            ),
            placement(),
            deadline,
        )
        .unwrap();
    assert!(rebuilt.validated_chunks() >= 1);
    reopened
        .layouts()
        .unwrap()
        .btree(DurableArtifactFamilyId::DedupeIndex)
        .unwrap();

    let blobs = reopened.blobs().unwrap();
    let third_object = blobs.issue_object_id(limits).unwrap();
    let third = BlobIngestDeclaration::new(
        third_object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline,
    )
    .unwrap();
    let mut third = blobs
        .begin_ingest(third, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap();
    let third_session = third.session_id().bytes();
    assert!(third.certification_force_next_dedupe_lookup_digest(digest));
    third.push(&different[..CHUNK_BYTES / 2]).unwrap();
    third.push(&different[CHUNK_BYTES / 2..]).unwrap();
    third.finish().unwrap();
    let selected = selected_blob_records(&reopened);
    assert_eq!(
        selected
            .iter()
            .filter(|(_, bytes)| match decode_blob_record(bytes) {
                Ok(BlobRecordV1::Chunk(chunk)) => chunk.occurrence().session() == third_session,
                _ => false,
            })
            .count(),
        1
    );
    assert_eq!(
        selected
            .iter()
            .filter(|(_, bytes)| match decode_blob_record(bytes) {
                Ok(BlobRecordV1::ChunkReuseClaim(claim)) =>
                    claim.destination_session() == third_session,
                Ok(BlobRecordV1::ChunkReuseClaimV2(claim)) =>
                    claim.claim().destination_session() == third_session,
                _ => false,
            })
            .count(),
        0
    );
    reopened.close();

    let report = observe_closed_store_named(
        root.path(),
        "c11-dedupe-quarantine",
        "forced-unequal-candidate-reopen-rebuild",
    );
    let quarantines = report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|artifact| artifact["family"] == "blob_dedupe_quarantine")
        .collect::<Vec<_>>();
    assert_eq!(quarantines.len(), 1, "{report}");
    assert_eq!(quarantines[0]["outcome"]["posture"], "intact", "{report}");
}

#[test]
fn crash_gap_before_quarantine_leaves_reclaimable_orphan_and_original_reuse() {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.layout.dedupe.quarantine-gap");
    let limits = BlobReadLimits::new(NonZeroU64::new(256).unwrap());
    let deadline = PhysicalMutationDeadline::after_milliseconds(30_000).unwrap();
    let original = vec![0x31; CHUNK_BYTES];
    let different = vec![0x72; CHUNK_BYTES];
    let serving = serving_from_initialization(root.path());
    let blobs = serving.blobs().unwrap();
    let first_object = blobs.issue_object_id(limits).unwrap();
    let first = BlobIngestDeclaration::new(
        first_object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline,
    )
    .unwrap();
    let mut first = blobs
        .begin_ingest(first, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap();
    first.push(&original[..CHUNK_BYTES / 2]).unwrap();
    first.push(&original[CHUNK_BYTES / 2..]).unwrap();
    first.finish().unwrap();
    let selected = selected_blob_records(&serving);
    let source = selected
        .iter()
        .find_map(|(id, frame)| match decode_blob_record(frame) {
            Ok(BlobRecordV1::Chunk(chunk)) => Some((persisted(*id), chunk.stored_digest())),
            _ => None,
        })
        .unwrap();

    let conflicting_object = blobs.issue_object_id(limits).unwrap();
    let conflicting = BlobIngestDeclaration::new(
        conflicting_object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline,
    )
    .unwrap();
    let mut conflicting = blobs
        .begin_ingest(conflicting, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap();
    let token = conflicting.resume_token();
    let conflict_session = conflicting.session_id().bytes();
    assert!(conflicting.certification_force_next_dedupe_lookup_digest(source.1));
    conflicting.certification_fail_before_dedupe_quarantine();
    conflicting.push(&different[..CHUNK_BYTES / 2]).unwrap();
    assert!(matches!(
        conflicting.push(&different[CHUNK_BYTES / 2..]),
        Err(BlobIngestFailure::CertificationQuarantineGap)
    ));
    drop(conflicting);
    drop(blobs);
    let selected = selected_blob_records(&serving);
    let orphan = selected
        .iter()
        .find_map(|(id, frame)| match decode_blob_record(frame) {
            Ok(BlobRecordV1::Chunk(chunk)) if chunk.occurrence().session() == conflict_session => {
                Some(persisted(*id))
            }
            _ => None,
        })
        .expect("conflicting original was selected before the fault");
    assert!(selected.iter().all(|(_, frame)| !matches!(
        decode_blob_record(frame),
        Ok(BlobRecordV1::DedupeQuarantine(_))
    )));
    serving.close();

    let reopened = serving_from_open(root.path());
    assert!(
        reopened
            .layouts()
            .unwrap()
            .btree(DurableArtifactFamilyId::DedupeIndex)
            .is_ok(),
        "without a selected kind12 marker the existing dedupe basis remains usable"
    );
    let blobs = reopened.blobs().unwrap();
    let third_object = blobs.issue_object_id(limits).unwrap();
    let third = BlobIngestDeclaration::new(
        third_object,
        BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline,
    )
    .unwrap();
    let mut third = blobs
        .begin_ingest(third, placement(), (CHUNK_BYTES / 2) as u64, limits)
        .unwrap();
    let third_session = third.session_id().bytes();
    third.push(&original[..CHUNK_BYTES / 2]).unwrap();
    third.push(&original[CHUNK_BYTES / 2..]).unwrap();
    third.finish().unwrap();
    assert!(selected_blob_records(&reopened)
        .iter()
        .any(|(_, frame)| matches!(
            decode_blob_record(frame),
            Ok(BlobRecordV1::ChunkReuseClaimV2(claim)) if claim.claim().destination_session() == third_session
                && claim.claim().selected_chunk() == source.0
        )));

    establish_recovery_frontier(&reopened);
    blobs
        .abort_ingest(token, &scope, placement(), deadline, terminal_limits())
        .unwrap();
    let receipt = blobs
        .reclaim(reclaim_request(token, &scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    assert!(receipt.dropped_records().contains(&orphan));
    let selected = selected_blob_records(&reopened);
    assert!(!selected.iter().any(|(id, _)| persisted(*id) == orphan));
    assert!(
        selected.iter().any(|(id, _)| persisted(*id) == source.0),
        "reclaim cannot retire the published shared source"
    );
    assert!(selected.iter().all(|(_, frame)| !matches!(
        decode_blob_record(frame),
        Ok(BlobRecordV1::DedupeQuarantine(_))
    )));
    drop(blobs);
    reopened.close();

    let report = observe_closed_store_named(
        root.path(),
        "c11-dedupe-quarantine-gap",
        "orphan-reclaimed-without-marker",
    );
    assert_eq!(report["completeness"], "complete", "{report}");
    assert!(!report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|artifact| artifact["family"] == "blob_dedupe_quarantine"));
}
