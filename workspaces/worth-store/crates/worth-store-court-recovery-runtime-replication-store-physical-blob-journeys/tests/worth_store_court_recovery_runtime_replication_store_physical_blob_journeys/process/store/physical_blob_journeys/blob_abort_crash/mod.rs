#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_abort_crash::*;

use {
    crate::physical_blob_journeys::blob_abort::deadline,
    crate::physical_blob_journeys::blob_abort::limits,
    crate::physical_blob_journeys::blob_abort::unfinished,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
};

#[test]
fn abort_wal_durable_before_root_replacement_replays_one_terminal() {
    assert_killed_terminal("crash-abort-wal");
}

#[test]
fn abort_root_replaced_before_completion_reopens_one_terminal() {
    assert_killed_terminal("crash-abort-root");
}

#[test]
fn racing_abort_and_resume_cannot_both_obtain_effect_authority() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let scope = admitted_blob_scope("c11.blob.abort.resume.race.scope");
    let ingest = unfinished(&serving, &scope);
    let token = ingest.resume_token();
    drop(ingest);
    let start = std::sync::Arc::new(std::sync::Barrier::new(2));
    let (results_tx, results_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    std::thread::scope(|workers| {
        let barrier = std::sync::Arc::clone(&start);
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
                Ok(receipt) if receipt.disposition() == worth_store::physical_runtime::BlobTerminalDisposition::NewlyAbandoned => {
                    crate::physical_blob_journeys::blob_abort_crash::RaceResult::AbortWon
                }
                Err(worth_store::physical_runtime::BlobTerminalFailure::Claim(worth_store::physical_runtime::BlobIngestClaimDenial::CompetingSession)) => {
                    crate::physical_blob_journeys::blob_abort_crash::RaceResult::AbortDenied
                }
                other => panic!("unexpected racing abort result: {other:?}"),
            };
            tx.send(observed).unwrap();
        });
        let barrier = std::sync::Arc::clone(&start);
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
                worth_store::physical_runtime::BlobResumeLimits::new(
                    std::num::NonZeroU64::new(128).unwrap(),
                    std::num::NonZeroU64::new(1 << 20).unwrap(),
                ),
            );
            match result {
                Ok(session) => {
                    results_tx.send(crate::physical_blob_journeys::blob_abort_crash::RaceResult::ResumeHeld).unwrap();
                    release_rx.recv().unwrap();
                    drop(session);
                }
                Err(worth_store::physical_runtime::BlobResumeFailure::AlreadyAbandoned)
                | Err(worth_store::physical_runtime::BlobResumeFailure::Claim(worth_store::physical_runtime::BlobIngestClaimDenial::CompetingSession)) => {
                    results_tx.send(crate::physical_blob_journeys::blob_abort_crash::RaceResult::ResumeDenied).unwrap();
                }
                Err(other) => panic!("unexpected racing resume denial: {other:?}"),
            }
        });
        let results = [results_rx.recv().unwrap(), results_rx.recv().unwrap()];
        let _ = release_tx.send(());
        assert!(
            (results
                .contains(&crate::physical_blob_journeys::blob_abort_crash::RaceResult::AbortWon)
                && results.contains(
                    &crate::physical_blob_journeys::blob_abort_crash::RaceResult::ResumeDenied
                ))
                || (results.contains(
                    &crate::physical_blob_journeys::blob_abort_crash::RaceResult::AbortDenied
                ) && results.contains(
                    &crate::physical_blob_journeys::blob_abort_crash::RaceResult::ResumeHeld
                )),
            "one owner must win the same session: {results:?}"
        );
    });
    serving.close();
}
