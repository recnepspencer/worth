use std::{
    fs,
    num::NonZeroU64,
    path::Path,
    sync::{mpsc, Arc, Barrier},
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobIngestClaimDenial, BlobReadLimits,
    BlobReadOpenFailure, BlobResumeFailure, BlobResumeLimits, BlobResumeToken,
    BlobTerminalDisposition, BlobTerminalFailure, ServingPhysicalRuntime,
};
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, PersistedRecordIdentity};

use super::{
    blob_abort::{assert_resume_abandoned, deadline, limits, unfinished},
    blob_crash::{
        establish_recovery_frontier, kill_at, marker_path, recover_closed_store, resume_token_path,
        write_marker, SCOPE_KEY,
    },
    blob_frontier::selected_blob_records,
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

const CHUNK: usize = 64 * 1024;

#[test]
fn abort_wal_durable_before_root_replacement_replays_one_terminal() {
    assert_killed_terminal("crash-abort-wal");
}

#[test]
fn abort_root_replaced_before_completion_reopens_one_terminal() {
    assert_killed_terminal("crash-abort-root");
}

fn assert_killed_terminal(role: &'static str) {
    let world = kill_at(role, Duration::from_secs(180));
    let token = BlobResumeToken::decode(&fs::read(resume_token_path(&world.root)).unwrap())
        .expect("child wrote one syntactically valid token before abort");
    assert_eq!(&token.encode()[24..40], world.session.as_slice());
    recover_closed_store(&world.root);

    let serving = serving_from_open(&world.root);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let blobs = serving.blobs().unwrap();
    assert!(matches!(
        blobs.resolve_publication(
            world.object,
            1,
            &scope,
            BlobReadLimits::new(NonZeroU64::new(128).unwrap())
        ),
        Err(BlobReadOpenFailure::PublicationNotFound)
    ));
    let selected = selected_blob_records(&serving);
    let mut declaration = None;
    let mut chunk = None;
    let mut terminal = None;
    for (record, bytes) in selected {
        if !bytes.starts_with(b"WRC11BLB") {
            continue;
        }
        match decode_blob_record(&bytes).expect("selected blob control/frame must decode") {
            BlobRecordV1::SessionDeclared(value) if value.session() == world.session => {
                assert!(declaration.replace(record).is_none());
                assert_eq!(value.object(), world.object);
            }
            BlobRecordV1::Chunk(value) if value.occurrence().session() == world.session => {
                assert!(chunk.replace(record).is_none());
                assert_eq!(value.occurrence().ordinal(), 0);
                assert_eq!(value.bytes(), &[0x5a; CHUNK]);
            }
            BlobRecordV1::SessionAbandoned(value) if value.session() == world.session => {
                assert!(terminal.replace((record, value)).is_none());
            }
            BlobRecordV1::GenerationPublished(value) if value.session() == world.session => {
                panic!("aborted session cannot publish a generation")
            }
            _ => {}
        }
    }
    let declaration = declaration.expect("selected declaration must survive C8");
    chunk.expect("claimed chunk remains selected, not reclaimed by abort");
    let (terminal_record, terminal) = terminal.expect("C8 selects exactly one abort terminal");
    assert_eq!(terminal.declaration_record(), persisted(declaration));
    let expected_digest: [u8; 32] = token.encode()[64..96].try_into().unwrap();
    assert_eq!(terminal.declaration_digest(), expected_digest);
    assert_resume_abandoned(&serving, token, &scope);
    let repeated = blobs
        .abort_ingest(token, &scope, placement(), deadline(), limits())
        .unwrap();
    assert_eq!(repeated.record(), persisted(terminal_record));
    assert_eq!(
        repeated.disposition(),
        BlobTerminalDisposition::AlreadyAbandoned
    );
    drop(blobs);
    serving.close();

    let report = observe_closed_store_named(&world.root, "c11-blob-abort-crash", role);
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    let count = |family: &str| {
        artifacts
            .iter()
            .filter(|row| row["family"] == family)
            .count()
    };
    assert_eq!(count("blob_resume_session"), 2, "{report}");
    assert_eq!(count("blob_chunk_frame"), 1, "{report}");
    assert_eq!(count("blob_generation_publication"), 0, "{report}");
    for row in artifacts.iter().filter(|row| {
        row["family"]
            .as_str()
            .is_some_and(|family| family.starts_with("blob_"))
    }) {
        assert_eq!(row["outcome"]["posture"], "intact", "{row}");
    }
}

fn persisted(record: worth_store::physical_runtime::PhysicalRecordId) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal()).unwrap()
}

