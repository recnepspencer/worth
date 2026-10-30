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

pub use std::num::NonZeroU64;
pub use std::path::Path;

pub use {sha2::Digest, sha2::Sha256};
pub use {
    worth_store::physical_runtime::certification::CertificationMediaFaultActivation,
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
pub use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::FilesystemAccessPosture,
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::OrderedJournalPublication,
    worth_store::physical_runtime::OrderedJournalPublicationFailure,
    worth_store::physical_runtime::OrderedJournalResolution,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub use {
    crate::production_entry::certified_release_serving, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};

pub use crate::ordered_journals::canonical_journal_oracle::relational_reference::RelationalReference;

pub const SEED: u64 = 0x5eed_0001;
pub const COMMITS: usize = 6;
/// Targets: the journal's first entry (no anchor yet) and a later one.
pub const TARGETS: [u64; 2] = [1, 4];

pub fn placement() -> AdmittedRecordPlacementPolicy {
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(format())
        .unwrap()
}

pub fn format() -> AdmittedPhysicalRecordFormat {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    )
}

pub fn deadline() -> PhysicalMutationDeadline {
    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
}

pub fn publish(
    serving: &ServingPhysicalRuntime,
    position: u64,
    bytes: &[u8],
) -> Result<(), OrderedJournalPublicationFailure> {
    serving
        .canonical_commits()
        .map_err(|_| worth_store::physical_runtime::OrderedJournalPublicationFailure::EmptyPayload)?
        .publish(
            worth_store::physical_runtime::OrderedJournalPublication::new(
                position,
                bytes.to_vec(),
                placement(),
                deadline(),
            ),
        )
        .map(|_| ())
}

/// Opens the cleanly closed store with one fault rule per `(role, n)`, each
/// firing at the nth identified operation of its role after its activation.
/// `None` when the schedule refuses a directive for its role.
pub fn open_with_faults(
    root: &Path,
    faults: &[(MediaOperationRole, NonZeroU64, MediaFaultDirective)],
) -> Option<(
    ServingPhysicalRuntime,
    Vec<CertificationMediaFaultActivation>,
)> {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
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
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap(),
    )
    .unwrap();
    let worth_proof::TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    else {
        panic!("fault-scheduled media must admit")
    };
    let worth_proof::TransitionOutcome::Success(durability) =
        crate::production_entry::certified_release_serving::serving_durability(
            std::num::NonZeroU64::new(16 << 20).unwrap(),
        )
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw()
    else {
        panic!("durability must admit")
    };
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format())
        .unwrap();
    let worth_proof::TransitionOutcome::Success(serving) = media
        .open_record_store(
            worth_store::physical_runtime::PhysicalRecordOpen::new(format(), access, durability)
                .with_journal_registry(
                    worth_store::physical_runtime::OrderedJournalRegistry::declare(
                        2,
                        &[
                            worth_store::physical_runtime::RELATIONAL_CANONICAL_COMMITS,
                            worth_store::physical_runtime::RELATIONAL_BRANCH_TRANSITIONS,
                        ],
                        &["home", "relational.checkpoint"],
                    )
                    .unwrap(),
                ),
        )
        .into_raw()
    else {
        panic!("a cleanly closed store must open")
    };
    Some((serving, activations))
}

/// A cleanly closed store holding reference entries `1..target`.
pub fn prepared_store(
    reference: &RelationalReference,
    target: u64,
) -> worth_store_test_support::TemporaryDirectory {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("journal-oracle")
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

pub fn serve(root: &Path, stage: &str) -> ServingPhysicalRuntime {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        crate::production_entry::certified_release_serving::recovery_request::request(root),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("{stage} did not recover: {outcome:?}");
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .unwrap();
    crate::production_entry::certified_release_serving::admit_serving_with_seal(root, seal)
}

/// Requires the journal to be exactly reference entries `1..=head`.
pub fn assert_prefix(serving: &ServingPhysicalRuntime, reference: &RelationalReference, head: u64) {
    let journal = serving.canonical_commits().expect("journal access");
    assert_eq!(
        journal.head().unwrap().map_or(0, |entry| entry.position()),
        head
    );
    for (index, expected) in reference.artifacts.iter().enumerate() {
        let position = index as u64 + 1;
        let digest: [u8; 32] = sha2::Sha256::digest(expected).into();
        let resolution = journal.resolve(position, digest).unwrap();
        if position <= head {
            assert!(matches!(
                resolution,
                worth_store::physical_runtime::OrderedJournalResolution::Committed(_)
            ));
            let read = journal.read(position, 1 << 24).unwrap();
            assert!(
                read.bytes() == expected.as_slice(),
                "entry {position} changed"
            );
            reference.assert_readmits(position, read.bytes());
        } else {
            assert_eq!(
                resolution,
                worth_store::physical_runtime::OrderedJournalResolution::NotCommitted { head }
            );
        }
    }
}

/// Counts each role's identified operations while `target` publishes.
pub fn measure(reference: &RelationalReference, target: u64) -> Vec<(MediaOperationRole, u64)> {
    let store = prepared_store(reference, target);
    let root = store.path().to_path_buf();
    let faults: Vec<_> = worth_store::physical_runtime::certification::MediaOperationRole::ALL
        .iter()
        .map(|role| {
            let never =
                worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
                    kind: std::io::ErrorKind::Other,
                    raw_os_error: None,
                };
            (*role, std::num::NonZeroU64::MAX, never)
        })
        .collect();
    let (serving, activations) =
        open_with_faults(&root, &faults).expect("fail-before is admissible for every role");
    for activation in &activations {
        activation.arm().unwrap();
    }
    publish(&serving, target, &reference.artifacts[target as usize - 1])
        .expect("the measured publication completes");
    let counts = worth_store::physical_runtime::certification::MediaOperationRole::ALL
        .iter()
        .zip(&activations)
        .map(|(role, activation)| (*role, activation.matching_operation_count()))
        .filter(|(_, count)| *count > 0)
        .collect();
    let _ = serving.close();
    counts
}

/// One faulted publication of `target`, then production recovery.
pub fn run_case(
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
        Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::ProvenNoEffect(_)) => {
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
