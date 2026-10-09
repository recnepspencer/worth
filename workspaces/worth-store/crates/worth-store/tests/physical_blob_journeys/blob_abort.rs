use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobCheckpointLimit, BlobIngestClaimDenial, BlobIngestDeclaration,
    BlobIngestSession, BlobReadLimits, BlobResumeFailure, BlobResumeLimits, BlobResumeToken,
    BlobTerminalDisposition, BlobTerminalFailure, BlobTerminalLimits, PhysicalMutationDeadline,
    RecordByteLimit, RecordReadLimits, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_physical_format::{decode_blob_record, BlobRecordV1};

use super::{
    blob_crash::{establish_recovery_frontier, recover_closed_store},
    blob_frontier::selected_blob_records,
    blob_ingest_process::observe_closed_store_named,
    fixture::{
        admitted_blob_scope, admitted_blob_scope_for_replay_boundary, placement,
        serving_from_initialization, serving_from_open,
    },
};

const CHUNK: usize = 64 * 1024;

#[test]
fn abort_is_durable_terminal_but_preserves_selected_bytes_and_read_pins() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope("c11.blob.abort.custody.scope");
    let ingest = unfinished(&serving, &scope);
    let token = ingest.resume_token();
    let session = ingest.session_id();
    drop(ingest);
    let before = selected_blob_records(&serving);
    let held = serving.records().unwrap();
    let protected_root = held.protected_root().root().generation();
    let blobs = serving.blobs().unwrap();
    let receipt = blobs
        .abort_ingest(token, &scope, placement(), deadline(), limits())
        .unwrap();
    assert_eq!(receipt.session(), session);
    assert_eq!(
        receipt.disposition(),
        BlobTerminalDisposition::NewlyAbandoned
    );
    assert_eq!(held.protected_root().root().generation(), protected_root);
    let (chunk_record, expected_chunk) = before
        .iter()
        .find(|(_, bytes)| matches!(decode_blob_record(bytes), Ok(BlobRecordV1::Chunk(_))))
        .expect("unfinished ingest selected its first chunk");
    let mut retained = held
        .open(
            *chunk_record,
            RecordReadLimits::new(
                RecordByteLimit::new(u32::try_from(expected_chunk.len()).unwrap()).unwrap(),
            ),
        )
        .unwrap();
    let mut observed_chunk = Vec::new();
    let mut scratch = [0_u8; 8192];
    loop {
        let read = retained.read_next(&mut scratch).unwrap();
        if read == 0 {
            break;
        }
        observed_chunk.extend_from_slice(&scratch[..read]);
    }
    assert_eq!(&observed_chunk, expected_chunk);
    drop(retained);
    let after = selected_blob_records(&serving);
    assert_eq!(after.len(), before.len() + 1);
    for retained in &before {
        assert!(
            after.contains(retained),
            "abort may not unroute or rewrite custody"
        );
    }
    let terminal = after
        .iter()
        .filter_map(|(_, bytes)| match decode_blob_record(bytes) {
            Ok(BlobRecordV1::SessionAbandoned(value)) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(terminal.len(), 1);
    assert_eq!(terminal[0].session(), session.bytes());
    let root_after = root_generation(&serving);
    let repeated = blobs
        .abort_ingest(token, &scope, placement(), deadline(), limits())
        .unwrap();
    assert_eq!(repeated.record(), receipt.record());
    assert_eq!(
        repeated.disposition(),
        BlobTerminalDisposition::AlreadyAbandoned
    );
    assert_eq!(root_generation(&serving), root_after);
    assert_resume_abandoned(&serving, token, &scope);
    drop(held);
    drop(blobs);
    serving.close();

    recover_closed_store(directory.path());
    let reopened = serving_from_open(directory.path());
    assert_resume_abandoned(&reopened, token, &scope);
    assert_eq!(selected_blob_records(&reopened), after);
    reopened.close();
    let report = observe_closed_store_named(directory.path(), "c11-blob-abort", "explicit-abort");
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    assert_eq!(
        artifacts
            .iter()
            .filter(|row| row["family"] == "blob_resume_session")
            .count(),
        2
    );
    assert_eq!(
        artifacts
            .iter()
            .filter(|row| row["family"] == "blob_chunk_frame")
            .count(),
        1
    );
    assert!(!artifacts
        .iter()
        .any(|row| row["family"] == "blob_generation_publication"));
    for row in artifacts.iter().filter(|row| {
        row["family"]
            .as_str()
            .is_some_and(|s| s.starts_with("blob_"))
    }) {
        assert_eq!(row["outcome"]["posture"], "intact", "{row}");
    }
}

#[test]
fn abort_denials_preserve_custody_and_release_inspecting_claim() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.abort.denial.scope");
    let ingest = unfinished(&serving, &scope);
    let token = ingest.resume_token();
    let blobs = serving.blobs().unwrap();
    let before = selected_blob_records(&serving);
    let media_before = serving.media_counters();
    assert!(matches!(
        blobs.abort_ingest(token, &scope, placement(), deadline(), limits()),
        Err(BlobTerminalFailure::Claim(
            BlobIngestClaimDenial::CompetingSession
        ))
    ));
    drop(ingest);
    let mut forged = token.encode();
    forged[64] ^= 1;
    assert!(matches!(
        blobs.abort_ingest(
            BlobResumeToken::decode(&forged).unwrap(),
            &scope,
            placement(),
            deadline(),
            limits()
        ),
        Err(BlobTerminalFailure::DeclarationMismatch)
    ));
    let foreign_scope = admitted_blob_scope_for_replay_boundary("c11.blob.abort.foreign.scope");
    assert!(matches!(
        blobs.abort_ingest(token, &foreign_scope, placement(), deadline(), limits()),
        Err(BlobTerminalFailure::ScopeMismatch)
    ));
    assert!(matches!(
        blobs.abort_ingest(
            token,
            &scope,
            placement(),
            deadline(),
            BlobTerminalLimits::new(NonZeroU64::new(1).unwrap())
        ),
        Err(BlobTerminalFailure::ScanBoundExhausted)
    ));
    assert_eq!(selected_blob_records(&serving), before);
    let media_after = serving.media_counters();
    assert_eq!(
        media_after.append_attempts(),
        media_before.append_attempts()
    );
    assert_eq!(
        media_after.positioned_write_attempts(),
        media_before.positioned_write_attempts()
    );
    let mut resumed = blobs
        .resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            deadline(),
            resume_limits(),
        )
        .expect("denied abort releases inspection claim");
    assert_eq!(resumed.frontier().bytes(), CHUNK as u64);
    resumed.push(&[0xa5; CHUNK]).unwrap();
    resumed.finish().unwrap();
    let published_root = root_generation(&serving);
    let media_before = serving.media_counters();
    assert!(matches!(
        blobs.abort_ingest(token, &scope, placement(), deadline(), limits()),
        Err(BlobTerminalFailure::AlreadyPublished)
    ));
    assert_eq!(root_generation(&serving), published_root);
    let media_after = serving.media_counters();
    assert_eq!(
        media_after.append_attempts(),
        media_before.append_attempts()
    );
    assert_eq!(
        media_after.positioned_write_attempts(),
        media_before.positioned_write_attempts()
    );
    drop(blobs);
    serving.close();
}

pub(super) fn unfinished<'a>(
    serving: &'a ServingPhysicalRuntime,
    scope: &AdmittedBlobScope,
) -> BlobIngestSession<'a> {
    let blobs = serving.blobs().unwrap();
    let read_limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(read_limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        (CHUNK * 2) as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(16).unwrap(),
        deadline(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), CHUNK as u64, read_limits)
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

fn resume_limits() -> BlobResumeLimits {
    BlobResumeLimits::new(
        NonZeroU64::new(128).unwrap(),
        NonZeroU64::new(1 << 20).unwrap(),
    )
}

pub(super) fn assert_resume_abandoned(
    serving: &ServingPhysicalRuntime,
    token: BlobResumeToken,
    scope: &AdmittedBlobScope,
) {
    assert!(matches!(
        serving.blobs().unwrap().resume_ingest(
            token,
            scope,
            placement(),
            CHUNK as u64,
            deadline(),
            resume_limits()
        ),
        Err(BlobResumeFailure::AlreadyAbandoned)
    ));
}

fn root_generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .records()
        .unwrap()
        .protected_root()
        .root()
        .generation()
        .get()
}
