use super::*;

#[test]
fn crash_gap_before_quarantine_leaves_reclaimable_orphan_and_original_reuse() {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.layout.dedupe.quarantine-gap");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let deadline =
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap();
    let original = vec![0x31; CHUNK_BYTES];
    let different = vec![0x72; CHUNK_BYTES];
    let serving = serving_from_initialization(root.path());
    let blobs = serving.blobs().unwrap();
    let first_object = blobs.issue_object_id(limits).unwrap();
    let first = worth_store::physical_runtime::BlobIngestDeclaration::new(
        first_object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
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
            Ok(worth_store_physical_format::BlobRecordV1::Chunk(chunk)) => {
                Some((persisted(*id), chunk.stored_digest()))
            }
            _ => None,
        })
        .unwrap();

    let conflicting_object = blobs.issue_object_id(limits).unwrap();
    let conflicting = worth_store::physical_runtime::BlobIngestDeclaration::new(
        conflicting_object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
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
        Err(worth_store::physical_runtime::BlobIngestFailure::CertificationQuarantineGap)
    ));
    drop(conflicting);
    let selected = selected_blob_records(&serving);
    let orphan = selected
        .iter()
        .find_map(|(id, frame)| match decode_blob_record(frame) {
            Ok(worth_store_physical_format::BlobRecordV1::Chunk(chunk))
                if chunk.occurrence().session() == conflict_session =>
            {
                Some(persisted(*id))
            }
            _ => None,
        })
        .expect("conflicting original was selected before the fault");
    assert!(selected.iter().all(|(_, frame)| !matches!(
        decode_blob_record(frame),
        Ok(worth_store_physical_format::BlobRecordV1::DedupeQuarantine(
            _
        ))
    )));
    serving.close();

    let reopened = serving_from_open(root.path());
    assert!(
        reopened
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::DedupeIndex)
            .is_ok(),
        "without a selected kind12 marker the existing dedupe basis remains usable"
    );
    let blobs = reopened.blobs().unwrap();
    let third_object = blobs.issue_object_id(limits).unwrap();
    let third = worth_store::physical_runtime::BlobIngestDeclaration::new(
        third_object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
        CHUNK_BYTES as u64,
        &scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(16).unwrap(),
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
            Ok(worth_store_physical_format::BlobRecordV1::ChunkReuseClaimV2(claim)) if claim.claim().destination_session() == third_session
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
    assert_eq!(
        receipt.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert!(receipt.dropped_records().contains(&orphan));
    let selected = selected_blob_records(&reopened);
    assert!(!selected.iter().any(|(id, _)| persisted(*id) == orphan));
    assert!(
        selected.iter().any(|(id, _)| persisted(*id) == source.0),
        "reclaim cannot retire the published shared source"
    );
    assert!(selected.iter().all(|(_, frame)| !matches!(
        decode_blob_record(frame),
        Ok(worth_store_physical_format::BlobRecordV1::DedupeQuarantine(
            _
        ))
    )));
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
