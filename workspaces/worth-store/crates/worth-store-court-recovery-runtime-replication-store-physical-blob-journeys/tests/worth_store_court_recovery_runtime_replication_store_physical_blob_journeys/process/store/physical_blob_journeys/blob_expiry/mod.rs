#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_expiry::*;
use {std::fs, std::num::NonZeroU64, std::path::Path};

use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store_blob_chunks::BlobChunkSize;
use {
    worth_store::physical_runtime::production::PhysicalCheckpointStep,
    worth_store::physical_runtime::AdmittedBlobScope,
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestClaimDenial,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobTerminalDisposition,
    worth_store::physical_runtime::BlobTerminalFailure,
    worth_store::physical_runtime::BlobTerminalLimits,
    worth_store::physical_runtime::CompletedPhysicalCheckpoint,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointHandle,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalWalReclamationObservation,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {
    worth_store_physical_format::decode_blob_record,
    worth_store_physical_format::BlobAbandonmentReasonV1,
    worth_store_physical_format::BlobRecordV1,
};

use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_abort::assert_resume_abandoned,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization_with_wal_segment_bytes,
    worth_store_testkit_store::fixture::serving_from_open_with_wal_segment_bytes,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

#[test]
fn only_completed_checkpoint_crossing_declared_max_can_publish_expiry() {
    let directory = tempfile::tempdir().unwrap();
    let serving =
        serving_from_initialization_with_wal_segment_bytes(directory.path(), WAL_SEGMENT_BYTES);
    let scope = admitted_blob_scope("c11.blob.expiry.completed.scope");
    let ingest = unfinished(&serving, &scope);
    let token = ingest.resume_token();
    let wire = token.encode();
    let declared_max = u64::from_le_bytes(wire[108..116].try_into().unwrap());
    assert_eq!(declared_max, 1, "fresh Store starts at checkpoint zero");
    let blobs = serving.blobs().unwrap();
    let before = selected_blob_records(&serving);
    let media_before = serving.media_counters();
    assert!(matches!(
        blobs.expire_ingest(token, &scope, placement(), deadline(), limits()),
        Err(worth_store::physical_runtime::BlobTerminalFailure::Claim(
            worth_store::physical_runtime::BlobIngestClaimDenial::CompetingSession
        ))
    ));
    let media_after_claim_denial = serving.media_counters();
    assert_eq!(
        media_after_claim_denial.append_attempts(),
        media_before.append_attempts()
    );
    assert_eq!(
        media_after_claim_denial.positioned_write_attempts(),
        media_before.positioned_write_attempts()
    );
    drop(ingest);

    assert_not_expired(&serving, token, &scope, 0, declared_max);
    let gate = serving.pause_physical_checkpoint_at(
        worth_store::physical_runtime::production::PhysicalCheckpointStep::CandidateSynchronization,
    );
    let pending = start_checkpoint(&serving, 1);
    assert!(
        gate.await_arrival(),
        "issued checkpoint reached prepublication pause"
    );
    assert_not_expired(&serving, token, &scope, 0, declared_max);
    gate.release();
    let first = completed(pending);
    assert_eq!(first.footer().identity().sequence().get(), declared_max);
    assert_not_expired(&serving, token, &scope, declared_max, declared_max);
    assert_eq!(selected_blob_records(&serving), before);

    let crossing = completed_checkpoint(&serving, 2);
    let crossing_sequence = crossing.footer().identity().sequence();
    assert!(crossing_sequence.get() > declared_max);
    let receipt = blobs
        .expire_ingest(token, &scope, placement(), deadline(), limits())
        .expect("completed durable checkpoint crosses original maximum");
    assert_eq!(
        receipt.disposition(),
        worth_store::physical_runtime::BlobTerminalDisposition::NewlyAbandoned
    );
    let after = selected_blob_records(&serving);
    assert_eq!(after.len(), before.len() + 1);
    for retained in &before {
        assert!(after.contains(retained), "expiry does not reclaim bytes");
    }
    let terminal = after
        .iter()
        .filter_map(|(_, bytes)| match decode_blob_record(bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::SessionAbandoned(value)) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(terminal.len(), 1);
    let expected_session: [u8; 16] = wire[24..40].try_into().unwrap();
    assert_eq!(terminal[0].session(), expected_session);
    assert_eq!(
        terminal[0].reason(),
        worth_store_physical_format::BlobAbandonmentReasonV1::CheckpointExpired {
            checkpoint_sequence: crossing_sequence,
        }
    );
    assert_resume_abandoned(&serving, token, &scope);
    let terminal_wal_segment = serving
        .record_submission()
        .wal_observation()
        .expect("live WAL observation")
        .segment();

    for ordinal in 0..5 {
        append_padding(&serving, ordinal);
    }
    let replacement = completed_checkpoint(&serving, 3);
    assert!(replacement.footer().identity().sequence() > crossing_sequence);
    let cutoff = replacement.binding_compaction().wal_cutoff_lsn_exclusive();
    assert_eq!(
        cutoff,
        replacement
            .retained_wal_tail()
            .durable_tail_end_lsn_exclusive()
            .get()
    );
    let retained_first = replacement
        .retained_wal_tail()
        .segments()
        .first()
        .expect("checkpoint retains its active WAL segment")
        .artifact()
        .segment()
        .get();
    assert!(
        retained_first > terminal_wal_segment,
        "replacement retained tail must exclude the terminal's WAL segment"
    );
    let worth_store::physical_runtime::PhysicalWalReclamationObservation::Reclaimed(reclamation) =
        replacement.wal_reclamation()
    else {
        panic!("replacement checkpoint must prune terminal-containing WAL prefix");
    };
    assert!(reclamation.reclaimed_segments() > 0);
    assert_eq!(reclamation.first_unreclaimed(), None);
    let repeated = blobs
        .expire_ingest(token, &scope, placement(), deadline(), limits())
        .unwrap();
    assert_eq!(repeated.record(), receipt.record());
    assert_eq!(
        repeated.disposition(),
        worth_store::physical_runtime::BlobTerminalDisposition::AlreadyAbandoned
    );
    assert_eq!(selected_blob_frames(&serving), after);
    serving.close();

    let before_c8 = assert_checkpoint_wal_range(directory.path(), cutoff, "before C8");
    let anchor = *before_c8
        .segment_facts()
        .last()
        .expect("completed checkpoint retains one WAL origin segment");
    assert_eq!(anchor.lsn_range().1, cutoff);
    let anchor_bytes =
        std::fs::read(wal_segment_path(directory.path(), anchor.identity())).unwrap();
    recover_closed_store(directory.path());
    let after_c8 = assert_checkpoint_wal_range(directory.path(), cutoff, "after C8");
    assert_eq!(after_c8.segment_facts(), &[anchor]);
    assert_eq!(
        std::fs::read(wal_segment_path(directory.path(), anchor.identity())).unwrap(),
        anchor_bytes,
        "C8 preserves the exact authenticated WAL origin bytes"
    );
    let reopened = serving_from_open_with_wal_segment_bytes(directory.path(), WAL_SEGMENT_BYTES);
    let reopened_wal = reopened.record_submission().wal_observation().unwrap();
    assert_eq!(
        (reopened_wal.segment(), reopened_wal.generation()),
        anchor.identity()
    );
    assert_eq!(reopened_wal.last_lsn_end(), Some(cutoff));
    assert_resume_abandoned(&reopened, token, &scope);
    assert_eq!(selected_blob_frames(&reopened), after);
    let repeated = reopened
        .blobs()
        .unwrap()
        .expire_ingest(token, &scope, placement(), deadline(), limits())
        .unwrap();
    assert_eq!(repeated.record(), receipt.record());
    assert_eq!(
        repeated.disposition(),
        worth_store::physical_runtime::BlobTerminalDisposition::AlreadyAbandoned
    );
    append_padding(&reopened, 6);
    let continued = reopened.record_submission().wal_observation().unwrap();
    assert_eq!(continued.last_lsn_end(), cutoff.checked_add(1));
    reopened.close();

    recover_closed_store(directory.path());
    let once = assert_checkpoint_wal_range(directory.path(), cutoff, "first repeated C8");
    assert_eq!(once.lsn_range().unwrap().1, cutoff + 1);
    recover_closed_store(directory.path());
    let twice = assert_checkpoint_wal_range(directory.path(), cutoff, "second repeated C8");
    assert_eq!(twice.segment_facts(), once.segment_facts());
    let continued_reopen =
        serving_from_open_with_wal_segment_bytes(directory.path(), WAL_SEGMENT_BYTES);
    assert_eq!(
        continued_reopen
            .record_submission()
            .wal_observation()
            .unwrap()
            .last_lsn_end(),
        Some(cutoff + 1)
    );
    assert_resume_abandoned(&continued_reopen, token, &scope);
    continued_reopen.close();

    let report = observe_closed_store_named(directory.path(), "c11-blob-expiry", "checkpoint");
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    assert_eq!(
        artifacts
            .iter()
            .filter(|row| row["family"] == "blob_resume_session")
            .count(),
        2
    );
    assert!(!artifacts
        .iter()
        .any(|row| row["family"] == "blob_generation_publication"));
    for row in artifacts.iter().filter(|row| {
        row["family"]
            .as_str()
            .is_some_and(|family| family.starts_with("blob_"))
    }) {
        assert_eq!(row["outcome"]["posture"], "intact", "{row}");
    }
}
