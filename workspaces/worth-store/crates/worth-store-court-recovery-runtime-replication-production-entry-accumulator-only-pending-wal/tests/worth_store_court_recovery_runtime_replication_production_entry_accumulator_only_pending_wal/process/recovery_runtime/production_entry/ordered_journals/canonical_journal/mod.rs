//! The canonical commit journal publishes at the next position under one
//! durable root decision, answers point and range reads, resolves positions
//! from durable state, and survives a killed process on both sides of the
//! first checkpoint.

#[allow(unused_imports)]
use worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::*;
use {
    std::fs, std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio,
    std::thread, std::time::Duration, std::time::Instant,
};

use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
use {sha2::Digest, sha2::Sha256};
use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::OrderedJournalPublication,
    worth_store::physical_runtime::OrderedJournalPublicationFailure,
    worth_store::physical_runtime::OrderedJournalPublished,
    worth_store::physical_runtime::OrderedJournalResolution,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

use {
    worth_proof::TransitionOutcome, worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
};

#[test]
fn the_journal_publishes_only_at_the_next_position_and_resolves_retries() {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("journal-in-process")
        .expect("journal store");
    let serving = world.serving();
    assert_journal_holds(serving, 0);
    assert!(matches!(
        publication(serving, world.placement(), 2, payload(2)),
        Err(
            worth_store::physical_runtime::OrderedJournalPublicationFailure::PositionGap {
                head: 0,
                requested: 2
            }
        )
    ));
    for position in 1..=ENTRIES {
        let published = publication(serving, world.placement(), position, payload(position))
            .expect("publication");
        assert!(matches!(
            published,
            worth_store::physical_runtime::OrderedJournalPublished::Committed(_)
        ));
        assert_eq!(published.entry().position(), position);
    }
    assert_journal_holds(serving, ENTRIES);
    assert!(matches!(
        publication(serving, world.placement(), 5, payload(5)),
        Ok(worth_store::physical_runtime::OrderedJournalPublished::AlreadyCommitted(entry)) if entry.position() == 5
    ));
    assert!(matches!(
        publication(serving, world.placement(), 5, payload(6)),
        Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::PositionConflict(entry)) if entry.position() == 5
    ));
    assert!(matches!(
        publication(
            serving,
            world.placement(),
            ENTRIES + 2,
            payload(ENTRIES + 2)
        ),
        Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::PositionGap { .. })
    ));
    assert_journal_holds(serving, ENTRIES);
}

#[test]
fn a_journal_killed_before_its_first_checkpoint_recovers_every_entry() {
    assert_killed_journal_recovers(worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop::BeforeFirstCheckpoint);
}

#[test]
fn a_journal_killed_above_a_checkpoint_recovers_every_entry() {
    assert_killed_journal_recovers(worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop::AboveCheckpoint);
}

#[test]
fn a_publication_killed_at_each_mutation_seam_recovers_to_one_complete_head() {
    for stop in [
        worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop::BeforeWalAppend,
        worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop::AfterWalDurability,
        worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop::AfterDataSettlement,
        worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop::DuringRootPublication,
        worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop::AfterRootReplacement,
    ] {
        assert_killed_journal_recovers(stop);
    }
}

#[test]
fn canonical_journal_child() {
    let Some(role) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let marker = std::path::PathBuf::from(std::env::var_os(MARKER_ENV).expect("child marker"));
    let stop = worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::Stop::ALL
        .into_iter()
        .find(|stop| Some(stop.role()) == role.to_str())
        .expect("journal child role");
    child(&marker, stop);
}
#[cfg(all())]
#[path = "recovery_interruption/mod.rs"]
pub(crate) mod recovery_interruption;
