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

use std::num::NonZeroU64;
use std::path::Path;

use worth_store::physical_runtime::certification::{MediaFaultDirective, MediaOperationRole};
use worth_store::physical_runtime::{
    OrderedJournalPublication, PhysicalRecoveryYieldpointStage, ServingPhysicalRuntime,
};

use super::canonical_journal::recovery_interruption::kill_recovery_at;
use super::canonical_journal_oracle::{open_with_faults, placement, serve};
use super::*;

/// Entries before the checkpoint the sweep interrupts.
const ENTRIES: u64 = 6;

/// Which checkpoint the sweep interrupts.
#[derive(Clone, Copy, Debug)]
enum Target {
    /// The store's first checkpoint: nothing is selected before it.
    First,
    /// A checkpoint that replaces a selected one; half of the entries lie
    /// above the selected checkpoint.
    Replacement,
}

fn payload(position: u64) -> Vec<u8> {
    let length = 48 + (position as usize * 997) % 4_000;
    (0..length)
        .map(|index| (position as u8).wrapping_mul(31) ^ (index as u8))
        .collect()
}

fn publish(serving: &ServingPhysicalRuntime, position: u64) {
    serving
        .canonical_commits()
        .expect("journal access")
        .publish(OrderedJournalPublication::new(
            position,
            payload(position),
            placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
        .unwrap_or_else(|failure| panic!("entry {position}: {failure:?}"));
}

fn checkpoint(serving: &ServingPhysicalRuntime, key: u8) -> Option<PhysicalCheckpointOutcome> {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => Some(handle.wait()),
        _ => None,
    }
}

/// A cleanly closed store holding every entry, with the first half covered
/// by a selected checkpoint when the target replaces one.
fn prepared_store(target: Target) -> worth_store_test_support::TemporaryDirectory {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("checkpoint-oracle")
        .expect("oracle store");
    for position in 1..=ENTRIES {
        publish(world.serving(), position);
        if matches!(target, Target::Replacement) && position == ENTRIES / 2 {
            assert!(matches!(
                checkpoint(world.serving(), 0xa1),
                Some(PhysicalCheckpointOutcome::Completed(_))
            ));
        }
    }
    let root = world.retained_root();
    let _ = world.close();
    root
}

/// Requires the journal to hold exactly entries `1..=head`.
fn assert_journal(serving: &ServingPhysicalRuntime, head: u64, case: &str) {
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
fn measure(target: Target) -> Vec<(MediaOperationRole, u64)> {
    let store = prepared_store(target);
    let never = MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::Other,
        raw_os_error: None,
    };
    let faults: Vec<_> = MediaOperationRole::ALL
        .iter()
        .map(|role| (*role, NonZeroU64::MAX, never.clone()))
        .collect();
    let (serving, activations) =
        open_with_faults(store.path(), &faults).expect("fail-before is admissible for every role");
    for activation in &activations {
        activation.arm().unwrap();
    }
    assert!(matches!(
        checkpoint(&serving, 0xb2),
        Some(PhysicalCheckpointOutcome::Completed(_))
    ));
    let counts = MediaOperationRole::ALL
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
fn interrupts_recovery(role: MediaOperationRole, directive: &MediaFaultDirective) -> bool {
    matches!(directive, MediaFaultDirective::IndeterminateAfterEffect)
        && matches!(
            role,
            MediaOperationRole::SynchronizeFileState | MediaOperationRole::AtomicReplace
        )
}

/// One faulted checkpoint, then production recovery. Returns whether a
/// recovery was killed at its obligation discharge.
fn run_case(
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
        && kill_recovery_at(root, PhysicalRecoveryYieldpointStage::ObligationDischarge);
    assert_recovers(root, &case);
    killed
}

/// Recovery serves without inspection, keeps every entry, publishes the next
/// one, and a second recovery reaches the same journal.
fn assert_recovers(root: &Path, case: &str) {
    let recovered = serve(root, case);
    assert_journal(&recovered, ENTRIES, case);
    publish(&recovered, ENTRIES + 1);
    let _ = recovered.close();
    let again = serve(root, &format!("{case} second recovery"));
    assert_journal(&again, ENTRIES + 1, case);
    let _ = again.close();
}

fn sweep(target: Target) {
    let counts = measure(target);
    assert!(
        !counts.is_empty(),
        "the checkpoint performed media operations"
    );
    let mut cases = 0;
    let mut killed = 0;
    for (role, count) in counts {
        for nth in 1..=count {
            let nth = NonZeroU64::new(nth).unwrap();
            for directive in [
                MediaFaultDirective::FailBefore {
                    kind: std::io::ErrorKind::Other,
                    raw_os_error: None,
                },
                MediaFaultDirective::IndeterminateAfterEffect,
            ] {
                cases += 1;
                killed += usize::from(run_case(target, role, nth, directive));
            }
        }
    }
    eprintln!("checkpoint oracle {target:?}: {cases} cases, {killed} recoveries killed");
    assert!(
        killed > 0,
        "no recovery was killed at its obligation discharge"
    );
}

#[test]
fn every_media_operation_of_a_checkpoint_replacement_recovers() {
    sweep(Target::Replacement);
}

#[test]
fn every_media_operation_of_a_first_checkpoint_recovers() {
    sweep(Target::First);
}
