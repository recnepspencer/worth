//! Crash-consistency oracle for the canonical commit journal.
//!
//! The reference is a memory-only Relational runtime's real canonical commit
//! artifacts. For one target publication, the oracle first counts the media
//! operations of every role that publication performs, then reruns it once per
//! (role, ordinal, directive) with a fault at exactly that operation, abandons
//! the runtime, recovers in the production path and requires: the journal is a
//! prefix of the reference holding every acknowledged entry; each entry
//! readmits through Relational; `resolve` agrees with the prefix; the suffix
//! publishes; and a second recovery reaches the same journal.

use std::num::NonZeroU64;
use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::certification::{
    CertificationMediaFaultActivation, MediaFaultDirective, MediaOperationRole,
};
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy, FilesystemAccessPosture,
    FilesystemMediaAdmission, ManifestEntryCapacity, OrderedJournalPublication,
    OrderedJournalPublicationFailure, OrderedJournalResolution, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordOpen, PhysicalRecordPlacementPolicy,
    PhysicalRuntimeAdmission, PhysicalStore, ServingPhysicalRuntime,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

use super::*;

#[path = "canonical_journal_oracle/relational_reference.rs"]
mod relational_reference;
use relational_reference::RelationalReference;

const SEED: u64 = 0x5eed_0001;
const COMMITS: usize = 6;
/// Targets: the journal's first entry (no anchor yet) and a later one.
const TARGETS: [u64; 2] = [1, 4];

pub(crate) fn placement() -> AdmittedRecordPlacementPolicy {
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(format())
        .unwrap()
}

fn format() -> AdmittedPhysicalRecordFormat {
    AdmittedPhysicalRecordFormat::admit(PhysicalRecordFormatDeclaration::builder().admit().unwrap())
}

fn deadline() -> PhysicalMutationDeadline {
    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
}

fn publish(
    serving: &ServingPhysicalRuntime,
    position: u64,
    bytes: &[u8],
) -> Result<(), OrderedJournalPublicationFailure> {
    serving
        .canonical_commits()
        .map_err(|_| OrderedJournalPublicationFailure::EmptyPayload)?
        .publish(OrderedJournalPublication::new(
            position,
            bytes.to_vec(),
            placement(),
            deadline(),
        ))
        .map(|_| ())
}

/// Opens the cleanly closed store with one fault rule per `(role, n)`, each
/// firing at the nth identified operation of its role after its activation.
/// `None` when the schedule refuses a directive for its role.
pub(crate) fn open_with_faults(
    root: &Path,
    faults: &[(MediaOperationRole, NonZeroU64, MediaFaultDirective)],
) -> Option<(
    ServingPhysicalRuntime,
    Vec<CertificationMediaFaultActivation>,
)> {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let activations: Vec<_> = faults
        .iter()
        .map(|_| authority.one_shot_activation())
        .collect();
    let rules = faults
        .iter()
        .zip(&activations)
        .map(|((role, nth, directive), activation)| {
            authority
                .rule(*role, 1, directive.clone())
                .for_nth_identified_operation_after_activation(activation.clone(), *nth)
        })
        .collect();
    let schedule = authority.schedule(rules).ok()?;
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    else {
        panic!("fault-scheduled media must admit")
    };
    let TransitionOutcome::Success(durability) =
        certified_release_serving::serving_durability(NonZeroU64::new(16 << 20).unwrap())
            .admit(media.physical_durability_admission_basis().unwrap())
            .into_raw()
    else {
        panic!("durability must admit")
    };
    let access = PhysicalRecordAccessPolicy::builder()
        .admit(format())
        .unwrap();
    let TransitionOutcome::Success(serving) = media
        .open_record_store(PhysicalRecordOpen::new(format(), access, durability))
        .into_raw()
    else {
        panic!("a cleanly closed store must open")
    };
    Some((serving, activations))
}

/// A cleanly closed store holding reference entries `1..target`.
fn prepared_store(
    reference: &RelationalReference,
    target: u64,
) -> worth_store_test_support::TemporaryDirectory {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("journal-oracle")
        .expect("oracle store");
    for position in 1..target {
        publish(
            world.serving(),
            position,
            &reference.artifacts[position as usize - 1],
        )
        .expect("prefix publication");
    }
    let root = world.retained_root();
    let _ = world.close();
    root
}

pub(crate) fn serve(root: &Path, stage: &str) -> ServingPhysicalRuntime {
    let outcome = WorthStoreRecovery::recover(certified_release_serving::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("{stage} did not recover: {outcome:?}");
    };
    let seal = handoff.into_core().into_checkpoint_custody().unwrap();
    certified_release_serving::admit_serving_with_seal(root, seal)
}

