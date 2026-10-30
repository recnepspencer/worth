#[allow(unused_imports)]
use crate::physical_blob_journeys::layout_dedupe_collision::*;

use worth_store_physical_format::decode_blob_record;

use {
    crate::layout_dedupe_collision::selected_evidence::persisted,
    crate::layout_dedupe_collision::selected_evidence::selected_chunk,
};

use {
    crate::physical_blob_journeys::blob_abort::limits as terminal_limits,
    crate::physical_blob_journeys::blob_crash::establish_recovery_frontier,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    crate::physical_blob_journeys::blob_reclaim::request as reclaim_request,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

#[test]
fn forced_unequal_candidate_is_durably_quarantined_and_rebuild_never_reuses_its_key() {
    let root = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.layout.dedupe.collision");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(256).unwrap());
    let deadline =
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap();
    let original = vec![0x31; CHUNK_BYTES];
    let different = vec![0x72; CHUNK_BYTES];
    let digest = {
        let occurrence =
            worth_store_physical_format::BlobChunkOccurrenceV1::new([1; 16], [2; 16], 0).unwrap();
        let frame = worth_store_physical_format::BlobChunkFrameV1::encode(
            occurrence,
            CHUNK_BYTES as u32,
            &original,
        )
        .unwrap();
        worth_store_physical_format::DecodedBlobChunkFrameV1::decode(&frame)
            .unwrap()
            .stored_digest()
    };

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
    let conflicting_token = conflicting.resume_token();
    assert!(conflicting.certification_force_next_dedupe_lookup_digest(digest));
    conflicting.push(&different[..CHUNK_BYTES / 2]).unwrap();
    assert!(matches!(
        conflicting.push(&different[CHUNK_BYTES / 2..]),
        Err(worth_store::physical_runtime::BlobIngestFailure::Dedupe(worth_store::physical_runtime::BlobDedupeFailure::DigestCollisionDenied {
            digest: disputed, ..
        })) if disputed == digest
    ));
    drop(conflicting);

    let selected = selected_blob_records(&serving);
    let quarantines = selected
        .iter()
        .filter_map(|(_, bytes)| match decode_blob_record(bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::DedupeQuarantine(value)) => Some(value),
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
        Err(worth_store::physical_runtime::BlobReclaimFailure::Deferred(
            worth_store::physical_runtime::BlobReclaimDeferral::SharedReferences
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
        Ok(worth_store_physical_format::BlobRecordV1::DedupeQuarantine(
            _
        ))
    )));
    assert!(matches!(
        serving
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::DedupeIndex),
        Err(worth_store::physical_runtime::PhysicalLayoutDenial::DedupeQuarantinePending { .. })
    ));
    serving.close();

    let reopened = serving_from_open(root.path());
    assert!(matches!(
        reopened
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::DedupeIndex),
        Err(worth_store::physical_runtime::PhysicalLayoutDenial::DedupeQuarantinePending { .. })
    ));
    let rebuilt = reopened
        .layouts()
        .unwrap()
        .rebuild(
            worth_store_contracts::DurableArtifactFamilyId::DedupeIndex,
            worth_store::physical_runtime::LayoutRebuildLimits::new(
                std::num::NonZeroU64::new(1_000).unwrap(),
                std::num::NonZeroU64::new(1_000).unwrap(),
            ),
            placement(),
            deadline,
        )
        .unwrap();
    assert!(rebuilt.validated_chunks() >= 1);
    reopened
        .layouts()
        .unwrap()
        .btree(worth_store_contracts::DurableArtifactFamilyId::DedupeIndex)
        .unwrap();

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
    assert!(third.certification_force_next_dedupe_lookup_digest(digest));
    third.push(&different[..CHUNK_BYTES / 2]).unwrap();
    third.push(&different[CHUNK_BYTES / 2..]).unwrap();
    third.finish().unwrap();
    let selected = selected_blob_records(&reopened);
    assert_eq!(
        selected
            .iter()
            .filter(|(_, bytes)| match decode_blob_record(bytes) {
                Ok(worth_store_physical_format::BlobRecordV1::Chunk(chunk)) =>
                    chunk.occurrence().session() == third_session,
                _ => false,
            })
            .count(),
        1
    );
    assert_eq!(
        selected
            .iter()
            .filter(|(_, bytes)| match decode_blob_record(bytes) {
                Ok(worth_store_physical_format::BlobRecordV1::ChunkReuseClaim(claim)) =>
                    claim.destination_session() == third_session,
                Ok(worth_store_physical_format::BlobRecordV1::ChunkReuseClaimV2(claim)) =>
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

#[path = "quarantine_crash_gap.rs"]
mod quarantine_crash_gap;
