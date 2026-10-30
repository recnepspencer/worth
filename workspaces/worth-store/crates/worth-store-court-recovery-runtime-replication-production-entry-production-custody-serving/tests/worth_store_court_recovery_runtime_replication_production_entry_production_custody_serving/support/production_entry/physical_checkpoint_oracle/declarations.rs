//! Crash-consistency oracle for one fuzzy Store physical checkpoint above
//! committed journal entries.
//!
//! The oracle counts every media operation of every role the checkpoint
//! performs, then reruns it once per (role, ordinal, directive) with a fault
//! at exactly that operation, abandons the runtime and recovers through the
//! production path. Recovery must settle whatever the interrupted checkpoint
//! left: the recovered runtime serves without inspection, holds every entry,
//! publishes the next one, and a second recovery reaches the same journal.
//! For the faults that leave obligations recovery must adjudicate, the first
//! recovery is also killed just before it discharges them, and the next one
//! must settle to the same result.

pub(crate) use std::num::NonZeroU64;
pub(crate) use std::path::Path;

pub(crate) use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
pub(crate) use {
    worth_store::physical_runtime::OrderedJournalPublication,
    worth_store::physical_runtime::PhysicalRecoveryYieldpointStage,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::recovery_interruption::kill_recovery_at};
pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::open_with_faults, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome};

/// Entries before the checkpoint the sweep interrupts.
pub(crate) const ENTRIES: u64 = 6;

/// Which checkpoint the sweep interrupts.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Target {
    /// The store's first checkpoint: nothing is selected before it.
    First,
    /// A checkpoint that replaces a selected one; half of the entries lie
    /// above the selected checkpoint.
    Replacement,
}

pub(crate) fn payload(position: u64) -> Vec<u8> {
    let length = 48 + (position as usize * 997) % 4_000;
    (0..length)
        .map(|index| (position as u8).wrapping_mul(31) ^ (index as u8))
        .collect()
}

