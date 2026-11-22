//! Full Phase 6 Host oracle under D14 derived-cache discard (graceful close & OS kill,
//! same & changed builds): compares authored truth, history, World occurrences, nontrivial
//! durable custody (settled release across graceful close; live unresolved across OS kill),
//! and exact idempotency receipts via DurableIdentityCorrespondence against MemoryOnly.
use super::super::{
    opening,
    protocol::{Disposition, Request},
    session::Session,
    *,
};
use host_observation::{comparison, projection, receipts};
use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

const FIRST_BUILD: &str = "11111111-1111-4111-8111-111111111111";
const SECOND_BUILD: &str = "22222222-2222-4222-8222-222222222222";

#[test]
fn host_oracle_discard_derived_graceful_close_preserves_truth_history_custody_and_receipts() {
    for reopen_build in [FIRST_BUILD, SECOND_BUILD] {
        run_discard_oracle(false, reopen_build);
    }
}

#[test]
fn host_oracle_discard_derived_process_kill_preserves_truth_history_custody_and_receipts() {
    for reopen_build in [FIRST_BUILD, SECOND_BUILD] {
        run_discard_oracle(true, reopen_build);
    }
}

fn cadence_map(disposition: Option<Disposition>) -> BTreeMap<String, (u64, u64)> {
    match disposition {
        Some(Disposition::Cadence { journals }) => journals
            .into_iter()
            .map(|(name, covered, head)| (name, (covered, head)))
            .collect(),
        other => panic!("expected Cadence disposition, got {other:?}"),
    }
}

