use std::{fs, num::NonZeroU64, path::Path};

use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    production::PhysicalCheckpointStep, AdmittedBlobScope, BlobCheckpointLimit,
    BlobIngestClaimDenial, BlobIngestDeclaration, BlobReadLimits, BlobTerminalDisposition,
    BlobTerminalFailure, BlobTerminalLimits, CompletedPhysicalCheckpoint,
    PhysicalCheckpointDeadline, PhysicalCheckpointHandle, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointRequest, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalWalReclamationObservation,
    RecordAppendBatch, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{decode_blob_record, BlobAbandonmentReasonV1, BlobRecordV1};

#[path = "../physical_record_journeys/durability_admission/independent_wal_oracle/segment_inventory.rs"]
pub(crate) mod wal_oracle;

use super::{
    blob_abort::assert_resume_abandoned,
    blob_crash::recover_closed_store,
    blob_frontier::selected_blob_records,
    blob_ingest_process::observe_closed_store_named,
    fixture::{
        admitted_blob_scope, placement, serving_from_initialization_with_wal_segment_bytes,
        serving_from_open_with_wal_segment_bytes,
    },
};

const CHUNK: usize = 64 * 1024;
const WAL_SEGMENT_BYTES: u64 = 256 * 1024;

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
        Err(BlobTerminalFailure::Claim(
            BlobIngestClaimDenial::CompetingSession
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
    let gate =
        serving.pause_physical_checkpoint_at(PhysicalCheckpointStep::CandidateSynchronization);
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
        BlobTerminalDisposition::NewlyAbandoned
    );
    let after = selected_blob_records(&serving);
    assert_eq!(after.len(), before.len() + 1);
    for retained in &before {
        assert!(after.contains(retained), "expiry does not reclaim bytes");
    }
    let terminal = after
        .iter()
        .filter_map(|(_, bytes)| match decode_blob_record(bytes) {
            Ok(BlobRecordV1::SessionAbandoned(value)) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(terminal.len(), 1);
    let expected_session: [u8; 16] = wire[24..40].try_into().unwrap();
    assert_eq!(terminal[0].session(), expected_session);
    assert_eq!(
        terminal[0].reason(),
        BlobAbandonmentReasonV1::CheckpointExpired {
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
    let PhysicalWalReclamationObservation::Reclaimed(reclamation) = replacement.wal_reclamation()
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
        BlobTerminalDisposition::AlreadyAbandoned
    );
    assert_eq!(selected_blob_frames(&serving), after);
    drop(blobs);
    serving.close();

    let before_c8 = assert_checkpoint_wal_range(directory.path(), cutoff, "before C8");
    let anchor = *before_c8
        .segment_facts()
        .last()
        .expect("completed checkpoint retains one WAL origin segment");
    assert_eq!(anchor.lsn_range().1, cutoff);
    let anchor_bytes = fs::read(wal_segment_path(directory.path(), anchor.identity())).unwrap();
    recover_closed_store(directory.path());
    let after_c8 = assert_checkpoint_wal_range(directory.path(), cutoff, "after C8");
    assert_eq!(after_c8.segment_facts(), &[anchor]);
    assert_eq!(
        fs::read(wal_segment_path(directory.path(), anchor.identity())).unwrap(),
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
        BlobTerminalDisposition::AlreadyAbandoned
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

fn assert_not_expired(
    serving: &ServingPhysicalRuntime,
    token: worth_store::physical_runtime::BlobResumeToken,
    scope: &AdmittedBlobScope,
    selected: u64,
    maximum: u64,
) {
    let before = selected_blob_records(serving);
    let media_before = serving.media_counters();
    assert!(matches!(
        serving
            .blobs()
            .unwrap()
            .expire_ingest(token, scope, placement(), deadline(), limits()),
        Err(BlobTerminalFailure::NotExpired {
            selected_checkpoint_sequence,
            maximum_checkpoint_sequence,
        }) if selected_checkpoint_sequence == selected && maximum_checkpoint_sequence == maximum
    ));
    assert_eq!(selected_blob_records(serving), before);
    let media_after = serving.media_counters();
    assert_eq!(
        media_after.append_attempts(),
        media_before.append_attempts()
    );
    assert_eq!(
        media_after.positioned_write_attempts(),
        media_before.positioned_write_attempts()
    );
}

pub(super) fn unfinished<'a>(
    serving: &'a ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
) -> worth_store::physical_runtime::BlobIngestSession<'a> {
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (CHUNK * 2) as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(1).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, limits)
        .unwrap();
    ingest.push(&[0x5a; CHUNK]).unwrap();
    ingest
}

pub(super) fn deadline() -> PhysicalMutationDeadline {
    PhysicalMutationDeadline::after_milliseconds(120_000).unwrap()
}

pub(super) fn limits() -> BlobTerminalLimits {
    BlobTerminalLimits::new(NonZeroU64::new(128).unwrap())
}

pub(super) fn completed_checkpoint(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> CompletedPhysicalCheckpoint {
    completed(start_checkpoint(serving, key))
}

fn start_checkpoint(serving: &ServingPhysicalRuntime, key: u8) -> PhysicalCheckpointHandle {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::at(
            TemporalDuration::temporal_duration(10_000).expect("positive deadline"),
        ),
    );
    match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        TransitionOutcome::Denied(cause) => panic!("checkpoint denied: {cause:?}"),
        TransitionOutcome::Deferred(cause) => panic!("checkpoint deferred: {cause:?}"),
        TransitionOutcome::Stale(cause) => panic!("checkpoint stale: {cause:?}"),
        TransitionOutcome::Failed(cause) => panic!("checkpoint start failed: {cause:?}"),
    }
}

fn completed(handle: PhysicalCheckpointHandle) -> CompletedPhysicalCheckpoint {
    match handle.wait() {
        PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("checkpoint must complete before expiry assertion: {other:?}"),
    }
}

fn append_padding(serving: &ServingPhysicalRuntime, ordinal: u8) {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
            [0xe0 + ordinal; 32],
        ))
        .unwrap();
    let frame = [ordinal; CHUNK];
    let batch = RecordAppendBatch::try_from_iter([frame.as_slice()]).unwrap();
    let TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) =
        submission
            .prepare_durable_append(
                batch,
                placement(),
                PhysicalMutationRequest::platform_durable(key, deadline()),
            )
            .into_raw()
    else {
        panic!("WAL-rotation padding must prepare through ordinary C5");
    };
    assert!(matches!(
        prepared.execute(),
        PhysicalMutationOutcome::Completed(_)
    ));
}

fn selected_blob_frames(
    serving: &ServingPhysicalRuntime,
) -> Vec<(worth_store::physical_runtime::PhysicalRecordId, Vec<u8>)> {
    selected_blob_records(serving)
        .into_iter()
        .filter(|(_, bytes)| bytes.starts_with(b"WRC11BLB"))
        .collect()
}

fn assert_checkpoint_wal_range(
    root: &Path,
    cutoff: u64,
    stage: &str,
) -> wal_oracle::IndependentWalInventory {
    let inventory = wal_oracle::inspect_wal_inventory(root)
        .unwrap_or_else(|denial| panic!("{stage}: retained WAL inventory denied: {denial:?}"));
    let (first, active_end) = inventory
        .lsn_range()
        .unwrap_or_else(|| panic!("{stage}: retained WAL inventory is empty"));
    assert!(
        first <= cutoff && cutoff <= active_end,
        "{stage}: checkpoint cutoff {cutoff} outside retained WAL [{first}, {active_end}]"
    );
    inventory
}

fn wal_segment_path(root: &Path, (segment, generation): (u64, u64)) -> std::path::PathBuf {
    root.join("families")
        .join("wal")
        .join(format!("segment-{segment}-generation-{generation}.wal"))
}
