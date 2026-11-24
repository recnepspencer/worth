use std::sync::Arc;
use std::time::{Duration, Instant};

use super::equivalence::{
    DurabilityReopenKey, EndedKey, EquivalenceLedger, InFlightClass, PostRecoveryEquivalenceKey,
    PublicationResidueKey, VerifiedPhysicalState,
};

fn sample_key() -> PostRecoveryEquivalenceKey {
    PostRecoveryEquivalenceKey {
        ended: EndedKey::Refused,
        in_flight_class: InFlightClass::Refused,
        recovered_schema: 1,
        commits: 3,
        transitions: 1,
        commit_artifacts_sha256: [1; 32],
        transition_artifacts_sha256: [2; 32],
        logical_state_sha256: [3; 32],
        transition_covered_sequence: 0,
        selected_checkpoint_generation: 1,
        physical: VerifiedPhysicalState {
            canonical_journal_head: 3,
            transition_journal_head: 1,
            checkpoint_generation: 1,
            checkpoint_relational_manifest_sha256: [6; 32],
            checkpoint_member_count: 6,
            checkpoint_members_content_sha256: [4; 32],
            application_journals_sha256: [7; 32],
            publication_residue: PublicationResidueKey {
                staging_catalog_candidate: false,
                successor_root: false,
                successor_routing_block: false,
                successor_free_space: false,
                successor_membership_blocks: false,
                next_segment_artifacts: false,
                reusable_segment_artifacts: false,
                next_arena_artifact: false,
            },
            durability_reopen: Some(DurabilityReopenKey {
                checkpoint_artifact_bytes: 4096,
                checkpoint_bytes_read: 512,
                dirty_body_bytes_skipped: 0,
                binding_records_read: 4,
                checkpoint_integrity_admissions: 1,
                wal_members_read: 2,
            }),
            recovery_journal_counters: (0, 0),
        },
        disk_layout_sha256: [5; 32],
    }
}

#[test]
fn distinct_manual_keys_never_coalesce_in_equivalence_ledger() {
    let base = sample_key();
    let mut variants = vec![base.clone()];

    let mut push_mutated = |mutate: fn(&mut PostRecoveryEquivalenceKey)| {
        let mut copy = base.clone();
        mutate(&mut copy);
        assert_ne!(copy, base);
        variants.push(copy);
    };

    push_mutated(|k| k.ended = EndedKey::Unresolved { recovered: false });
    push_mutated(|k| k.ended = EndedKey::Unresolved { recovered: true });
    push_mutated(|k| k.in_flight_class = InFlightClass::NotDurable);
    push_mutated(|k| k.recovered_schema = 2);
    push_mutated(|k| k.commits = 4);
    push_mutated(|k| k.transitions = 2);
    push_mutated(|k| k.commit_artifacts_sha256 = [9; 32]);
    push_mutated(|k| k.transition_artifacts_sha256 = [9; 32]);
    push_mutated(|k| k.logical_state_sha256 = [9; 32]);
    push_mutated(|k| k.transition_covered_sequence = 1);
    push_mutated(|k| k.selected_checkpoint_generation = 2);
    push_mutated(|k| k.physical.canonical_journal_head = 4);
    push_mutated(|k| k.physical.transition_journal_head = 2);
    push_mutated(|k| k.physical.checkpoint_generation = 2);
    push_mutated(|k| k.physical.checkpoint_relational_manifest_sha256 = [9; 32]);
    push_mutated(|k| k.physical.checkpoint_member_count = 7);
    push_mutated(|k| k.physical.checkpoint_members_content_sha256 = [9; 32]);
    push_mutated(|k| k.physical.application_journals_sha256 = [9; 32]);
    push_mutated(|k| k.physical.publication_residue.staging_catalog_candidate = true);
    push_mutated(|k| k.physical.publication_residue.next_arena_artifact = true);
    push_mutated(|k| {
        k.physical
            .durability_reopen
            .as_mut()
            .unwrap()
            .wal_members_read = 3;
    });
    push_mutated(|k| k.physical.recovery_journal_counters = (1, 2));
    push_mutated(|k| k.disk_layout_sha256 = [8; 32]);

    let ledger = EquivalenceLedger::default();
    for variant in &variants {
        let guard = ledger
            .claim_continuation(variant.clone())
            .expect("distinct key must not coalesce");
        guard.mark_succeeded();
    }
    assert!(ledger.claim_continuation(base).is_none());

    let stats = ledger.stats();
    assert_eq!(stats.distinct_classes, variants.len());
    assert_eq!(stats.total_cases, variants.len() + 1);
    assert_eq!(stats.continuations_deduped, 1);
}