pub(crate) fn publish(serving: &ServingPhysicalRuntime, position: u64) {
    serving
        .canonical_commits()
        .expect("journal access")
        .publish(
            worth_store::physical_runtime::OrderedJournalPublication::new(
                position,
                payload(position),
                placement(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .unwrap_or_else(|failure| panic!("entry {position}: {failure:?}"));
}

pub(crate) fn checkpoint(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> Option<PhysicalCheckpointOutcome> {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([key; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    match serving.checkpoints().start(request).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => Some(handle.wait()),
        _ => None,
    }
}

/// A cleanly closed store holding every entry, with the first half covered
/// by a selected checkpoint when the target replaces one.
pub(crate) fn prepared_store(target: Target) -> worth_store_test_support::TemporaryDirectory {
    let _ = worth_store_testkit_recovery_runtime::setup_observation::record(
        "physical-checkpoint-prefix",
        &|inputs| {
            inputs.boolean(
                "replaces-checkpoint",
                match target {
                    Target::First => false,
                    Target::Replacement => true,
                },
            );
            inputs.unsigned("entries", ENTRIES);
            for position in 1..=ENTRIES {
                inputs.unsigned("journal-position", position);
                inputs.bytes("journal-payload", &payload(position));
            }
            if matches!(target, Target::Replacement) {
                inputs.unsigned("checkpoint-after-position", ENTRIES / 2);
                inputs.bytes("checkpoint-key", &[0xa1; 32]);
            }
        },
    );
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("checkpoint-oracle")
        .expect("oracle store");
    for position in 1..=ENTRIES {
        publish(world.serving(), position);
        if matches!(
            target,
            crate::production_entry::physical_checkpoint_oracle::Target::Replacement
        ) && position == ENTRIES / 2
        {
            assert!(matches!(
                checkpoint(world.serving(), 0xa1),
                Some(worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_))
            ));
        }
    }
    let root = world.retained_root();
    let _ = world.close();
    root
}

/// Requires the journal to hold exactly entries `1..=head`.
pub(crate) fn assert_journal(serving: &ServingPhysicalRuntime, head: u64, case: &str) {
    let journal = serving
        .canonical_commits()
        .unwrap_or_else(|denial| panic!("{case}: serving refused the journal: {denial:?}"));
    assert_eq!(
        journal.head().unwrap().map_or(0, |entry| entry.position()),
        head,
        "{case}"
    );
    for position in 1..=head {
        let read = journal.read(position, 1 << 20).unwrap();
        assert!(
            read.bytes() == payload(position),
            "{case}: entry {position}"
        );
    }
}

/// Counts each role's identified operations while the target checkpoint runs.
pub(crate) fn measure(target: Target) -> Vec<(MediaOperationRole, u64)> {
    let store = prepared_store(target);
    let never = worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::Other,
        raw_os_error: None,
    };
    let faults: Vec<_> = worth_store::physical_runtime::certification::MediaOperationRole::ALL
        .iter()
        .map(|role| (*role, std::num::NonZeroU64::MAX, never.clone()))
        .collect();
    let (serving, activations) =
        open_with_faults(store.path(), &faults).expect("fail-before is admissible for every role");
    for activation in &activations {
        activation.arm().unwrap();
    }
    assert!(matches!(
        checkpoint(&serving, 0xb2),
        Some(worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_))
    ));
    let counts = worth_store::physical_runtime::certification::MediaOperationRole::ALL
        .iter()
        .zip(&activations)
        .map(|(role, activation)| (*role, activation.matching_operation_count()))
        .filter(|(_, count)| *count > 0)
        .collect();
    let _ = serving.close();
    counts
}

/// Whether the case also kills recovery before it discharges obligations:
/// an interrupted candidate synchronization (an unselected candidate) and an
/// interrupted replacement (the selected checkpoint's publication).
pub(crate) fn interrupts_recovery(
    role: MediaOperationRole,
    directive: &MediaFaultDirective,
) -> bool {
    matches!(
        directive,
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect
    ) && matches!(
        role,
        worth_store::physical_runtime::certification::MediaOperationRole::SynchronizeFileState
            | worth_store::physical_runtime::certification::MediaOperationRole::AtomicReplace
    )
}

/// One faulted checkpoint, then production recovery. Returns whether a
/// recovery was killed at its obligation discharge.
pub(crate) fn run_case(
    target: Target,
    role: MediaOperationRole,
    nth: NonZeroU64,
    directive: MediaFaultDirective,
) -> bool {
    let store = prepared_store(target);
    let root = store.path();
    let Some((serving, activations)) = open_with_faults(root, &[(role, nth, directive.clone())])
    else {
        return false;
    };
    activations[0].arm().unwrap();
    let _ = checkpoint(&serving, 0xb2);
    let case = format!("{target:?} {role:?}#{nth} {directive:?}");
    assert!(
        activations[0].is_consumed(),
        "{case}: the fault did not fire"
    );
    let _ = serving.abort();
    let killed = interrupts_recovery(role, &directive)
        && kill_recovery_at(
            root,
            worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::ObligationDischarge,
        );
    assert_recovers(root, &case);
    killed
}

/// Recovery serves without inspection, keeps every entry, publishes the next
/// one, and a second recovery reaches the same journal.
pub(crate) fn assert_recovers(root: &Path, case: &str) {
    let recovered = serve(root, case);
    assert_journal(&recovered, ENTRIES, case);
    publish(&recovered, ENTRIES + 1);
    let _ = recovered.close();
    let again = serve(root, &format!("{case} second recovery"));
    assert_journal(&again, ENTRIES + 1, case);
    let _ = again.close();
}

pub(crate) fn sweep(target: Target) {
    let counts = measure(target);
    assert!(
        !counts.is_empty(),
        "the checkpoint performed media operations"
    );
    let mut cases = Vec::new();
    for (role, count) in counts {
        for nth in 1..=count {
            let nth = std::num::NonZeroU64::new(nth).unwrap();
            for directive in [
                worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
                    kind: std::io::ErrorKind::Other,
                    raw_os_error: None,
                },
                worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
            ] {
                cases.push((role, nth, directive));
            }
        }
    }
    let scheduled = cases.len();
    let outcomes = worth_store_test_support::sweep::run(cases, |(role, nth, directive)| {
        run_case(target, role, nth, directive)
    });
    let cases = outcomes.len();
    assert_eq!(cases, scheduled, "scheduled checkpoint cases were lost");
    let killed: usize = outcomes.into_iter().map(usize::from).sum();
    eprintln!("checkpoint oracle {target:?}: {cases} cases, {killed} recoveries killed");
    assert!(
        killed > 0,
        "no recovery was killed at its obligation discharge"
    );
}