pub(super) fn child(root: &Path, role: &str) {
    let checkpoint = match role {
        "crash-abort-wal" => PhysicalMutationCheckpoint::AfterWalDurability,
        "crash-abort-root" => PhysicalMutationCheckpoint::AfterRootReplacement,
        _ => panic!("unknown abort crash role: {role}"),
    };
    let serving = serving_from_initialization(root);
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let ingest = unfinished(&serving, &scope);
    let token = ingest.resume_token();
    let session = ingest.session_id().bytes();
    let object = selected_object(&serving, session);
    let token_path = resume_token_path(root);
    let pending = token_path.with_extension("pending");
    fs::write(&pending, token.encode()).unwrap();
    fs::rename(pending, token_path).unwrap();
    drop(ingest);

    let gate = serving.pause_physical_mutation_at(checkpoint);
    let marker = marker_path(root);
    thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(150);
            while Instant::now() < deadline {
                if gate.await_arrival() {
                    write_marker(&marker, object, session);
                    return;
                }
            }
            panic!("explicit abort never reached {checkpoint:?}");
        });
        let _ =
            serving
                .blobs()
                .unwrap()
                .abort_ingest(token, &scope, placement(), deadline(), limits());
        panic!("abort escaped {checkpoint:?} before process kill");
    });
}

fn selected_object(serving: &ServingPhysicalRuntime, session: [u8; 16]) -> [u8; 16] {
    selected_blob_records(serving)
        .into_iter()
        .find_map(|(_, bytes)| match decode_blob_record(&bytes) {
            Ok(BlobRecordV1::SessionDeclared(value)) if value.session() == session => {
                Some(value.object())
            }
            _ => None,
        })
        .expect("child selected declaration binds its object")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RaceResult {
    AbortWon,
    AbortDenied,
    ResumeHeld,
    ResumeDenied,
}

#[test]
fn racing_abort_and_resume_cannot_both_obtain_effect_authority() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.abort.resume.race.scope");
    let ingest = unfinished(&serving, &scope);
    let token = ingest.resume_token();
    drop(ingest);
    let start = Arc::new(Barrier::new(2));
    let (results_tx, results_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    thread::scope(|workers| {
        let barrier = Arc::clone(&start);
        let tx = results_tx.clone();
        let serving_ref = &serving;
        let scope_ref = &scope;
        workers.spawn(move || {
            barrier.wait();
            let result = serving_ref.blobs().unwrap().abort_ingest(
                token,
                scope_ref,
                placement(),
                deadline(),
                limits(),
            );
            let observed = match result {
                Ok(receipt) if receipt.disposition() == BlobTerminalDisposition::NewlyAbandoned => {
                    RaceResult::AbortWon
                }
                Err(BlobTerminalFailure::Claim(BlobIngestClaimDenial::CompetingSession)) => {
                    RaceResult::AbortDenied
                }
                other => panic!("unexpected racing abort result: {other:?}"),
            };
            tx.send(observed).unwrap();
        });
        let barrier = Arc::clone(&start);
        let serving_ref = &serving;
        let scope_ref = &scope;
        workers.spawn(move || {
            barrier.wait();
            let result = serving_ref.blobs().unwrap().resume_ingest(
                token,
                scope_ref,
                placement(),
                CHUNK as u64,
                deadline(),
                BlobResumeLimits::new(
                    NonZeroU64::new(128).unwrap(),
                    NonZeroU64::new(1 << 20).unwrap(),
                ),
            );
            match result {
                Ok(session) => {
                    results_tx.send(RaceResult::ResumeHeld).unwrap();
                    release_rx.recv().unwrap();
                    drop(session);
                }
                Err(BlobResumeFailure::AlreadyAbandoned)
                | Err(BlobResumeFailure::Claim(BlobIngestClaimDenial::CompetingSession)) => {
                    results_tx.send(RaceResult::ResumeDenied).unwrap();
                }
                Err(other) => panic!("unexpected racing resume denial: {other:?}"),
            }
        });
        let results = [results_rx.recv().unwrap(), results_rx.recv().unwrap()];
        let _ = release_tx.send(());
        assert!(
            (results.contains(&RaceResult::AbortWon)
                && results.contains(&RaceResult::ResumeDenied))
                || (results.contains(&RaceResult::AbortDenied)
                    && results.contains(&RaceResult::ResumeHeld)),
            "one owner must win the same session: {results:?}"
        );
    });
    serving.close();
}