#[test]
fn panicking_representative_does_not_suppress_duplicate_continuation() {
    let timeout = Duration::from_secs(5);
    let ledger = Arc::new(EquivalenceLedger::default());
    let key = sample_key();
    let (rep_claimed_tx, rep_claimed_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let (dup_done_tx, dup_done_rx) = std::sync::mpsc::channel();

    let rep = {
        let ledger = Arc::clone(&ledger);
        let key = key.clone();
        std::thread::spawn(move || {
            let _guard = ledger
                .claim_continuation(key)
                .expect("representative claims class");
            let _ = rep_claimed_tx.send(());
            let _ = release_rx.recv_timeout(timeout);
            panic!("simulated continuation failure");
        })
    };

    rep_claimed_rx
        .recv_timeout(timeout)
        .expect("representative must claim class before spawning duplicate");

    let dup = {
        let ledger = Arc::clone(&ledger);
        let key = key.clone();
        std::thread::spawn(move || {
            let guard = ledger
                .claim_continuation(key)
                .expect("duplicate must run continuation after representative panic");
            guard.mark_succeeded();
            let _ = dup_done_tx.send(());
        })
    };

    // Wait until the duplicate has deterministically entered Condvar::wait
    // before letting the representative panic and drop its guard. On timeout,
    // release and join the representative so it is never left spinning.
    let wait_deadline = Instant::now() + timeout;
    let entered_wait = loop {
        if ledger.waiter_count() > 0 {
            break true;
        }
        if Instant::now() >= wait_deadline {
            break false;
        }
        std::thread::yield_now();
    };
    let _ = release_tx.send(());
    assert!(rep.join().is_err(), "representative panicked as expected");
    assert!(entered_wait, "duplicate never entered Condvar::wait");
    dup_done_rx
        .recv_timeout(timeout)
        .expect("duplicate must be woken by Drop notify_all after representative panic");
    dup.join()
        .expect("duplicate ran and succeeded after representative panic");

    assert!(
        ledger.claim_continuation(key).is_none(),
        "only after a representative succeeds may duplicates deduplicate"
    );
}

#[test]
fn same_length_member_corruption_prevents_coalescing() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let script = crate::workload::Script::generate(0x5eed_0301, 6, 3);
    let checkpoint_index = script
        .steps
        .iter()
        .position(|step| matches!(step, crate::workload::Step::Checkpoint))
        .unwrap();
    let prefix = &script.steps[..=checkpoint_index];
    let prepared = crate::prepared_home::PreparedHome::prepare(prefix, &execution);
    let schema = crate::durable_run::served_schema(prefix);

    // 1. Clean reopen verifies checkpoint sections, members, and all Application journals.
    let clean = crate::durable_run::open_declaring(prepared.location(), schema, &execution)
        .expect("clean checkpointed home opens");
    let clean_physical = crate::media_case::verified_recovered_physical(&clean)
        .expect("clean home checkpoint members verify byte-for-byte");
    assert!(clean_physical.checkpoint_member_count > 0);
    assert_ne!(
        clean_physical.checkpoint_relational_manifest_sha256,
        [0u8; 32]
    );
    assert_ne!(clean_physical.application_journals_sha256, [0u8; 32]);
    let prepared_two = crate::prepared_home::PreparedHome::prepare(prefix, &execution);
    let clean_two = crate::durable_run::open_declaring(prepared_two.location(), schema, &execution)
        .expect("second clean home opens");
    let second_physical = crate::media_case::verified_recovered_physical(&clean_two)
        .expect("second clean home verifies");
    assert_eq!(
        clean_physical, second_physical,
        "two identical prepared homes must produce identical VerifiedPhysicalState"
    );
    drop(clean_two);
    let member_bytes = crate::media_case::first_checkpoint_member_bytes(&clean);
    drop(clean);

    // 2. Flip 1 byte in-place inside a materialized member file under `families/`.
    let flipped: usize = std::fs::read_dir(prepared.location().join("families"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.file_name().unwrap() != "wal")
        .map(|path| {
            crate::open_refusals::flip_after(&path, &member_bytes, member_bytes.len() - 1, &|_| {
                true
            })
        })
        .sum();
    assert!(flipped > 0, "expected to flip materialized member bytes");

    // 3. Reopen the damaged home: content verification MUST fail (`None`).
    let damaged = crate::durable_run::open_declaring(prepared.location(), schema, &execution)
        .expect("damaged cold member still allows initial reopen before member read");
    assert!(
        crate::media_case::verified_recovered_physical(&damaged).is_none(),
        "same-length 1-byte member corruption must fail verification and force continuation"
    );
    drop(damaged);
}
