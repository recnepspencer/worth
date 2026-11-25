//! Phase 4 decisive journey: true lost caller response, immediate OS kill, full projection parity.
use super::super::protocol::Response;
use super::*;

#[test]
fn decisive_journey_lost_response_kill_reopen_before_first_checkpoint() {
    decisive_journey(false);
}

#[test]
fn decisive_journey_lost_response_kill_reopen_after_first_checkpoint() {
    decisive_journey(true);
}

fn decisive_journey(take_first_checkpoint: bool) {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("home");
    let mut generation = 0;
    let (mut session, opened) = start(&path, root.path(), &mut generation);
    let mut expected_contacts = opened.contacts;
    if take_first_checkpoint {
        let seeded = session.send(Request::Mutate { key: 1, value: 10 });
        assert_eq!(
            seeded.mutation.as_ref().map(|m| m.fate),
            Some(receipts::MutationFate::Committed)
        );
        expected_contacts = seeded.contacts;
        let checkpointed = session.send(Request::Checkpoint);
        assert!(matches!(
            checkpointed.disposition,
            Some(Disposition::Checkpointed { generation }) if generation >= 1
        ));
    }
    assert_checkpoint_posture(
        &mut session,
        take_first_checkpoint,
        "before decisive mutation",
    );

    // Execute the decisive mutation without delivering any caller response,
    // wait only for the passive post-commit observation file, verify no caller
    // response was emitted, and kill immediately with zero intervening commands.
    session.dispatch(Request::MutateDroppingResponse { key: 2, value: 20 });
    let observed_file = session.channel.join("dropped_response_observation");
    let until = Instant::now() + Duration::from_secs(120);
    while !observed_file.is_file() {
        assert!(
            session.child.try_wait().unwrap().is_none() && Instant::now() < until,
            "child did not reach dropped-response boundary"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        !session.channel.join("response").exists(),
        "caller must not receive a response on the lost-response journey"
    );
    let dropped: Response =
        serde_json::from_slice(&std::fs::read(&observed_file).unwrap()).unwrap();
    session.kill();

    assert_eq!(dropped.contacts, expected_contacts + 1);
    let first_mutation = dropped.mutation.unwrap();
    assert_eq!(first_mutation.fate, receipts::MutationFate::Committed);
    let first_receipt = first_mutation.receipt.expect("committed original receipt");
    let first_state = dropped.state.unwrap();
    assert!(first_state.custody.is_empty());

    let (mut session, reopened) = start(&path, root.path(), &mut generation);
    assert_eq!(reopened.initialized, 0);
    assert_eq!(reopened.contacts, 0);
    let reopened_state = reopened.state.unwrap();
    assert_full_projection_equal(&reopened_state, &first_state, "after kill/reopen");
    assert!(reopened_state.custody.is_empty());
    assert_checkpoint_posture(&mut session, take_first_checkpoint, "after kill/reopen");

    let retry = session.send(Request::Mutate { key: 2, value: 20 });
    assert_eq!(retry.contacts, 0, "same-key retry must not rerun handler");
    let retry_mutation = retry.mutation.unwrap();
    assert_eq!(retry_mutation.fate, receipts::MutationFate::Committed);
    assert_eq!(retry_mutation.receipt.as_ref(), Some(&first_receipt));
    let retry_state = retry.state.unwrap();
    assert_full_projection_equal(&retry_state, &first_state, "after same-key retry");
    assert!(retry_state.custody.is_empty());

    let drift = session.send(Request::Mutate { key: 2, value: 99 });
    assert_eq!(drift.contacts, 0);
    assert_eq!(
        drift.mutation.as_ref().map(|m| m.fate),
        Some(receipts::MutationFate::IntentDrift)
    );
    let drift_state = drift.state.unwrap();
    assert_full_projection_equal(&drift_state, &first_state, "after intent drift refusal");
    assert!(drift_state.custody.is_empty());

    let next = session.send(Request::Mutate { key: 3, value: 30 });
    assert_eq!(next.contacts, 1);
    assert_eq!(
        next.mutation.as_ref().map(|m| m.fate),
        Some(receipts::MutationFate::Committed)
    );
    let next_state = next.state.unwrap();
    assert_eq!(next_state.head.generation, first_state.head.generation + 1);
    assert_eq!(next_state.head.commit, first_state.head.commit + 1);
    assert!(
        next_state.occurrences.iter().any(|entry| {
            entry.commit == next_state.head.commit && entry.parent == Some(first_state.head.commit)
        }),
        "fresh key must commit one new occurrence chained to the prior head: {:?}",
        next_state.occurrences
    );
    assert_eq!(
        next_state.idempotency.len(),
        first_state.idempotency.len() + 1
    );
    assert!(next_state.custody.is_empty());
}

fn assert_full_projection_equal(
    actual: &host_observation::projection::StateProjection,
    expected: &host_observation::projection::StateProjection,
    label: &str,
) {
    assert_eq!(actual.head, expected.head, "head mismatch ({label})");
    assert_eq!(
        actual.occurrences, expected.occurrences,
        "occurrences mismatch ({label})"
    );
    assert_eq!(
        actual.entities, expected.entities,
        "entities mismatch ({label})"
    );
    assert_eq!(
        actual.relations, expected.relations,
        "relations mismatch ({label})"
    );
    assert_eq!(
        actual.idempotency, expected.idempotency,
        "idempotency mismatch ({label})"
    );
}
