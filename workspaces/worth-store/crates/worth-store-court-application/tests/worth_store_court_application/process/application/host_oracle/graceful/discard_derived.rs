//! Full Phase 6 Host oracle under D14 derived-cache discard (graceful close & OS kill,
//! same & changed builds): compares authored truth, history, World occurrences, nontrivial
//! durable custody (settled release across graceful close; live unresolved across OS kill),
//! post-reopen IntentDrift, once-only continuation, and exact receipts via
//! DurableIdentityCorrespondence against MemoryOnly.
use super::super::{
    opening,
    protocol::{Disposition, Request},
    session::Session,
    *,
};
use host_observation::{
    comparison,
    custody::{CustodyAction, CustodyCause},
    projection, receipts,
};
use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

const FIRST_BUILD: &str = "11111111-1111-4111-8111-111111111111";
const SECOND_BUILD: &str = "22222222-2222-4222-8222-222222222222";
const EXPECTED_CADENCE_KEYS: [&str; 5] = [
    "relational.branch-transitions",
    "relational.canonical-commits",
    "world.branches",
    "world.custody",
    "world.occurrences",
];

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

fn open_memory_ref(
    initialized: Arc<AtomicUsize>,
    contacts: Arc<AtomicUsize>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> host_observation::OracleHome {
    host_observation::opening::open(
        worth_query_host::facade::WorthQueryApplicationHomeLocation::MemoryOnly,
        worth_foundational::facade::ApplicationBuildIdentity::parse(FIRST_BUILD).unwrap(),
        initialized,
        contacts,
        None,
        Default::default(),
        execution,
    )
}

fn run_discard_oracle(kill: bool, reopen_build: &str) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("home");
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let reference = open_memory_ref(initialized.clone(), contacts.clone(), &execution);
    let mut correspondence = comparison::HostOracleComparison::default();
    let mut reference_originals = std::collections::HashMap::new();
    let continued_reference = kill.then(|| {
        open_memory_ref(
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
            &execution,
        )
    });
    let mut continued_originals = std::collections::HashMap::new();
    let mut continued_correspondence = comparison::HostOracleComparison::default();

    let (mut session, opened) =
        Session::start_with_build(&path, &root.path().join("first"), 0, Some(FIRST_BUILD));
    assert_eq!(opened.initialized, 1);
    let mut reference_state = projection::observe(reference.runtime(), &execution);
    let open_state = opened.state.as_ref().unwrap();
    comparison::compare(&mut correspondence, open_state, &reference_state).unwrap();
    if let Some(ref cont_ref) = continued_reference {
        let cont_open = projection::observe(cont_ref.runtime(), &execution);
        comparison::compare(&mut continued_correspondence, open_state, &cont_open).unwrap();
    }

    // 1. Committed writes (keys 1 and 2) + pre-discard IntentDrift on key 1.
    for (key, value, expected_fate, handler_calls) in [
        (1_u64, 11_u64, receipts::MutationFate::Committed, 1_usize),
        (2_u64, 22_u64, receipts::MutationFate::Committed, 1_usize),
        (1_u64, 99_u64, receipts::MutationFate::IntentDrift, 0_usize),
    ] {
        let before_ref_contacts = contacts.load(Ordering::SeqCst);
        let cmd = HostCommand::Mutate { branch: BranchLabel::Main, key, value };
        let expected =
            super::super::execution::mutation(&reference, &cmd, &mut reference_originals).unwrap();
        if let Some(ref cont_ref) = continued_reference {
            let _ =
                super::super::execution::mutation(cont_ref, &cmd, &mut continued_originals).unwrap();
        }
        let actual = session.send(Request::Mutate { key, value });
        let actual_mutation = actual.mutation.as_ref().unwrap();
        assert_eq!(actual_mutation.fate, expected_fate);
        assert_eq!(expected.fate, expected_fate);
        assert_eq!(contacts.load(Ordering::SeqCst) - before_ref_contacts, handler_calls);
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
        if let Some(ref cont_ref) = continued_reference {
            let cont_state = projection::observe(cont_ref.runtime(), &execution);
            comparison::compare(&mut continued_correspondence, actual_state, &cont_state).unwrap();
        }
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
    // Under OS kill, mirror AbandonCaller on the MemoryOnly reference so we can compare full
    // authored truth, history, occurrences, idempotency, and custody immediately at kill reopen.
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
        let _ = super::super::execution::mutation(
            &reference,
            &HostCommand::AbandonCaller { key: 3, value: 33 },
            &mut reference_originals,
        )
        .unwrap();
        reference_state = projection::observe(reference.runtime(), &execution);
        comparison::compare(&mut correspondence, &abandoned_state, &reference_state).unwrap();
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
    let cadence_keys: Vec<&str> = cadence.keys().map(String::as_str).collect();
    assert_eq!(cadence_keys, EXPECTED_CADENCE_KEYS);
    assert!(
        cadence.values().all(|&(covered, _)| covered == 0),
        "all 5 reported cadence journals must have zero coverage after discard: {cadence:?}"
    );
    assert!(cadence["relational.canonical-commits"].1 > 0);
    assert!(cadence["world.occurrences"].1 > 0);
    assert!(cadence["world.custody"].1 > 0);

    let recovered_state = recovered.state.as_ref().unwrap();
    if kill {
        // Immediate kill-reopen parity BEFORE continuation: full authored truth, history,
        // occurrences, idempotency, head, and custody match MemoryOnly under
        // DurableIdentityCorrespondence, with explicit recovered posture and allowed actions.
        comparison::compare(&mut correspondence, recovered_state, &reference_state).unwrap();
        assert_eq!(recovered_state.custody.len(), 1);
        let custody = &recovered_state.custody[0];
        assert_eq!(custody.owner, original_custody.owner);
        assert_eq!(custody.identity, original_custody.identity);
        assert_eq!(custody.effects, Some(1));
        assert_eq!(custody.settlement, Some(false));
        assert_eq!(custody.cause, Some(CustodyCause::RecoveredWithoutProductMovement));
        assert_eq!(custody.expected, Some(abandoned_state.head.clone()));
        assert_eq!(
            custody.actions.as_deref(),
            Some(
                &[
                    CustodyAction::Inspect,
                    CustodyAction::ContinueOriginalPublication,
                    CustodyAction::ReleaseObligations,
                    CustodyAction::CloseOwner,
                ][..]
            )
        );
        assert_eq!(custody.original_effect, original_custody.original_effect);
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

    // 5. Post-reopen replay of committed keys (1 and 2) + post-reopen IntentDrift on key 1.
    for (key, value, expected_fate) in [
        (1_u64, 11_u64, receipts::MutationFate::Committed),
        (2_u64, 22_u64, receipts::MutationFate::Committed),
        (1_u64, 999_u64, receipts::MutationFate::IntentDrift),
    ] {
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
        assert_eq!(replayed.contacts, 0, "replaying or drifting key invokes no handler");
        let actual_mutation = replayed.mutation.as_ref().unwrap();
        assert_eq!(actual_mutation.fate, expected_fate);
        assert_eq!(expected.fate, expected_fate);
        reference_state = projection::observe(reference.runtime(), &execution);
        let replayed_state = replayed.state.as_ref().unwrap();
        assert_eq!(replayed_state.head, recovered_state.head);
        assert_eq!(replayed_state.occurrences, recovered_state.occurrences);
        receipts::compare(
            correspondence.identities_mut(),
            actual_mutation,
            &expected,
            replayed_state,
            &reference_state,
        )
        .unwrap();
        comparison::compare(&mut correspondence, replayed_state, &reference_state).unwrap();
    }

    // 6. Under OS kill, continue the live recovered custody (key 3, value 33) -> completes once,
    // matching an independent MemoryOnly reference across full truth, history, occurrences,
    // receipt, and empty custody, and retrying continuation cannot settle or advance again.
    if let Some(cont_ref) = continued_reference {
        let expected_continued = super::super::execution::mutation(
            &cont_ref,
            &HostCommand::Mutate {
                branch: BranchLabel::Main,
                key: 3,
                value: 33,
            },
            &mut continued_originals,
        )
        .unwrap();
        let continued = session.send(Request::ContinueOriginal {
            key: 3,
            value: 33,
            pause_after_disposition: false,
        });
        assert_eq!(continued.contacts, 0, "continuing custody never re-runs handler");
        let continued_mutation = match continued.disposition {
            Some(Disposition::Continued(mutation)) => mutation,
            other => panic!("expected Continued disposition, got {other:?}"),
        };
        let cont_ref_state = projection::observe(cont_ref.runtime(), &execution);
        let continued_state = continued.state.as_ref().unwrap();
        assert!(continued_state.custody.is_empty());
        assert_eq!(continued_state.head.generation, abandoned_state.head.generation + 1);
        assert_eq!(continued_state.occurrences.len(), abandoned_state.occurrences.len() + 1);
        receipts::compare(
            continued_correspondence.identities_mut(),
            &continued_mutation,
            &expected_continued,
            continued_state,
            &cont_ref_state,
        )
        .unwrap();
        comparison::compare(
            &mut continued_correspondence,
            continued_state,
            &cont_ref_state,
        )
        .unwrap();

        let retry_continue = session.send(Request::ContinueOriginal {
            key: 3,
            value: 33,
            pause_after_disposition: false,
        });
        assert_eq!(retry_continue.contacts, 0);
        let retry_state = retry_continue.state.as_ref().unwrap();
        assert!(retry_state.custody.is_empty());
        assert_eq!(retry_state.head, continued_state.head);
        assert_eq!(retry_state.occurrences, continued_state.occurrences);
        let retry_mutation = match retry_continue.disposition {
            Some(Disposition::Continued(mutation)) => mutation,
            other => panic!("expected Continued retry disposition, got {other:?}"),
        };
        assert_eq!(retry_mutation, continued_mutation);
        comparison::compare(&mut continued_correspondence, retry_state, &cont_ref_state).unwrap();
        let refused_cont = cont_ref.close(&execution).err().unwrap();
        drop(refused_cont.resume().unwrap());
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