fn run_discard_oracle(kill: bool, reopen_build: &str) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("home");
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let reference = host_observation::opening::open(
        worth_query_host::facade::WorthQueryApplicationHomeLocation::MemoryOnly,
        worth_foundational::facade::ApplicationBuildIdentity::parse(FIRST_BUILD).unwrap(),
        initialized.clone(),
        contacts.clone(),
        None,
        Default::default(),
        &execution,
    );
    let mut correspondence = comparison::HostOracleComparison::default();
    let mut reference_originals = std::collections::HashMap::new();

    let (mut session, opened) =
        Session::start_with_build(&path, &root.path().join("first"), 0, Some(FIRST_BUILD));
    assert_eq!(opened.initialized, 1);
    let mut reference_state = projection::observe(reference.runtime(), &execution);
    comparison::compare(
        &mut correspondence,
        opened.state.as_ref().unwrap(),
        &reference_state,
    )
    .unwrap();

    // 1. Committed writes (keys 1 and 2) + IntentDrift on key 1.
    for (key, value, expected_fate, handler_calls) in [
        (1_u64, 11_u64, receipts::MutationFate::Committed, 1_usize),
        (2_u64, 22_u64, receipts::MutationFate::Committed, 1_usize),
        (1_u64, 99_u64, receipts::MutationFate::IntentDrift, 0_usize),
    ] {
        let before_ref_contacts = contacts.load(Ordering::SeqCst);
        let expected = super::super::execution::mutation(
            &reference,
            &HostCommand::Mutate {
                branch: BranchLabel::Main,
                key,
                value,
            },
            &mut reference_originals,
        )
        .unwrap();
        let actual = session.send(Request::Mutate { key, value });
        let actual_mutation = actual.mutation.as_ref().unwrap();
        assert_eq!(actual_mutation.fate, expected_fate);
        assert_eq!(expected.fate, expected_fate);
        assert_eq!(
            contacts.load(Ordering::SeqCst) - before_ref_contacts,
            handler_calls
        );
        reference_state = projection::observe(reference.runtime(), &execution);
        let actual_state = actual.state.as_ref().unwrap();
        receipts::compare(
            correspondence.identities_mut(),
            actual_mutation,
            &expected,
            actual_state,
            &reference_state,
        )
        .unwrap();
        comparison::compare(&mut correspondence, actual_state, &reference_state).unwrap();
    }

    // 2. Seed nontrivial durable World/Query unpublished custody (key 3, value 33).
    let abandoned = session.send(Request::AbandonOriginal { key: 3, value: 33 });
    assert!(matches!(
        abandoned.mutation.as_ref().unwrap().fate,
        receipts::MutationFate::ProductPreparationPending
            | receipts::MutationFate::ProductUnpublished
    ));
    let abandoned_state = abandoned.state.unwrap();
    assert_eq!(abandoned_state.custody.len(), 1);
    let original_custody = abandoned_state.custody[0].clone();
    assert!(original_custody.original_effect.is_some());

    // Graceful close requires business custody to be settled first; kill + reopen to transition
    // the unpublished effect to recovered custody, settle via ReleaseOriginal (recording durable
    // release disposition in world.custody), then checkpoint, discard, and gracefully close.
    // Under OS kill, keep the live unresolved custody pending across discard + kill.
    let pre_discard_released = if !kill {
        session.kill();
        let (reopened_session, recovered_before_discard) =
            Session::start_with_build(&path, &root.path().join("settle"), 0, Some(FIRST_BUILD));
        assert_eq!(recovered_before_discard.initialized, 0);
        assert_eq!(recovered_before_discard.contacts, 0);
        session = reopened_session;
        let released = session.send(Request::ReleaseOriginal {
            key: 3,
            value: 33,
            expect_continued: false,
        });
        assert!(matches!(released.disposition, Some(Disposition::Released(1))));
        let observed = session.send(Request::ObserveReleased { key: 3, value: 33 });
        let Some(Disposition::ReleasedOriginal(projection)) = observed.disposition else {
            panic!("expected ReleasedOriginal projection before discard");
        };
        assert!(projection.release_position.is_some());
        Some(projection)
    } else {
        None
    };

    // 3. Explicit checkpoint, then discard derived caches via D14 door.
    let discarded = session.send(Request::CheckpointAndDiscard);
    let discard_generation = match discarded.disposition {
        Some(Disposition::Discarded {
            checkpoint_generation,
            discard_generation,
        }) => {
            assert!(checkpoint_generation >= 1);
            assert_eq!(discard_generation, checkpoint_generation + 1);
            discard_generation
        }
        other => panic!("expected Discarded disposition, got {other:?}"),
    };

    if kill {
        session.kill();
    } else {
        let closed = session.send(Request::CloseExpectingDiscarded);
        assert!(closed.closed);
        session.finish();
    }

    // 4. Fresh-process reopen under same or changed build.
    let (mut session, recovered) = Session::start_with_build(
        &path,
        &root.path().join("second"),
        0,
        Some(reopen_build),
    );
    assert_eq!(recovered.initialized, 0, "reopen never initializes again");
    assert_eq!(recovered.contacts, 0, "reopen never invokes a handler");
    let cadence = cadence_map(session.send(Request::InspectCadence).disposition);
    assert_eq!(cadence.len(), 5);
    assert!(
        cadence.values().all(|&(covered, _)| covered == 0),
        "all 5 reported cadence journals must have zero coverage after discard: {cadence:?}"
    );
    assert!(cadence["relational.canonical-commits"].1 > 0);
    assert!(cadence["world.occurrences"].1 > 0);
    assert!(cadence["world.custody"].1 > 0);

    let recovered_state = recovered.state.as_ref().unwrap();
    if kill {
        // Exact live unresolved custody survives discard + OS kill before continuation.
        assert_eq!(recovered_state.custody.len(), 1);
        assert_eq!(recovered_state.custody[0].owner, original_custody.owner);
        assert_eq!(recovered_state.custody[0].identity, original_custody.identity);
        assert_eq!(
            recovered_state.custody[0].original_effect,
            original_custody.original_effect
        );
        assert_eq!(recovered_state.head, abandoned_state.head);
    } else {
        // Exact settled released custody record survives discard + graceful close.
        assert!(recovered_state.custody.is_empty());
        let observed = session.send(Request::ObserveReleased { key: 3, value: 33 });
        assert_eq!(
            observed.disposition,
            Some(Disposition::ReleasedOriginal(pre_discard_released.unwrap()))
        );
        comparison::compare(&mut correspondence, recovered_state, &reference_state).unwrap();
    }

    // 5. Replay committed keys (1 and 2) -> exact original receipts via DurableIdentityCorrespondence
    // and ZERO handler contacts.
    for (key, value) in [(1_u64, 11_u64), (2_u64, 22_u64)] {
        let expected = super::super::execution::mutation(
            &reference,
            &HostCommand::Mutate {
                branch: BranchLabel::Main,
                key,
                value,
            },
            &mut reference_originals,
        )
        .unwrap();
        let replayed = session.send(Request::Mutate { key, value });
        assert_eq!(replayed.contacts, 0, "replaying committed key invokes no handler");
        let actual_mutation = replayed.mutation.as_ref().unwrap();
        assert_eq!(actual_mutation.fate, receipts::MutationFate::Committed);
        reference_state = projection::observe(reference.runtime(), &execution);
        receipts::compare(
            correspondence.identities_mut(),
            actual_mutation,
            &expected,
            replayed.state.as_ref().unwrap(),
            &reference_state,
        )
        .unwrap();
    }

    // 6. Under OS kill, continue the live recovered custody (key 3, value 33) -> completes once,
    // matching MemoryOnly reference across full truth, history, occurrences, and empty custody.
    if kill {
        let expected_continued = super::super::execution::mutation(
            &reference,
            &HostCommand::Mutate {
                branch: BranchLabel::Main,
                key: 3,
                value: 33,
            },
            &mut reference_originals,
        )
        .unwrap();
        let continued = session.send(Request::ContinueOriginal {
            key: 3,
            value: 33,
            pause_after_disposition: false,
        });
        assert_eq!(
            continued.contacts, 0,
            "continuing recovered custody never re-runs handler"
        );
        let continued_mutation = match continued.disposition {
            Some(Disposition::Continued(mutation)) => mutation,
            other => panic!("expected Continued disposition, got {other:?}"),
        };
        reference_state = projection::observe(reference.runtime(), &execution);
        let continued_state = continued.state.as_ref().unwrap();
        assert!(continued_state.custody.is_empty());
        receipts::compare(
            correspondence.identities_mut(),
            &continued_mutation,
            &expected_continued,
            continued_state,
            &reference_state,
        )
        .unwrap();
        comparison::compare(&mut correspondence, continued_state, &reference_state).unwrap();
    }

    // 7. Graceful close succeeds with a fresh checkpoint at discard_generation + 1.
    let closed = session.send(Request::Close);
    assert!(closed.closed);
    assert_eq!(
        closed.disposition,
        Some(Disposition::ClosedWithCheckpoint {
            generation: Some(discard_generation + 1),
        })
    );
    session.finish();

    let refused = reference
        .close(&execution)
        .err()
        .expect("MemoryOnly preserves lifecycle refusal");
    drop(refused.resume().unwrap());
}