/// Requires the journal to be exactly reference entries `1..=head`.
fn assert_prefix(serving: &ServingPhysicalRuntime, reference: &RelationalReference, head: u64) {
    let journal = serving.canonical_commits().expect("journal access");
    assert_eq!(
        journal.head().unwrap().map_or(0, |entry| entry.position()),
        head
    );
    for (index, expected) in reference.artifacts.iter().enumerate() {
        let position = index as u64 + 1;
        let digest: [u8; 32] = Sha256::digest(expected).into();
        let resolution = journal.resolve(position, digest).unwrap();
        if position <= head {
            assert!(matches!(resolution, OrderedJournalResolution::Committed(_)));
            let read = journal.read(position, 1 << 24).unwrap();
            assert!(
                read.bytes() == expected.as_slice(),
                "entry {position} changed"
            );
            reference.assert_readmits(position, read.bytes());
        } else {
            assert_eq!(resolution, OrderedJournalResolution::NotCommitted { head });
        }
    }
}

/// Counts each role's identified operations while `target` publishes.
fn measure(reference: &RelationalReference, target: u64) -> Vec<(MediaOperationRole, u64)> {
    let store = prepared_store(reference, target);
    let root = store.path().to_path_buf();
    let faults: Vec<_> = MediaOperationRole::ALL
        .iter()
        .map(|role| {
            let never = MediaFaultDirective::FailBefore {
                kind: std::io::ErrorKind::Other,
                raw_os_error: None,
            };
            (*role, NonZeroU64::MAX, never)
        })
        .collect();
    let (serving, activations) =
        open_with_faults(&root, &faults).expect("fail-before is admissible for every role");
    for activation in &activations {
        activation.arm().unwrap();
    }
    publish(&serving, target, &reference.artifacts[target as usize - 1])
        .expect("the measured publication completes");
    let counts = MediaOperationRole::ALL
        .iter()
        .zip(&activations)
        .map(|(role, activation)| (*role, activation.matching_operation_count()))
        .filter(|(_, count)| *count > 0)
        .collect();
    let _ = serving.close();
    counts
}

/// One faulted publication of `target`, then production recovery.
fn run_case(
    reference: &RelationalReference,
    target: u64,
    role: MediaOperationRole,
    nth: NonZeroU64,
    directive: MediaFaultDirective,
) {
    let store = prepared_store(reference, target);
    let root = store.path().to_path_buf();
    let Some((serving, activations)) = open_with_faults(&root, &[(role, nth, directive.clone())])
    else {
        return;
    };
    activations[0].arm().unwrap();
    let outcome = publish(&serving, target, &reference.artifacts[target as usize - 1]);
    assert!(activations[0].is_consumed(), "{role:?}#{nth} did not fire");
    let _ = serving.abort();
    let case = format!("target {target} {role:?}#{nth} {directive:?}");
    let recovered = serve(&root, &case);
    let head = recovered
        .canonical_commits()
        .unwrap()
        .head()
        .unwrap()
        .map_or(0, |entry| entry.position());
    match &outcome {
        Ok(()) => assert_eq!(head, target, "{case}: acknowledged entry lost"),
        Err(OrderedJournalPublicationFailure::ProvenNoEffect(_)) => {
            assert_eq!(
                head,
                target - 1,
                "{case}: a no-effect publication committed"
            )
        }
        Err(_) => assert!(head == target - 1 || head == target, "{case}: head {head}"),
    }
    assert_prefix(&recovered, reference, head);
    for position in head + 1..=COMMITS as u64 {
        publish(
            &recovered,
            position,
            &reference.artifacts[position as usize - 1],
        )
        .unwrap_or_else(|failure| panic!("{case}: suffix {position}: {failure:?}"));
    }
    assert_prefix(&recovered, reference, COMMITS as u64);
    let _ = recovered.close();
    let again = serve(&root, &format!("{case} second recovery"));
    assert_prefix(&again, reference, COMMITS as u64);
    let _ = again.close();
}

#[test]
fn every_media_operation_of_a_publication_recovers_to_a_reference_prefix() {
    let reference = RelationalReference::generate(SEED, COMMITS);
    for target in TARGETS {
        let counts = measure(&reference, target);
        assert!(
            !counts.is_empty(),
            "the publication performed media operations"
        );
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
                    run_case(&reference, target, role, nth, directive);
                }
            }
        }
    }
}
